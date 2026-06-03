// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node {
    manifest: Manifest,
    is_ready: bool,
}

impl Resource for Node {
    fn from(manifest: Manifest) -> Node {
        let is_ready = manifest.has_condition_status("Ready", "True");
        Node { manifest, is_ready }
    }

    fn is_error(&self) -> bool {
        !self.is_ready
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Node {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Node"
    }

    fn namespace(&self) -> Option<&str> {
        None // Nodes are cluster-scoped
    }

    fn uid(&self) -> &str {
        // For now, use name as UID if not available
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        if self.is_ready {
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
        let mut warnings = Vec::new();
        for cond in ResourceV2::conditions(self) {
            if cond.type_ != "Ready" && cond.status == "True" {
                warnings.push(format!(
                    "{}: {}",
                    cond.type_,
                    cond.message.as_deref().unwrap_or("No message")
                ));
            }
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        if !self.is_ready {
            vec!["Node is not Ready".to_string()]
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
            "Node {} - Status: {}",
            ResourceV2::name(self),
            if self.is_ready { "Ready" } else { "NotReady" }
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "status".to_string(),
            if self.is_ready {
                "Ready".to_string()
            } else {
                "NotReady".to_string()
            },
        );
        fields
    }
}
