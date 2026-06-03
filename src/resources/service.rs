// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Service {
    manifest: Manifest,
    namespace: Option<String>,
    service_type: String,
    cluster_ip: Option<String>,
    ports: Vec<String>,
    selector: HashMap<String, String>,
}

impl Resource for Service {
    fn from(manifest: Manifest) -> Service {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let service_type = yaml["spec"]["type"]
            .as_str()
            .unwrap_or("ClusterIP")
            .to_string();
        let cluster_ip = yaml["spec"]["clusterIP"].as_str().map(|s| s.to_string());

        // Extract ports
        let mut ports = Vec::new();
        if let Some(port_list) = yaml["spec"]["ports"].as_vec() {
            for port in port_list {
                let port_num = port["port"].as_i64().unwrap_or(0);
                let protocol = port["protocol"].as_str().unwrap_or("TCP");
                let name = port["name"].as_str().unwrap_or("");

                if !name.is_empty() {
                    ports.push(format!("{}/{} ({})", port_num, protocol, name));
                } else {
                    ports.push(format!("{}/{}", port_num, protocol));
                }
            }
        }

        // Extract selector
        let mut selector = HashMap::new();
        if let Some(sel_hash) = yaml["spec"]["selector"].as_hash() {
            for (k, v) in sel_hash {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    selector.insert(key.to_string(), val.to_string());
                }
            }
        }

        Service {
            manifest,
            namespace,
            service_type,
            cluster_ip,
            ports,
            selector,
        }
    }

    fn is_error(&self) -> bool {
        false // Service itself doesn't have error states
    }

    fn is_warning(&self) -> bool {
        self.selector.is_empty() // Warning if no selector
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Service {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Service"
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
        if self.selector.is_empty() {
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
        if self.selector.is_empty() {
            warnings
                .push("Service has no selector - will not automatically route to pods".to_string());
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
        Some(format!(
            "Service {} - Type: {} - ClusterIP: {} - Ports: {}",
            ResourceV2::name(self),
            self.service_type,
            self.cluster_ip.as_deref().unwrap_or("None"),
            self.ports.len()
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("type".to_string(), self.service_type.clone());
        if let Some(ip) = &self.cluster_ip {
            fields.insert("cluster_ip".to_string(), ip.clone());
        }
        fields.insert("ports".to_string(), self.ports.join(", "));

        // Add selector as a single string
        let selector_str: Vec<String> = self
            .selector
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        fields.insert("selector".to_string(), selector_str.join(", "));

        fields
    }
}
