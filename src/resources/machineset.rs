// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MachineSet {
    manifest: Manifest,
    autoscaling: bool,
    replicas: String,
}

impl MachineSet {
    pub fn is_autoscaling(&self) -> bool {
        self.autoscaling
    }

    pub fn replicas(&self) -> &String {
        &self.replicas
    }
}

impl Resource for MachineSet {
    fn from(manifest: Manifest) -> MachineSet {
        let autoscaling = has_autoscaling_annotations(&manifest);
        let replicas = status_replicas(&manifest);
        MachineSet {
            manifest,
            autoscaling,
            replicas,
        }
    }

    fn is_error(&self) -> bool {
        false
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for MachineSet {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "MachineSet"
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

    fn errors(&self) -> Vec<String> {
        Vec::new()
    }

    fn warnings(&self) -> Vec<String> {
        if self.autoscaling {
            vec!["MachineSet is managed by autoscaling annotations".to_string()]
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
            namespace: self.manifest.namespace(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(format!(
            "MachineSet {} - Replicas: {}",
            ResourceV2::name(self),
            self.replicas
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("replicas".to_string(), self.replicas.clone());
        fields.insert("autoscaling".to_string(), self.autoscaling.to_string());
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}

fn has_autoscaling_annotations(manifest: &Manifest) -> bool {
    !(manifest.as_yaml()["metadata"]["annotations"]
        ["machine.openshift.io/cluster-api-autoscaler-node-group-min-size"]
        .is_badvalue()
        && manifest.as_yaml()["metadata"]["annotations"]
            ["machine.openshift.io/cluster-api-autoscaler-node-group-max-size"]
            .is_badvalue())
}

fn status_replicas(manifest: &Manifest) -> String {
    if manifest.as_yaml()["status"]["replicas"].is_badvalue() {
        String::from("Not Found")
    } else {
        match manifest.as_yaml()["status"]["replicas"].as_i64() {
            Some(v) => format!("{}", v),
            None => String::from("Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_machineset_has_autoscaling_annotations_true() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/machine.openshift.io/machinesets/testdata-compute-region-2.yaml",
        ))
        .unwrap();
        assert_eq!(has_autoscaling_annotations(&manifest), true)
    }

    #[test]
    fn test_machineset_has_autoscaling_annotations_false() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/machine.openshift.io/machinesets/testdata-compute-region-1.yaml",
        )).unwrap();
        assert_eq!(has_autoscaling_annotations(&manifest), false)
    }

    #[test]
    fn test_machineset_status_replicas() {
        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/namespaces/openshift-machine-api/machine.openshift.io/machinesets/testdata-compute-region-2.yaml",
        )).unwrap();
        assert_eq!(status_replicas(&manifest), String::from("0"))
    }
}
