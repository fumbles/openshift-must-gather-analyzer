// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IngressController {
    manifest: Manifest,
    namespace: Option<String>,
    domain: Option<String>,
    replicas: i64,
    available_replicas: i64,
}

impl Resource for IngressController {
    fn from(manifest: Manifest) -> IngressController {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let domain = yaml["spec"]["domain"].as_str().map(|s| s.to_string());
        let replicas = yaml["spec"]["replicas"].as_i64().unwrap_or(2);
        let available_replicas = yaml["status"]["availableReplicas"].as_i64().unwrap_or(0);

        IngressController {
            manifest,
            namespace,
            domain,
            replicas,
            available_replicas,
        }
    }

    fn is_error(&self) -> bool {
        self.available_replicas == 0
    }

    fn is_warning(&self) -> bool {
        self.available_replicas < self.replicas && self.available_replicas > 0
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for IngressController {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "IngressController"
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
        if self.available_replicas == 0 {
            HealthStatus::Error
        } else if self.available_replicas < self.replicas {
            HealthStatus::Warning
        } else if self.manifest.has_condition_status("Available", "True") {
            HealthStatus::Healthy
        } else if self.manifest.has_condition_status("Degraded", "True") {
            HealthStatus::Warning
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

        if self.available_replicas < self.replicas && self.available_replicas > 0 {
            warnings.push(format!(
                "Only {}/{} replicas are available",
                self.available_replicas, self.replicas
            ));
        }

        if self.manifest.has_condition_status("Degraded", "True") {
            warnings.push("IngressController is degraded".to_string());
        }

        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.available_replicas == 0 {
            errors.push("No replicas are available - ingress is unavailable".to_string());
        }

        if !self.manifest.has_condition_status("Available", "True") {
            errors.push("IngressController is not available".to_string());
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
            "IngressController {} - Domain: {} - Replicas: {}/{}",
            ResourceV2::name(self),
            self.domain.as_deref().unwrap_or("N/A"),
            self.available_replicas,
            self.replicas
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        if let Some(domain) = &self.domain {
            fields.insert("domain".to_string(), domain.clone());
        }
        fields.insert("replicas".to_string(), self.replicas.to_string());
        fields.insert(
            "available_replicas".to_string(),
            self.available_replicas.to_string(),
        );
        fields
    }
}
