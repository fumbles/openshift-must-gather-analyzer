// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SecurityContextConstraint {
    manifest: Manifest,
    allow_privileged: bool,
}

impl Resource for SecurityContextConstraint {
    fn from(manifest: Manifest) -> SecurityContextConstraint {
        let allow_privileged = manifest.as_yaml()["allowPrivilegedContainer"]
            .as_bool()
            .unwrap_or(false);

        SecurityContextConstraint {
            manifest,
            allow_privileged,
        }
    }

    fn is_error(&self) -> bool {
        false
    }

    fn is_warning(&self) -> bool {
        self.allow_privileged
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for SecurityContextConstraint {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "SecurityContextConstraint"
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
        if self.allow_privileged {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    fn conditions(&self) -> Vec<Condition> {
        Vec::new()
    }

    fn warnings(&self) -> Vec<String> {
        if self.allow_privileged {
            vec!["Privileged containers are allowed".to_string()]
        } else {
            Vec::new()
        }
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
            "SecurityContextConstraint {} - {} user(s), {} group(s)",
            ResourceV2::name(self),
            self.manifest.as_yaml()["users"]
                .as_vec()
                .map(|users| users.len())
                .unwrap_or(0),
            self.manifest.as_yaml()["groups"]
                .as_vec()
                .map(|groups| groups.len())
                .unwrap_or(0)
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "priority".to_string(),
            self.manifest.as_yaml()["priority"]
                .as_i64()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unset".to_string()),
        );
        fields.insert(
            "allow_privileged_container".to_string(),
            self.allow_privileged.to_string(),
        );
        fields.insert(
            "user_count".to_string(),
            self.manifest.as_yaml()["users"]
                .as_vec()
                .map(|users| users.len())
                .unwrap_or(0)
                .to_string(),
        );
        fields.insert(
            "group_count".to_string(),
            self.manifest.as_yaml()["groups"]
                .as_vec()
                .map(|groups| groups.len())
                .unwrap_or(0)
                .to_string(),
        );
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}
