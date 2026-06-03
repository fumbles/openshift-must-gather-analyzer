// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MachineAutoscaler {
    manifest: Manifest,
}

impl Resource for MachineAutoscaler {
    fn from(manifest: Manifest) -> MachineAutoscaler {
        MachineAutoscaler { manifest }
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for MachineAutoscaler {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "MachineAutoscaler"
    }

    fn namespace(&self) -> Option<&str> {
        self.manifest.as_yaml()["metadata"]["namespace"].as_str()
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
            namespace: self.manifest.namespace(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(format!("MachineAutoscaler {}", ResourceV2::name(self)))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        if let Some(min_replicas) = self.manifest.as_yaml()["spec"]["minReplicas"].as_i64() {
            fields.insert("min_replicas".to_string(), min_replicas.to_string());
        }
        if let Some(max_replicas) = self.manifest.as_yaml()["spec"]["maxReplicas"].as_i64() {
            fields.insert("max_replicas".to_string(), max_replicas.to_string());
        }
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}
