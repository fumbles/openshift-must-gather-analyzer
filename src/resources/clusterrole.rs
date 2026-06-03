// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ClusterRole {
    manifest: Manifest,
}

impl Resource for ClusterRole {
    fn from(manifest: Manifest) -> ClusterRole {
        ClusterRole { manifest }
    }

    fn is_error(&self) -> bool {
        false
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

impl ResourceV2 for ClusterRole {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "ClusterRole"
    }

    fn namespace(&self) -> Option<&str> {
        None
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
            namespace: None,
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(format!(
            "ClusterRole {} - {} rule(s)",
            ResourceV2::name(self),
            self.manifest.as_yaml()["rules"]
                .as_vec()
                .map(|rules| rules.len())
                .unwrap_or(0)
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "rule_count".to_string(),
            self.manifest.as_yaml()["rules"]
                .as_vec()
                .map(|rules| rules.len())
                .unwrap_or(0)
                .to_string(),
        );
        fields.insert(
            "aggregation_rule".to_string(),
            self.manifest.as_yaml()["aggregationRule"]
                .as_hash()
                .is_some()
                .to_string(),
        );
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}
