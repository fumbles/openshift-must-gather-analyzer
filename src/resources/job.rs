// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, Resource, ResourceLink, ResourceMetadata, ResourceV2,
    workload_dependency_relationships,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Job {
    manifest: Manifest,
    namespace: Option<String>,
    completions: i64,
    succeeded: i64,
    failed: i64,
    active: i64,
}

impl Resource for Job {
    fn from(manifest: Manifest) -> Job {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let completions = yaml["spec"]["completions"].as_i64().unwrap_or(1);
        let succeeded = yaml["status"]["succeeded"].as_i64().unwrap_or(0);
        let failed = yaml["status"]["failed"].as_i64().unwrap_or(0);
        let active = yaml["status"]["active"].as_i64().unwrap_or(0);

        Job {
            manifest,
            namespace,
            completions,
            succeeded,
            failed,
            active,
        }
    }

    fn is_error(&self) -> bool {
        self.failed > 0 || self.manifest.has_condition_status("Failed", "True")
    }

    fn is_warning(&self) -> bool {
        self.active > 0 && self.succeeded < self.completions
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Job {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Job"
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
        if self.failed > 0 || self.manifest.has_condition_status("Failed", "True") {
            HealthStatus::Error
        } else if self.manifest.has_condition_status("Complete", "True") {
            HealthStatus::Healthy
        } else if self.active > 0 {
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
        if self.active > 0 && self.succeeded < self.completions {
            warnings.push(format!(
                "Job in progress: {}/{} completions",
                self.succeeded, self.completions
            ));
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.failed > 0 {
            errors.push(format!("{} pods failed", self.failed));
        }
        if self.manifest.has_condition_status("Failed", "True") {
            errors.push("Job has failed".to_string());
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
        let status = if self.manifest.has_condition_status("Complete", "True") {
            "Complete"
        } else if self.manifest.has_condition_status("Failed", "True") {
            "Failed"
        } else if self.active > 0 {
            "Running"
        } else {
            "Pending"
        };

        Some(format!(
            "Job {} - Status: {} ({}/{} completions)",
            ResourceV2::name(self),
            status,
            self.succeeded,
            self.completions
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("completions".to_string(), self.completions.to_string());
        fields.insert("succeeded".to_string(), self.succeeded.to_string());
        fields.insert("failed".to_string(), self.failed.to_string());
        fields.insert("active".to_string(), self.active.to_string());
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        workload_dependency_relationships(&self.manifest)
    }
}
