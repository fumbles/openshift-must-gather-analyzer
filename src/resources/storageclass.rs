// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StorageClass {
    manifest: Manifest,
    provisioner: String,
    reclaim_policy: Option<String>,
    volume_binding_mode: Option<String>,
    allow_volume_expansion: bool,
}

impl Resource for StorageClass {
    fn from(manifest: Manifest) -> StorageClass {
        let yaml = manifest.as_yaml();

        let provisioner = yaml["provisioner"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let reclaim_policy = yaml["reclaimPolicy"].as_str().map(|s| s.to_string());
        let volume_binding_mode = yaml["volumeBindingMode"].as_str().map(|s| s.to_string());
        let allow_volume_expansion = yaml["allowVolumeExpansion"].as_bool().unwrap_or(false);

        StorageClass {
            manifest,
            provisioner,
            reclaim_policy,
            volume_binding_mode,
            allow_volume_expansion,
        }
    }

    fn is_error(&self) -> bool {
        false // StorageClass is a configuration object
    }

    fn is_warning(&self) -> bool {
        false
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for StorageClass {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "StorageClass"
    }

    fn namespace(&self) -> Option<&str> {
        None // StorageClass is cluster-scoped
    }

    fn uid(&self) -> &str {
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        // StorageClass is a configuration object, always healthy
        HealthStatus::Healthy
    }

    fn conditions(&self) -> Vec<Condition> {
        // StorageClass doesn't have conditions
        Vec::new()
    }

    fn warnings(&self) -> Vec<String> {
        Vec::new()
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
            "StorageClass {} - Provisioner: {} - Binding: {}",
            ResourceV2::name(self),
            self.provisioner,
            self.volume_binding_mode.as_deref().unwrap_or("Immediate")
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("provisioner".to_string(), self.provisioner.clone());
        if let Some(policy) = &self.reclaim_policy {
            fields.insert("reclaim_policy".to_string(), policy.clone());
        }
        if let Some(mode) = &self.volume_binding_mode {
            fields.insert("volume_binding_mode".to_string(), mode.clone());
        }
        fields.insert(
            "allow_volume_expansion".to_string(),
            self.allow_volume_expansion.to_string(),
        );
        fields
    }
}
