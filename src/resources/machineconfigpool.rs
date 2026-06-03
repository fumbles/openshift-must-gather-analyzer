// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MachineConfigPool {
    manifest: Manifest,
    degraded: bool,
    updating: bool,
}

impl Resource for MachineConfigPool {
    fn from(manifest: Manifest) -> MachineConfigPool {
        let degraded = manifest.has_condition_status("Degraded", "True");
        let updating = manifest.has_condition_status("Updating", "True");
        MachineConfigPool {
            manifest,
            degraded,
            updating,
        }
    }

    fn is_error(&self) -> bool {
        self.degraded
    }

    fn is_warning(&self) -> bool {
        self.updating
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
        if self.updating {
            conditions.push(String::from("Updating"));
        }
        conditions
    }
}

impl ResourceV2 for MachineConfigPool {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "MachineConfigPool"
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
        if self.degraded {
            HealthStatus::Error
        } else if self.updating {
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
        if self.updating {
            vec!["MachineConfigPool is updating".to_string()]
        } else {
            Vec::new()
        }
    }

    fn errors(&self) -> Vec<String> {
        if self.degraded {
            vec!["MachineConfigPool is degraded".to_string()]
        } else {
            Vec::new()
        }
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
            "MachineConfigPool {} - {}",
            ResourceV2::name(self),
            if self.degraded {
                "Degraded"
            } else if self.updating {
                "Updating"
            } else {
                "Healthy"
            }
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("degraded".to_string(), self.degraded.to_string());
        fields.insert("updating".to_string(), self.updating.to_string());
        if let Some(value) = self.manifest.as_yaml()["status"]["machineCount"].as_i64() {
            fields.insert("machine_count".to_string(), value.to_string());
        }
        if let Some(value) = self.manifest.as_yaml()["status"]["readyMachineCount"].as_i64() {
            fields.insert("ready_machine_count".to_string(), value.to_string());
        }
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_machineconfigpool_degraded_false() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/machineconfiguration.openshift.io/machineconfigpools/master.yaml",
        ))
        .unwrap();
        let mcp = <MachineConfigPool as Resource>::from(manifest);
        assert_eq!(Resource::is_error(&mcp), false)
    }

    #[test]
    fn test_machineconfigpool_degraded_true() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/machineconfiguration.openshift.io/machineconfigpools/worker.yaml",
        ))
        .unwrap();
        let mcp = <MachineConfigPool as Resource>::from(manifest);
        assert_eq!(Resource::is_error(&mcp), true)
    }

    #[test]
    fn test_machineconfigpool_updating_true() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/machineconfiguration.openshift.io/machineconfigpools/worker.yaml",
        ))
        .unwrap();
        let mcp = <MachineConfigPool as Resource>::from(manifest);
        assert_eq!(Resource::is_warning(&mcp), false)
    }
}
