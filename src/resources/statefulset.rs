// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, Resource, ResourceLink, ResourceMetadata, ResourceV2,
    workload_dependency_relationships,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StatefulSet {
    manifest: Manifest,
    namespace: Option<String>,
    replicas: i64,
    ready_replicas: i64,
    current_replicas: i64,
    updated_replicas: i64,
}

impl Resource for StatefulSet {
    fn from(manifest: Manifest) -> StatefulSet {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let replicas = yaml["spec"]["replicas"].as_i64().unwrap_or(1);
        let ready_replicas = yaml["status"]["readyReplicas"].as_i64().unwrap_or(0);
        let current_replicas = yaml["status"]["currentReplicas"].as_i64().unwrap_or(0);
        let updated_replicas = yaml["status"]["updatedReplicas"].as_i64().unwrap_or(0);

        StatefulSet {
            manifest,
            namespace,
            replicas,
            ready_replicas,
            current_replicas,
            updated_replicas,
        }
    }

    fn is_error(&self) -> bool {
        self.ready_replicas < self.replicas
    }

    fn is_warning(&self) -> bool {
        self.updated_replicas < self.replicas && self.ready_replicas > 0
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for StatefulSet {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "StatefulSet"
    }

    fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    fn uid(&self) -> &str {
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        if self.ready_replicas < self.replicas {
            HealthStatus::Error
        } else if self.updated_replicas < self.replicas {
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
        let mut warnings = Vec::new();
        if self.updated_replicas < self.replicas && self.ready_replicas > 0 {
            warnings.push(format!(
                "Only {}/{} replicas are updated",
                self.updated_replicas, self.replicas
            ));
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.ready_replicas < self.replicas {
            errors.push(format!(
                "Only {}/{} replicas are ready",
                self.ready_replicas, self.replicas
            ));
        }
        errors
    }

    fn metadata(&self) -> ResourceMetadata {
        ResourceMetadata {
            uid: self
                .manifest
                .uid()
                .unwrap_or_else(|| self.manifest.name.clone()),
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(format!(
            "StatefulSet {} - {}/{} replicas ready",
            ResourceV2::name(self),
            self.ready_replicas,
            self.replicas
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("replicas".to_string(), self.replicas.to_string());
        fields.insert(
            "ready_replicas".to_string(),
            self.ready_replicas.to_string(),
        );
        fields.insert(
            "current_replicas".to_string(),
            self.current_replicas.to_string(),
        );
        fields.insert(
            "updated_replicas".to_string(),
            self.updated_replicas.to_string(),
        );
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        workload_dependency_relationships(&self.manifest)
    }
}
