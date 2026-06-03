// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, Resource, ResourceLink, ResourceMetadata, ResourceV2,
    workload_dependency_relationships,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Deployment {
    manifest: Manifest,
    namespace: Option<String>,
    replicas: i64,
    ready_replicas: i64,
    available_replicas: i64,
    unavailable_replicas: i64,
}

impl Resource for Deployment {
    fn from(manifest: Manifest) -> Deployment {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let replicas = yaml["spec"]["replicas"].as_i64().unwrap_or(1);
        let ready_replicas = yaml["status"]["readyReplicas"].as_i64().unwrap_or(0);
        let available_replicas = yaml["status"]["availableReplicas"].as_i64().unwrap_or(0);
        let unavailable_replicas = yaml["status"]["unavailableReplicas"].as_i64().unwrap_or(0);

        Deployment {
            manifest,
            namespace,
            replicas,
            ready_replicas,
            available_replicas,
            unavailable_replicas,
        }
    }

    fn is_error(&self) -> bool {
        self.available_replicas < self.replicas || self.unavailable_replicas > 0
    }

    fn is_warning(&self) -> bool {
        self.ready_replicas < self.replicas && self.available_replicas > 0
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Deployment {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Deployment"
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
        if self.available_replicas < self.replicas || self.unavailable_replicas > 0 {
            HealthStatus::Error
        } else if self.ready_replicas < self.replicas {
            HealthStatus::Warning
        } else if self.manifest.has_condition_status("Available", "True")
            && self.manifest.has_condition_status("Progressing", "True")
        {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
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
        if self.ready_replicas < self.replicas && self.available_replicas > 0 {
            warnings.push(format!(
                "Only {}/{} replicas are ready",
                self.ready_replicas, self.replicas
            ));
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.available_replicas < self.replicas {
            errors.push(format!(
                "Only {}/{} replicas are available",
                self.available_replicas, self.replicas
            ));
        }
        if self.unavailable_replicas > 0 {
            errors.push(format!(
                "{} replicas are unavailable",
                self.unavailable_replicas
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
            "Deployment {} - {}/{} replicas available",
            ResourceV2::name(self),
            self.available_replicas,
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
            "available_replicas".to_string(),
            self.available_replicas.to_string(),
        );
        fields.insert(
            "unavailable_replicas".to_string(),
            self.unavailable_replicas.to_string(),
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
