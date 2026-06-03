// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Endpoints {
    manifest: Manifest,
    namespace: Option<String>,
    ready_addresses: usize,
    not_ready_addresses: usize,
    ports: Vec<String>,
}

impl Resource for Endpoints {
    fn from(manifest: Manifest) -> Endpoints {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let mut ready_addresses = 0;
        let mut not_ready_addresses = 0;
        let mut ports = Vec::new();

        // Count addresses from subsets
        if let Some(subsets) = yaml["subsets"].as_vec() {
            for subset in subsets {
                // Count ready addresses
                if let Some(addrs) = subset["addresses"].as_vec() {
                    ready_addresses += addrs.len();
                }

                // Count not ready addresses
                if let Some(not_ready) = subset["notReadyAddresses"].as_vec() {
                    not_ready_addresses += not_ready.len();
                }

                // Extract ports
                if let Some(port_list) = subset["ports"].as_vec() {
                    for port in port_list {
                        let port_num = port["port"].as_i64().unwrap_or(0);
                        let protocol = port["protocol"].as_str().unwrap_or("TCP");
                        let name = port["name"].as_str().unwrap_or("");

                        let port_str = if !name.is_empty() {
                            format!("{}/{} ({})", port_num, protocol, name)
                        } else {
                            format!("{}/{}", port_num, protocol)
                        };

                        if !ports.contains(&port_str) {
                            ports.push(port_str);
                        }
                    }
                }
            }
        }

        Endpoints {
            manifest,
            namespace,
            ready_addresses,
            not_ready_addresses,
            ports,
        }
    }

    fn is_error(&self) -> bool {
        false
    }

    fn is_warning(&self) -> bool {
        self.ready_addresses == 0
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Endpoints {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Endpoints"
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
        if self.ready_addresses > 0 {
            HealthStatus::Healthy
        } else if self.not_ready_addresses > 0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Warning
        }
    }

    fn conditions(&self) -> Vec<Condition> {
        // Endpoints don't typically have conditions
        Vec::new()
    }

    fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.ready_addresses == 0 {
            if self.not_ready_addresses > 0 {
                warnings.push(format!(
                    "No ready endpoints - {} not ready addresses",
                    self.not_ready_addresses
                ));
            } else {
                warnings.push("No endpoints available - service has no backing pods".to_string());
            }
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
            "Endpoints {} - Ready: {} - Not Ready: {}",
            ResourceV2::name(self),
            self.ready_addresses,
            self.not_ready_addresses
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "ready_addresses".to_string(),
            self.ready_addresses.to_string(),
        );
        fields.insert(
            "not_ready_addresses".to_string(),
            self.not_ready_addresses.to_string(),
        );
        fields.insert("ports".to_string(), self.ports.join(", "));
        fields
    }
}
