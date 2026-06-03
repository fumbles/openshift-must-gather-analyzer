// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{
    Condition, HealthStatus, RelationshipType, Resource, ResourceLink, ResourceMetadata, ResourceV2,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Event {
    manifest: Manifest,
    namespace: Option<String>,
    event_type: String,
    reason: String,
    message: String,
    count: u64,
    involved_kind: Option<String>,
    involved_name: Option<String>,
    involved_namespace: Option<String>,
    timestamp: Option<String>,
    reporting_component: Option<String>,
    display_name: String,
}

impl Resource for Event {
    fn from(manifest: Manifest) -> Event {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let event_type = yaml["type"].as_str().unwrap_or("Normal").to_string();
        let reason = yaml["reason"].as_str().unwrap_or("Unknown").to_string();
        let message = yaml["message"].as_str().unwrap_or("").to_string();
        let count = yaml["count"].as_i64().unwrap_or(1).max(1) as u64;
        let involved_kind = yaml["involvedObject"]["kind"]
            .as_str()
            .map(|s| s.to_string());
        let involved_name = yaml["involvedObject"]["name"]
            .as_str()
            .map(|s| s.to_string());
        let involved_namespace = yaml["involvedObject"]["namespace"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| namespace.clone());
        let timestamp = yaml["lastTimestamp"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| yaml["eventTime"].as_str().map(|s| s.to_string()))
            .or_else(|| yaml["firstTimestamp"].as_str().map(|s| s.to_string()))
            .or_else(|| manifest.creation_timestamp());
        let reporting_component = yaml["reportingComponent"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| yaml["source"]["component"].as_str().map(|s| s.to_string()));

        let display_name = match (involved_kind.as_deref(), involved_name.as_deref()) {
            (Some(kind), Some(name)) => format!("{} - {}/{}", reason, kind, name),
            _ => reason.clone(),
        };

        Event {
            manifest,
            namespace,
            event_type,
            reason,
            message,
            count,
            involved_kind,
            involved_name,
            involved_namespace,
            timestamp,
            reporting_component,
            display_name,
        }
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Event {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn kind(&self) -> &str {
        "Event"
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
        match self.event_type.as_str() {
            "Warning" => HealthStatus::Warning,
            "Normal" => HealthStatus::Healthy,
            _ => HealthStatus::Unknown,
        }
    }

    fn conditions(&self) -> Vec<Condition> {
        Vec::new()
    }

    fn warnings(&self) -> Vec<String> {
        if self.event_type == "Warning" && !self.message.is_empty() {
            vec![self.message.clone()]
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
            namespace: self.namespace.clone(),
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn relationships(&self) -> Vec<ResourceLink> {
        match (self.involved_kind.as_deref(), self.involved_name.as_deref()) {
            (Some(kind), Some(name)) => vec![ResourceLink {
                kind: kind.to_string(),
                name: name.to_string(),
                namespace: self.involved_namespace.clone(),
                relationship: RelationshipType::References,
            }],
            _ => Vec::new(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(self.message.clone())
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("type".to_string(), self.event_type.clone());
        fields.insert("reason".to_string(), self.reason.clone());
        fields.insert("count".to_string(), self.count.to_string());
        if !self.message.is_empty() {
            fields.insert("message".to_string(), self.message.clone());
        }
        if let Some(kind) = &self.involved_kind {
            fields.insert("involved_kind".to_string(), kind.clone());
        }
        if let Some(name) = &self.involved_name {
            fields.insert("involved_name".to_string(), name.clone());
        }
        if let Some(timestamp) = &self.timestamp {
            fields.insert("timestamp".to_string(), timestamp.clone());
        }
        if let Some(component) = &self.reporting_component {
            fields.insert("reporting_component".to_string(), component.clone());
        }
        fields
    }
}
