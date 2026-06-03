// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ControlPlaneMachineSet {
    manifest: Manifest,
    ready: bool,
}

impl Resource for ControlPlaneMachineSet {
    fn from(manifest: Manifest) -> ControlPlaneMachineSet {
        let ready = manifest.as_yaml()["status"]["conditions"]
            .as_vec()
            .map(|conditions| {
                conditions.iter().any(|condition| {
                    condition["type"].as_str() == Some("Available")
                        && condition["status"].as_str() == Some("True")
                })
            })
            .unwrap_or(false);
        ControlPlaneMachineSet { manifest, ready }
    }

    fn is_error(&self) -> bool {
        !self.ready
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for ControlPlaneMachineSet {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "ControlPlaneMachineSet"
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
        if self.ready {
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
        if self.ready {
            Vec::new()
        } else {
            vec!["ControlPlaneMachineSet is not Available".to_string()]
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
            "ControlPlaneMachineSet {} - {}",
            ResourceV2::name(self),
            if self.ready {
                "Available"
            } else {
                "Unavailable"
            }
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("ready".to_string(), self.ready.to_string());
        fields
    }
}
