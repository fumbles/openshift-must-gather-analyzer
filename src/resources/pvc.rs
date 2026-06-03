// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;
use yaml_rust::YamlLoader;

#[derive(Debug, Clone)]
pub struct PersistentVolumeClaim {
    manifest: Manifest,
    namespace: Option<String>,
    phase: String,
    volume_name: Option<String>,
    storage_class: Option<String>,
    capacity: Option<String>,
    access_modes: Vec<String>,
}

impl Resource for PersistentVolumeClaim {
    fn from(manifest: Manifest) -> PersistentVolumeClaim {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let phase = yaml["status"]["phase"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let volume_name = yaml["spec"]["volumeName"].as_str().map(|s| s.to_string());
        let storage_class = yaml["spec"]["storageClassName"]
            .as_str()
            .map(|s| s.to_string());

        // Extract capacity from status
        let capacity = yaml["status"]["capacity"]["storage"]
            .as_str()
            .map(|s| s.to_string());

        // Extract access modes
        let mut access_modes = Vec::new();
        if let Some(modes) = yaml["spec"]["accessModes"].as_vec() {
            for mode in modes {
                if let Some(m) = mode.as_str() {
                    access_modes.push(m.to_string());
                }
            }
        }

        PersistentVolumeClaim {
            manifest,
            namespace,
            phase,
            volume_name,
            storage_class,
            capacity,
            access_modes,
        }
    }

    fn is_error(&self) -> bool {
        self.phase == "Lost"
    }

    fn is_warning(&self) -> bool {
        self.phase == "Pending"
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for PersistentVolumeClaim {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "PersistentVolumeClaim"
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
        match self.phase.as_str() {
            "Bound" => HealthStatus::Healthy,
            "Pending" => HealthStatus::Warning,
            "Lost" => HealthStatus::Error,
            _ => HealthStatus::Unknown,
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
        if self.phase == "Pending" {
            warnings.push("PVC is pending - waiting for volume provisioning".to_string());
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.phase == "Lost" {
            errors.push("PVC is in Lost state - bound volume no longer exists".to_string());
        }
        errors
    }

    fn metadata(&self) -> ResourceMetadata {
        ResourceMetadata {
            uid: self
                .manifest
                .uid()
                .unwrap_or_else(|| self.manifest.name.clone()),
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(format!(
            "PVC {} - Phase: {} - Storage: {}",
            ResourceV2::name(self),
            self.phase,
            self.capacity.as_deref().unwrap_or("N/A")
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("phase".to_string(), self.phase.clone());
        if let Some(vol) = &self.volume_name {
            fields.insert("volume_name".to_string(), vol.clone());
        }
        if let Some(sc) = &self.storage_class {
            fields.insert("storage_class".to_string(), sc.clone());
        }
        if let Some(cap) = &self.capacity {
            fields.insert("capacity".to_string(), cap.clone());
        }
        fields.insert("access_modes".to_string(), self.access_modes.join(", "));
        fields
    }
}

impl PersistentVolumeClaim {
    pub fn synthetic_bound(
        name: String,
        namespace: String,
        volume_name: String,
        storage_class: Option<String>,
        capacity: Option<String>,
    ) -> PersistentVolumeClaim {
        let storage_class_yaml = storage_class
            .as_ref()
            .map(|value| format!("  storageClassName: {}\n", value))
            .unwrap_or_default();
        let capacity_yaml = capacity
            .as_ref()
            .map(|value| format!("    storage: {}\n", value))
            .unwrap_or_default();

        let raw = format!(
            "---\napiVersion: v1\nkind: PersistentVolumeClaim\nmetadata:\n  name: {name}\n  namespace: {namespace}\nspec:\n{storage_class_yaml}  volumeName: {volume_name}\nstatus:\n  phase: Bound\n  capacity:\n{capacity_yaml}"
        );

        let yaml = YamlLoader::load_from_str(&raw).ok().and_then(|mut docs| {
            if docs.is_empty() {
                None
            } else {
                Some(docs.remove(0))
            }
        });

        PersistentVolumeClaim {
            manifest: Manifest {
                name,
                raw: format!("{raw}\n"),
                yaml,
            },
            namespace: Some(namespace),
            phase: "Bound".to_string(),
            volume_name: Some(volume_name),
            storage_class,
            capacity,
            access_modes: Vec::new(),
        }
    }
}
