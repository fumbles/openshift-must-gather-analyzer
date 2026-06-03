// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VolumeAttachment {
    manifest: Manifest,
    attacher: String,
    node_name: Option<String>,
    pv_name: Option<String>,
    attached: bool,
}

impl Resource for VolumeAttachment {
    fn from(manifest: Manifest) -> VolumeAttachment {
        let yaml = manifest.as_yaml();

        let attacher = yaml["spec"]["attacher"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let node_name = yaml["spec"]["nodeName"].as_str().map(|s| s.to_string());
        let pv_name = yaml["spec"]["source"]["persistentVolumeName"]
            .as_str()
            .map(|s| s.to_string());
        let attached = yaml["status"]["attached"].as_bool().unwrap_or(false);

        VolumeAttachment {
            manifest,
            attacher,
            node_name,
            pv_name,
            attached,
        }
    }

    fn is_error(&self) -> bool {
        !self.attached && self.has_attachment_error()
    }

    fn is_warning(&self) -> bool {
        !self.attached && !self.has_attachment_error()
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl VolumeAttachment {
    fn has_attachment_error(&self) -> bool {
        // Check if there's an attachment error in status
        let yaml = self.manifest.as_yaml();
        if let Some(error) = yaml["status"]["attachError"]["message"].as_str() {
            !error.is_empty()
        } else {
            false
        }
    }
}

impl ResourceV2 for VolumeAttachment {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "VolumeAttachment"
    }

    fn namespace(&self) -> Option<&str> {
        None // VolumeAttachment is cluster-scoped
    }

    fn uid(&self) -> &str {
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        if self.attached {
            HealthStatus::Healthy
        } else if self.has_attachment_error() {
            HealthStatus::Error
        } else {
            HealthStatus::Warning
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
        if !self.attached && !self.has_attachment_error() {
            warnings.push("Volume is not yet attached".to_string());
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !self.attached && self.has_attachment_error() {
            let yaml = self.manifest.as_yaml();
            if let Some(error_msg) = yaml["status"]["attachError"]["message"].as_str() {
                errors.push(format!("Attachment failed: {}", error_msg));
            } else {
                errors.push("Volume attachment failed".to_string());
            }
        }
        errors
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
            "VolumeAttachment {} - Node: {} - PV: {} - Attached: {}",
            ResourceV2::name(self),
            self.node_name.as_deref().unwrap_or("N/A"),
            self.pv_name.as_deref().unwrap_or("N/A"),
            self.attached
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("attacher".to_string(), self.attacher.clone());
        if let Some(node) = &self.node_name {
            fields.insert("node_name".to_string(), node.clone());
        }
        if let Some(pv) = &self.pv_name {
            fields.insert("pv_name".to_string(), pv.clone());
        }
        fields.insert("attached".to_string(), self.attached.to_string());
        fields
    }
}
