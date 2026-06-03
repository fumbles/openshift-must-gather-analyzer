// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Machine {
    manifest: Manifest,
    running: bool,
}

impl Resource for Machine {
    fn from(manifest: Manifest) -> Machine {
        let running = is_running_phase(&manifest);
        Machine { manifest, running }
    }

    fn is_error(&self) -> bool {
        !self.running
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Machine {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Machine"
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
        if self.running {
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
        if self.running {
            Vec::new()
        } else {
            vec!["Machine is not Running".to_string()]
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
        Some(format!(
            "Machine {} - Phase: {}",
            ResourceV2::name(self),
            self.manifest.as_yaml()["status"]["phase"]
                .as_str()
                .unwrap_or("Unknown")
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "phase".to_string(),
            self.manifest.as_yaml()["status"]["phase"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
        );
        if let Some(provider_id) = self.manifest.as_yaml()["spec"]["providerID"].as_str() {
            fields.insert("provider_id".to_string(), provider_id.to_string());
        }
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}

fn is_running_phase(manifest: &Manifest) -> bool {
    let phase = manifest.as_yaml()["status"]["phase"]
        .as_str()
        .unwrap_or("Unknown");
    if phase != "Running" {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_machine_is_running_phase_true() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/machine.openshift.io/machines/testdata-control-plane-0.yaml",
        ))
        .unwrap();
        assert_eq!(is_running_phase(&manifest), true)
    }

    #[test]
    fn test_machine_is_running_phase_false() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/machine.openshift.io/machines/testdata-control-plane-1.yaml"
        )).unwrap();
        assert_eq!(is_running_phase(&manifest), false)
    }
}
