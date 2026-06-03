// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Secret {
    manifest: Manifest,
    namespace: Option<String>,
    secret_type: String,
    data_keys: Vec<String>,
    immutable: bool,
}

impl Resource for Secret {
    fn from(manifest: Manifest) -> Secret {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let secret_type = yaml["type"].as_str().unwrap_or("Opaque").to_string();

        let data_keys = yaml["data"]
            .as_hash()
            .map(|entries| {
                entries
                    .keys()
                    .filter_map(|k| k.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let immutable = yaml["immutable"].as_bool().unwrap_or(false);

        Secret {
            manifest,
            namespace,
            secret_type,
            data_keys,
            immutable,
        }
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Secret {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Secret"
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
        HealthStatus::Healthy
    }

    fn conditions(&self) -> Vec<Condition> {
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
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(format!(
            "Secret {} - Type: {} - {} data keys",
            ResourceV2::name(self),
            self.secret_type,
            self.data_keys.len()
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("type".to_string(), self.secret_type.clone());
        fields.insert("data_count".to_string(), self.data_keys.len().to_string());
        fields.insert("immutable".to_string(), self.immutable.to_string());
        fields.insert("data_keys".to_string(), self.data_keys.join(", "));
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}
