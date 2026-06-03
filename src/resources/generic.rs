// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GenericResource {
    manifest: Manifest,
    kind: String,
    namespace: Option<String>,
}

impl Resource for GenericResource {
    fn from(manifest: Manifest) -> GenericResource {
        let kind = manifest.as_yaml()["kind"]
            .as_str()
            .unwrap_or("Resource")
            .to_string();
        let namespace = manifest.namespace();

        GenericResource {
            manifest,
            kind,
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

impl ResourceV2 for GenericResource {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        &self.kind
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
        if has_condition_status(&self.manifest, &["Degraded", "Failing", "Failure"], "True") {
            HealthStatus::Error
        } else if has_condition_status(&self.manifest, &["Available", "Ready"], "False")
            || has_condition_status(&self.manifest, &["Progressing"], "True")
        {
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
        ResourceV2::conditions(self)
            .into_iter()
            .filter(|condition| {
                (matches!(condition.type_.as_str(), "Available" | "Ready")
                    && condition.status == "False")
                    || (condition.type_ == "Progressing" && condition.status == "True")
            })
            .map(format_condition_message)
            .collect()
    }

    fn errors(&self) -> Vec<String> {
        ResourceV2::conditions(self)
            .into_iter()
            .filter(|condition| {
                matches!(condition.type_.as_str(), "Degraded" | "Failing" | "Failure")
                    && condition.status == "True"
            })
            .map(format_condition_message)
            .collect()
    }

    fn metadata(&self) -> ResourceMetadata {
        ResourceMetadata {
            uid: self
                .manifest
                .uid()
                .unwrap_or_else(|| format!("{}__{}", self.kind.to_lowercase(), self.manifest.name)),
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        if let Some(api_version) = self.manifest.as_yaml()["apiVersion"].as_str() {
            fields.insert("api_version".to_string(), api_version.to_string());
        }
        fields
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        self.manifest.owner_references()
    }
}

fn has_condition_status(manifest: &Manifest, condition_types: &[&str], status: &str) -> bool {
    manifest.as_yaml()["status"]["conditions"]
        .as_vec()
        .into_iter()
        .flatten()
        .any(|condition| {
            condition["status"].as_str() == Some(status)
                && condition["type"]
                    .as_str()
                    .is_some_and(|type_| condition_types.contains(&type_))
        })
}

fn format_condition_message(condition: Condition) -> String {
    let mut message = format!("{} is {}", condition.type_, condition.status);
    if let Some(reason) = condition.reason {
        if !reason.is_empty() {
            message.push_str(&format!(" ({reason})"));
        }
    }
    if let Some(detail) = condition.message {
        if !detail.is_empty() {
            message.push_str(&format!(": {detail}"));
        }
    }
    message
}
