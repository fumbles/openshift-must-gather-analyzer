// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MachineHealthCheck {
    manifest: Manifest,
    remediation_allowed: bool,
}

impl Resource for MachineHealthCheck {
    fn from(manifest: Manifest) -> MachineHealthCheck {
        let remediation_allowed = manifest.has_condition_status("RemediationAllowed", "True");
        MachineHealthCheck {
            manifest,
            remediation_allowed,
        }
    }

    fn is_error(&self) -> bool {
        !self.remediation_allowed
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for MachineHealthCheck {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "MachineHealthCheck"
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
        if self.remediation_allowed {
            HealthStatus::Healthy
        } else {
            HealthStatus::Error
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
        Vec::new()
    }

    fn errors(&self) -> Vec<String> {
        if self.remediation_allowed {
            Vec::new()
        } else {
            vec!["MachineHealthCheck remediation is not allowed".to_string()]
        }
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
        let healthy = self.manifest.as_yaml()["status"]["currentHealthy"]
            .as_i64()
            .unwrap_or(0);
        let expected = self.manifest.as_yaml()["status"]["expectedMachines"]
            .as_i64()
            .unwrap_or(0);
        Some(format!(
            "MachineHealthCheck {} - {}/{} healthy",
            ResourceV2::name(self),
            healthy,
            expected
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "remediation_allowed".to_string(),
            self.remediation_allowed.to_string(),
        );
        if let Some(value) = self.manifest.as_yaml()["status"]["currentHealthy"].as_i64() {
            fields.insert("current_healthy".to_string(), value.to_string());
        }
        if let Some(value) = self.manifest.as_yaml()["status"]["expectedMachines"].as_i64() {
            fields.insert("expected_machines".to_string(), value.to_string());
        }
        if let Some(value) = self.manifest.as_yaml()["status"]["remediationsAllowed"].as_i64() {
            fields.insert("remediations_allowed".to_string(), value.to_string());
        }
        if let Some(value) = self.manifest.as_yaml()["spec"]["maxUnhealthy"].as_str() {
            fields.insert("max_unhealthy".to_string(), value.to_string());
        }
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}
