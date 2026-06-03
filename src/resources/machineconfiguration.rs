// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MachineConfiguration {
    manifest: Manifest,
    boot_image_update_degraded: bool,
    boot_image_update_progressing: bool,
}

impl Resource for MachineConfiguration {
    fn from(manifest: Manifest) -> MachineConfiguration {
        let boot_image_update_degraded =
            manifest.has_condition_status("BootImageUpdateDegraded", "True");
        let boot_image_update_progressing =
            manifest.has_condition_status("BootImageUpdateProgressing", "True");
        MachineConfiguration {
            manifest,
            boot_image_update_degraded,
            boot_image_update_progressing,
        }
    }

    fn is_error(&self) -> bool {
        self.boot_image_update_degraded
    }

    fn is_warning(&self) -> bool {
        self.boot_image_update_progressing
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }

    fn conditions(&self) -> Vec<String> {
        let mut conditions = Vec::new();
        if self.boot_image_update_degraded {
            conditions.push(String::from("BootImageUpdateDegraded"));
        }
        if self.boot_image_update_progressing {
            conditions.push(String::from("BootImageUpdateProgressing"));
        }
        conditions
    }
}

impl ResourceV2 for MachineConfiguration {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "MachineConfiguration"
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
        if self.boot_image_update_degraded {
            HealthStatus::Error
        } else if self.boot_image_update_progressing {
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
        if self.boot_image_update_progressing {
            vec!["Boot image update is progressing".to_string()]
        } else {
            Vec::new()
        }
    }

    fn errors(&self) -> Vec<String> {
        if self.boot_image_update_degraded {
            vec!["Boot image update is degraded".to_string()]
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
        Some(format!("MachineConfiguration {}", ResourceV2::name(self)))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "boot_image_update_degraded".to_string(),
            self.boot_image_update_degraded.to_string(),
        );
        fields.insert(
            "boot_image_update_progressing".to_string(),
            self.boot_image_update_progressing.to_string(),
        );
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_machineconfiguration_not_degraded() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/operator.openshift.io/machineconfigurations/cluster.yaml",
        ))
        .unwrap();
        let mc = <MachineConfiguration as Resource>::from(manifest);
        assert_eq!(Resource::is_error(&mc), false)
    }

    #[test]
    fn test_machineconfiguration_not_progressing() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/operator.openshift.io/machineconfigurations/cluster.yaml",
        ))
        .unwrap();
        let mc = <MachineConfiguration as Resource>::from(manifest);
        assert_eq!(Resource::is_warning(&mc), false)
    }
}
