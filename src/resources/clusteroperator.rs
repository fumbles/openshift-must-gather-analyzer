// Copyright (C) 2023 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, RelationshipType, Resource, ResourceLink, ResourceMetadata, ResourceV2,
};
use std::collections::HashMap;

fn related_object_kind(resource: &str) -> String {
    match resource {
        "namespaces" => "Namespace",
        "deployments" => "Deployment",
        "daemonsets" => "DaemonSet",
        "statefulsets" => "StatefulSet",
        "replicasets" => "ReplicaSet",
        "pods" => "Pod",
        "configmaps" => "ConfigMap",
        "secrets" => "Secret",
        "services" => "Service",
        "routes" => "Route",
        "serviceaccounts" => "ServiceAccount",
        "clusterserviceversions" => "ClusterServiceVersion",
        "subscriptions" => "Subscription",
        "packageservers" => "PackageServer",
        _ => resource,
    }
    .to_string()
}

#[derive(Debug, Clone)]
pub struct ClusterOperator {
    manifest: Manifest,
    available: bool,
    degraded: bool,
    upgradeable: bool,
    has_upgradeable: bool,
}

impl Resource for ClusterOperator {
    fn from(manifest: Manifest) -> ClusterOperator {
        let available = manifest.has_condition_status("Available", "True");
        let degraded = manifest.has_condition_status("Degraded", "True");
        let upgradeable = manifest.has_condition_status("Upgradeable", "True");
        let has_upgradeable = manifest.has_condition("Upgradeable");
        ClusterOperator {
            manifest,
            available,
            degraded,
            upgradeable,
            has_upgradeable,
        }
    }

    fn is_error(&self) -> bool {
        !self.available || self.degraded
    }

    fn is_warning(&self) -> bool {
        !self.upgradeable && self.has_upgradeable
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }

    fn conditions(&self) -> Vec<String> {
        let mut conditions = Vec::new();

        if self.degraded {
            conditions.push(String::from("Degraded"));
        }
        if self.manifest.has_condition_status("Progressing", "True") {
            conditions.push(String::from("Progressing"));
        }

        conditions
    }
}

impl ResourceV2 for ClusterOperator {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "ClusterOperator"
    }

    fn namespace(&self) -> Option<&str> {
        None // ClusterOperators are cluster-scoped
    }

    fn uid(&self) -> &str {
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        if !self.available || self.degraded {
            HealthStatus::Error
        } else if !self.upgradeable && self.has_upgradeable {
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
        let mut warnings = Vec::new();
        if !self.upgradeable && self.has_upgradeable {
            warnings.push("Operator is not upgradeable".to_string());
        }
        if self.manifest.has_condition_status("Progressing", "True") {
            warnings.push("Operator is progressing".to_string());
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !self.available {
            errors.push("Operator is not available".to_string());
        }
        if self.degraded {
            errors.push("Operator is degraded".to_string());
        }
        errors
    }

    fn metadata(&self) -> ResourceMetadata {
        ResourceMetadata {
            uid: self
                .manifest
                .uid()
                .unwrap_or_else(|| self.manifest.name.clone()),
            namespace: None,
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        let status = if self.available && !self.degraded {
            "Available"
        } else if self.degraded {
            "Degraded"
        } else {
            "Unavailable"
        };
        Some(format!(
            "ClusterOperator {} - Status: {}",
            ResourceV2::name(self),
            status
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("available".to_string(), self.available.to_string());
        fields.insert("degraded".to_string(), self.degraded.to_string());
        fields.insert("upgradeable".to_string(), self.upgradeable.to_string());
        fields
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        let mut links = Vec::new();

        if let Some(items) = self.manifest.as_yaml()["status"]["relatedObjects"].as_vec() {
            for item in items {
                let resource = item["resource"].as_str().unwrap_or("");
                let name = item["name"].as_str().unwrap_or("");
                if resource.is_empty() || name.is_empty() {
                    continue;
                }

                links.push(ResourceLink {
                    kind: related_object_kind(resource),
                    name: name.to_string(),
                    namespace: item["namespace"].as_str().map(|s| s.to_string()),
                    relationship: RelationshipType::References,
                });
            }
        }

        links
    }
}
