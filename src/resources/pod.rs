// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, Resource, ResourceLink, ResourceMetadata, ResourceV2,
    workload_dependency_relationships,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Pod {
    manifest: Manifest,
    pub containers: Vec<Container>,
    namespace: Option<String>,
}

impl Pod {
    pub fn new() -> Pod {
        Pod {
            manifest: Manifest::new(),
            containers: Vec::new(),
            namespace: None,
        }
    }

    pub fn push_container(&mut self, container: Container) {
        self.containers.push(container);
    }
}

impl Resource for Pod {
    fn from(manifest: Manifest) -> Pod {
        let containers = Vec::new();
        let namespace = manifest.namespace();
        Pod {
            manifest,
            containers,
            namespace,
        }
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

/// Holds the name and raw log files of a container within a pod.
#[derive(Debug, Clone)]
pub struct Container {
    pub name: String,
    pub current_log: String,
    pub current_log_path: Option<String>,
}

impl ResourceV2 for Pod {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Pod"
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
        let yaml = self.manifest.as_yaml();

        if let Some(phase) = yaml["status"]["phase"].as_str() {
            if phase == "Succeeded" {
                return HealthStatus::Healthy;
            }
        }

        // Check container statuses for critical errors
        let container_statuses = yaml["status"]["containerStatuses"].as_vec();
        let init_container_statuses = yaml["status"]["initContainerStatuses"].as_vec();

        // Check for ImagePullBackOff, ErrImagePull, CrashLoopBackOff
        for statuses in [container_statuses, init_container_statuses]
            .iter()
            .flatten()
        {
            for status in statuses.iter() {
                // Check waiting state
                if let Some(reason) = status["state"]["waiting"]["reason"].as_str() {
                    match reason {
                        "ImagePullBackOff"
                        | "ErrImagePull"
                        | "CrashLoopBackOff"
                        | "CreateContainerConfigError"
                        | "InvalidImageName" => {
                            return HealthStatus::Error;
                        }
                        _ => {}
                    }
                }

                // Check terminated state with non-zero exit code
                if let Some(exit_code) = status["state"]["terminated"]["exitCode"].as_i64() {
                    if exit_code != 0 {
                        return HealthStatus::Error;
                    }
                }
            }
        }

        // Check pod phase
        if let Some(phase) = yaml["status"]["phase"].as_str() {
            match phase {
                "Failed" | "Unknown" => return HealthStatus::Error,
                "Pending" => {
                    // Pending can be normal during startup, check conditions
                    if self.manifest.has_condition_status("PodScheduled", "False") {
                        return HealthStatus::Error;
                    }
                    return HealthStatus::Warning;
                }
                _ => {}
            }
        }

        // Check if pod is ready
        if self.manifest.has_condition_status("Ready", "True") {
            HealthStatus::Healthy
        } else if self.manifest.has_condition_status("Ready", "False") {
            HealthStatus::Error
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
        let yaml = self.manifest.as_yaml();

        if yaml["status"]["phase"].as_str() == Some("Succeeded") {
            return warnings;
        }

        // Check for container restarts
        if let Some(statuses) = yaml["status"]["containerStatuses"].as_vec() {
            for status in statuses {
                if let Some(restart_count) = status["restartCount"].as_i64() {
                    if restart_count > 0 {
                        let name = status["name"].as_str().unwrap_or("unknown");
                        warnings.push(format!(
                            "Container '{}' has restarted {} times",
                            name, restart_count
                        ));
                    }
                }
            }
        }

        // Check for container issues
        for cond in ResourceV2::conditions(self) {
            if cond.type_ == "ContainersReady" && cond.status == "False" {
                warnings.push(format!(
                    "Containers not ready: {}",
                    cond.message.as_deref().unwrap_or("Unknown reason")
                ));
            }
        }

        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let yaml = self.manifest.as_yaml();

        if yaml["status"]["phase"].as_str() == Some("Succeeded") {
            return errors;
        }

        // Check container statuses for errors
        let container_statuses = yaml["status"]["containerStatuses"].as_vec();
        let init_container_statuses = yaml["status"]["initContainerStatuses"].as_vec();

        for (is_init, statuses) in [(false, container_statuses), (true, init_container_statuses)] {
            if let Some(statuses) = statuses {
                for status in statuses {
                    let name = status["name"].as_str().unwrap_or("unknown");
                    let prefix = if is_init {
                        "Init container"
                    } else {
                        "Container"
                    };

                    // Check waiting state
                    if let Some(reason) = status["state"]["waiting"]["reason"].as_str() {
                        let message = status["state"]["waiting"]["message"].as_str().unwrap_or("");
                        match reason {
                            "ImagePullBackOff" | "ErrImagePull" => {
                                errors.push(format!(
                                    "{} '{}': {} - {}",
                                    prefix, name, reason, message
                                ));
                            }
                            "CrashLoopBackOff" => {
                                errors.push(format!(
                                    "{} '{}': CrashLoopBackOff - container is crashing repeatedly",
                                    prefix, name
                                ));
                            }
                            "CreateContainerConfigError" | "InvalidImageName" => {
                                errors.push(format!(
                                    "{} '{}': {} - {}",
                                    prefix, name, reason, message
                                ));
                            }
                            _ => {}
                        }
                    }

                    // Check terminated state
                    if let Some(reason) = status["state"]["terminated"]["reason"].as_str() {
                        if reason == "Error" || reason == "OOMKilled" {
                            let exit_code = status["state"]["terminated"]["exitCode"]
                                .as_i64()
                                .unwrap_or(0);
                            let message = status["state"]["terminated"]["message"]
                                .as_str()
                                .unwrap_or("");
                            errors.push(format!(
                                "{} '{}': {} (exit code: {}) - {}",
                                prefix, name, reason, exit_code, message
                            ));
                        }
                    }
                }
            }
        }

        // Check pod phase
        if let Some(phase) = yaml["status"]["phase"].as_str() {
            if phase == "Failed" {
                if let Some(reason) = yaml["status"]["reason"].as_str() {
                    errors.push(format!("Pod failed: {}", reason));
                } else {
                    errors.push("Pod is in Failed state".to_string());
                }
            }
        }

        // Check for scheduling issues
        if self.manifest.has_condition_status("PodScheduled", "False") {
            for cond in ResourceV2::conditions(self) {
                if cond.type_ == "PodScheduled" && cond.status == "False" {
                    if let Some(msg) = &cond.message {
                        errors.push(format!("Pod scheduling failed: {}", msg));
                    }
                }
            }
        }

        // Check for other failed conditions
        for cond in ResourceV2::conditions(self) {
            if cond.status == "False"
                && cond.type_ != "ContainersReady"
                && cond.type_ != "PodScheduled"
            {
                if let Some(msg) = &cond.message {
                    errors.push(format!("{}: {}", cond.type_, msg));
                }
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
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        let status = if self.manifest.has_condition_status("Ready", "True") {
            "Ready"
        } else {
            "Not Ready"
        };
        Some(format!(
            "Pod {} - Status: {} - Containers: {}",
            ResourceV2::name(self),
            status,
            self.containers.len()
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        let yaml = self.manifest.as_yaml();
        let restart_count = yaml["status"]["containerStatuses"]
            .as_vec()
            .map(|statuses| {
                statuses
                    .iter()
                    .filter_map(|status| status["restartCount"].as_i64())
                    .sum::<i64>()
            })
            .unwrap_or(0);

        if let Some(phase) = yaml["status"]["phase"].as_str() {
            fields.insert("phase".to_string(), phase.to_string());
        }
        fields.insert(
            "ready".to_string(),
            self.manifest
                .has_condition_status("Ready", "True")
                .to_string(),
        );
        fields.insert("containers".to_string(), self.containers.len().to_string());
        fields.insert("restart_count".to_string(), restart_count.to_string());
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        workload_dependency_relationships(&self.manifest)
    }

    fn logs(&self) -> Vec<(String, String, Option<String>)> {
        self.containers
            .iter()
            .filter(|container| !container.current_log.trim().is_empty())
            .map(|container| {
                (
                    container.name.clone(),
                    container.current_log.clone(),
                    container.current_log_path.clone(),
                )
            })
            .collect()
    }
}
