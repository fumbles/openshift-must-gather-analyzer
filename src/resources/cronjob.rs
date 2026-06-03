// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, Resource, ResourceLink, ResourceMetadata, ResourceV2,
    workload_dependency_relationships,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CronJob {
    manifest: Manifest,
    namespace: Option<String>,
    schedule: String,
    suspend: bool,
    active_jobs: i64,
    last_schedule_time: Option<String>,
}

impl Resource for CronJob {
    fn from(manifest: Manifest) -> CronJob {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let schedule = yaml["spec"]["schedule"].as_str().unwrap_or("").to_string();
        let suspend = yaml["spec"]["suspend"].as_bool().unwrap_or(false);
        let active_jobs = yaml["status"]["active"]
            .as_vec()
            .map(|v| v.len() as i64)
            .unwrap_or(0);
        let last_schedule_time = yaml["status"]["lastScheduleTime"]
            .as_str()
            .map(|s| s.to_string());

        CronJob {
            manifest,
            namespace,
            schedule,
            suspend,
            active_jobs,
            last_schedule_time,
        }
    }

    fn is_error(&self) -> bool {
        false // CronJobs don't typically have error states
    }

    fn is_warning(&self) -> bool {
        self.suspend
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for CronJob {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "CronJob"
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
        if self.suspend {
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
        if self.suspend {
            warnings.push("CronJob is suspended".to_string());
        }
        warnings
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
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        let status = if self.suspend {
            "Suspended"
        } else if self.active_jobs > 0 {
            "Active"
        } else {
            "Scheduled"
        };

        Some(format!(
            "CronJob {} - Status: {}, Schedule: {}",
            ResourceV2::name(self),
            status,
            self.schedule
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("schedule".to_string(), self.schedule.clone());
        fields.insert("suspend".to_string(), self.suspend.to_string());
        fields.insert("active_jobs".to_string(), self.active_jobs.to_string());
        if let Some(ref last_schedule) = self.last_schedule_time {
            fields.insert("last_schedule_time".to_string(), last_schedule.clone());
        }
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        workload_dependency_relationships(&self.manifest)
    }
}
