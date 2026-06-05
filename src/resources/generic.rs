// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, RelationshipType, Resource, ResourceLink, ResourceMetadata, ResourceV2,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GenericResource {
    manifest: Manifest,
    kind: String,
    namespace: Option<String>,
    relationships: Vec<ResourceLink>,
}

impl Resource for GenericResource {
    fn from(manifest: Manifest) -> GenericResource {
        let kind = manifest.as_yaml()["kind"]
            .as_str()
            .unwrap_or("Resource")
            .to_string();
        let namespace = manifest.namespace();

        GenericResource {
            manifest,
            kind,
            namespace,
            relationships: Vec::new(),
        }
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl GenericResource {
    pub fn add_relationships(&mut self, relationships: Vec<ResourceLink>) {
        self.relationships.extend(relationships);
    }

    pub fn crd_group(&self) -> Option<&str> {
        self.manifest.as_yaml()["spec"]["group"].as_str()
    }

    pub fn crd_plural(&self) -> Option<&str> {
        self.manifest.as_yaml()["spec"]["names"]["plural"].as_str()
    }

    pub fn crd_kind(&self) -> Option<&str> {
        self.manifest.as_yaml()["spec"]["names"]["kind"]
            .as_str()
            .or_else(|| self.manifest.as_yaml()["status"]["acceptedNames"]["kind"].as_str())
    }

    pub fn crd_scope(&self) -> Option<&str> {
        self.manifest.as_yaml()["spec"]["scope"].as_str()
    }
}

impl ResourceV2 for GenericResource {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        &self.kind
    }

    fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    fn uid(&self) -> &str {
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        if has_condition_status(&self.manifest, &["Degraded", "Failing", "Failure"], "True") {
            HealthStatus::Error
        } else if has_condition_status(&self.manifest, &["Available", "Ready"], "False")
            || has_condition_status(&self.manifest, &["Progressing"], "True")
        {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    fn conditions(&self) -> Vec<Condition> {
        self.manifest
            .conditions()
            .into_iter()
            .map(|(type_, status, reason, message)| Condition {
                type_,
                status,
                reason,
                message,
                last_transition: None,
            })
            .collect()
    }

    fn warnings(&self) -> Vec<String> {
        ResourceV2::conditions(self)
            .into_iter()
            .filter(|condition| {
                (matches!(condition.type_.as_str(), "Available" | "Ready")
                    && condition.status == "False")
                    || (condition.type_ == "Progressing" && condition.status == "True")
            })
            .map(format_condition_message)
            .collect()
    }

    fn errors(&self) -> Vec<String> {
        ResourceV2::conditions(self)
            .into_iter()
            .filter(|condition| {
                matches!(condition.type_.as_str(), "Degraded" | "Failing" | "Failure")
                    && condition.status == "True"
            })
            .map(format_condition_message)
            .collect()
    }

    fn metadata(&self) -> ResourceMetadata {
        ResourceMetadata {
            uid: self
                .manifest
                .uid()
                .unwrap_or_else(|| format!("{}__{}", self.kind.to_lowercase(), self.manifest.name)),
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        if let Some(api_version) = self.manifest.as_yaml()["apiVersion"].as_str() {
            fields.insert("api_version".to_string(), api_version.to_string());
        }
        if self.kind == "CustomResourceDefinition" {
            if let Some(group) = self.crd_group() {
                fields.insert("crd_group".to_string(), group.to_string());
            }
            if let Some(plural) = self.crd_plural() {
                fields.insert("crd_plural".to_string(), plural.to_string());
            }
            if let Some(kind) = self.crd_kind() {
                fields.insert("crd_kind".to_string(), kind.to_string());
            }
            if let Some(scope) = self.crd_scope() {
                fields.insert("crd_scope".to_string(), scope.to_string());
            }
        }
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        let mut links = self.relationships.clone();
        let mut seen = HashSet::new();
        for link in &links {
            seen.insert((link.kind.clone(), link.name.clone(), link.namespace.clone()));
        }

        if let Some(refs) = self.manifest.as_yaml()["status"]["components"]["refs"].as_vec() {
            for reference in refs {
                let kind = reference["kind"].as_str().unwrap_or("");
                let name = reference["name"].as_str().unwrap_or("");
                if kind.is_empty() || name.is_empty() {
                    continue;
                }

                let namespace = reference["namespace"].as_str().map(|s| s.to_string());
                let key = (kind.to_string(), name.to_string(), namespace.clone());
                if seen.insert(key) {
                    links.push(ResourceLink {
                        kind: kind.to_string(),
                        name: name.to_string(),
                        namespace,
                        relationship: RelationshipType::References,
                    });
                }
            }
        }

        links
    }
}

fn has_condition_status(manifest: &Manifest, condition_types: &[&str], status: &str) -> bool {
    manifest.as_yaml()["status"]["conditions"]
        .as_vec()
        .into_iter()
        .flatten()
        .any(|condition| {
            condition["status"].as_str() == Some(status)
                && condition["type"]
                    .as_str()
                    .is_some_and(|type_| condition_types.contains(&type_))
        })
}

fn format_condition_message(condition: Condition) -> String {
    let mut message = format!("{} is {}", condition.type_, condition.status);
    if let Some(reason) = condition.reason {
        if !reason.is_empty() {
            message.push_str(&format!(" ({reason})"));
        }
    }
    if let Some(detail) = condition.message {
        if !detail.is_empty() {
            message.push_str(&format!(": {detail}"));
        }
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_manifest(raw: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mga-generic-resource-test-{unique}.yaml"));
        fs::write(&path, raw).unwrap();
        path
    }

    #[test]
    fn generic_resource_relationships_include_olm_operator_component_refs() {
        let path = write_temp_manifest(
            r#"apiVersion: operators.coreos.com/v1
kind: Operator
metadata:
  name: isf-operator.ibm-spectrum-fusion-ns
status:
  components:
    refs:
    - apiVersion: operators.coreos.com/v1alpha1
      kind: ClusterServiceVersion
      name: isf-operator.v2.12.2
      namespace: ibm-spectrum-fusion-ns
    - apiVersion: rbac.authorization.k8s.io/v1
      kind: ClusterRole
      name: isf-operator-role
    - apiVersion: rbac.authorization.k8s.io/v1
      kind: ClusterRole
      name: isf-operator-role
"#,
        );
        let resource = <GenericResource as Resource>::from(Manifest::from(path).unwrap());

        let relationships = resource.relationships();

        assert_eq!(relationships.len(), 2);
        assert!(relationships.iter().any(|link| {
            link.kind == "ClusterServiceVersion"
                && link.name == "isf-operator.v2.12.2"
                && link.namespace.as_deref() == Some("ibm-spectrum-fusion-ns")
                && link.relationship == RelationshipType::References
        }));
        assert!(relationships.iter().any(|link| {
            link.kind == "ClusterRole"
                && link.name == "isf-operator-role"
                && link.namespace.is_none()
                && link.relationship == RelationshipType::References
        }));
    }
}
