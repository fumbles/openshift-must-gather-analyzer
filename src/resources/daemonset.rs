// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, Resource, ResourceLink, ResourceMetadata, ResourceV2,
    workload_dependency_relationships,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DaemonSet {
    manifest: Manifest,
    namespace: Option<String>,
    desired_number_scheduled: i64,
    current_number_scheduled: i64,
    number_ready: i64,
    number_available: i64,
    number_unavailable: i64,
    number_misscheduled: i64,
}

impl Resource for DaemonSet {
    fn from(manifest: Manifest) -> DaemonSet {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let desired_number_scheduled = yaml["status"]["desiredNumberScheduled"]
            .as_i64()
            .unwrap_or(0);
        let current_number_scheduled = yaml["status"]["currentNumberScheduled"]
            .as_i64()
            .unwrap_or(0);
        let number_ready = yaml["status"]["numberReady"].as_i64().unwrap_or(0);
        let number_available = yaml["status"]["numberAvailable"].as_i64().unwrap_or(0);
        let number_unavailable = yaml["status"]["numberUnavailable"].as_i64().unwrap_or(0);
        let number_misscheduled = yaml["status"]["numberMisscheduled"].as_i64().unwrap_or(0);

        DaemonSet {
            manifest,
            namespace,
            desired_number_scheduled,
            current_number_scheduled,
            number_ready,
            number_available,
            number_unavailable,
            number_misscheduled,
        }
    }

    fn is_error(&self) -> bool {
        self.number_ready < self.desired_number_scheduled
            || self.number_unavailable > 0
            || self.number_misscheduled > 0
    }

    fn is_warning(&self) -> bool {
        self.current_number_scheduled < self.desired_number_scheduled
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for DaemonSet {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "DaemonSet"
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
        if self.number_ready < self.desired_number_scheduled
            || self.number_unavailable > 0
            || self.number_misscheduled > 0
        {
            HealthStatus::Error
        } else if self.current_number_scheduled < self.desired_number_scheduled {
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
        if self.current_number_scheduled < self.desired_number_scheduled {
            warnings.push(format!(
                "Only {}/{} pods scheduled",
                self.current_number_scheduled, self.desired_number_scheduled
            ));
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.number_ready < self.desired_number_scheduled {
            errors.push(format!(
                "Only {}/{} pods ready",
                self.number_ready, self.desired_number_scheduled
            ));
        }
        if self.number_unavailable > 0 {
            errors.push(format!("{} pods unavailable", self.number_unavailable));
        }
        if self.number_misscheduled > 0 {
            errors.push(format!("{} pods misscheduled", self.number_misscheduled));
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
            "DaemonSet {} - {}/{} pods ready",
            ResourceV2::name(self),
            self.number_ready,
            self.desired_number_scheduled
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "desired_number_scheduled".to_string(),
            self.desired_number_scheduled.to_string(),
        );
        fields.insert(
            "current_number_scheduled".to_string(),
            self.current_number_scheduled.to_string(),
        );
        fields.insert("number_ready".to_string(), self.number_ready.to_string());
        fields.insert(
            "number_available".to_string(),
            self.number_available.to_string(),
        );
        fields.insert(
            "number_unavailable".to_string(),
            self.number_unavailable.to_string(),
        );
        fields.insert(
            "number_misscheduled".to_string(),
            self.number_misscheduled.to_string(),
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
