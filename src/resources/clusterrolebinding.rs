// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, RelationshipType, Resource, ResourceLink, ResourceMetadata, ResourceV2,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ClusterRoleBinding {
    manifest: Manifest,
}

impl Resource for ClusterRoleBinding {
    fn from(manifest: Manifest) -> ClusterRoleBinding {
        ClusterRoleBinding { manifest }
    }

    fn is_error(&self) -> bool {
        false
    }

    fn is_warning(&self) -> bool {
        false
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for ClusterRoleBinding {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "ClusterRoleBinding"
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
        HealthStatus::Healthy
    }

    fn conditions(&self) -> Vec<Condition> {
        Vec::new()
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
            namespace: None,
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        let role_ref = self.manifest.as_yaml()["roleRef"]["name"]
            .as_str()
            .unwrap_or("Unknown");
        Some(format!(
            "ClusterRoleBinding {} - {} subject(s) -> {}",
            ResourceV2::name(self),
            self.manifest.as_yaml()["subjects"]
                .as_vec()
                .map(|subjects| subjects.len())
                .unwrap_or(0),
            role_ref
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "role_ref".to_string(),
            self.manifest.as_yaml()["roleRef"]["name"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
        );
        fields.insert(
            "subject_count".to_string(),
            self.manifest.as_yaml()["subjects"]
                .as_vec()
                .map(|subjects| subjects.len())
                .unwrap_or(0)
                .to_string(),
        );
        fields
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        let mut links = Vec::new();

        if let Some(role_name) = self.manifest.as_yaml()["roleRef"]["name"].as_str() {
            links.push(ResourceLink {
                kind: "ClusterRole".to_string(),
                name: role_name.to_string(),
                namespace: None,
                relationship: RelationshipType::References,
            });
        }

        links
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}
