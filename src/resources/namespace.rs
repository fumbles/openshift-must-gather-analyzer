// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Namespace {
    manifest: Manifest,
    pub phase: String,
}

impl Namespace {
    pub fn new(name: String) -> Self {
        let manifest = Manifest {
            name: name.clone(),
            raw: name,
            yaml: None,
        };
        Namespace {
            manifest,
            phase: "Active".to_string(),
        }
    }
}

impl Resource for Namespace {
    fn from(manifest: Manifest) -> Namespace {
        let phase = manifest.as_yaml()["status"]["phase"]
            .as_str()
            .unwrap_or("Active")
            .to_string();

        Namespace { manifest, phase }
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Namespace {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Namespace"
    }

    fn namespace(&self) -> Option<&str> {
        None // Namespaces are cluster-scoped
    }

    fn uid(&self) -> &str {
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        if self.phase == "Active" {
            HealthStatus::Healthy
        } else {
            HealthStatus::Warning
        }
    }

    fn conditions(&self) -> Vec<Condition> {
        Vec::new()
    }

    fn warnings(&self) -> Vec<String> {
        if self.phase != "Active" {
            vec![format!("Namespace is in {} phase", self.phase)]
        } else {
            Vec::new()
        }
    }

    fn errors(&self) -> Vec<String> {
        Vec::new()
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
        Some(format!(
            "Namespace {} - Phase: {}",
            self.manifest.name, self.phase
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("phase".to_string(), self.phase.clone());
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_namespace_manifest() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("mga-namespace-test-{}", unique));
        fs::create_dir_all(&dir).unwrap();

        let manifest_path = dir.join("test-ns.yaml");
        fs::write(
            &manifest_path,
            r#"apiVersion: v1
kind: Namespace
metadata:
  name: test-ns
  uid: "1234"
  creationTimestamp: "2024-06-24T12:04:16Z"
  labels:
    kubernetes.io/metadata.name: test-ns
  annotations:
    openshift.io/display-name: ""
status:
  phase: Active
"#,
        )
        .unwrap();

        manifest_path
    }

    #[test]
    fn test_namespace_from_manifest_preserves_raw_yaml_and_metadata() {
        let manifest = Manifest::from(write_temp_namespace_manifest()).unwrap();

        let namespace = <Namespace as Resource>::from(manifest);

        assert_eq!(ResourceV2::name(&namespace), "test-ns");
        assert!(ResourceV2::raw(&namespace).contains("kind: Namespace"));
        assert!(ResourceV2::raw(&namespace).contains("metadata:"));
        assert_eq!(namespace.phase, "Active");
        assert_eq!(namespace.metadata().uid, "1234");
        assert!(!namespace.metadata().labels.is_empty());
    }
}
