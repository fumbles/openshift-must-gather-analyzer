// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Route {
    manifest: Manifest,
    namespace: Option<String>,
    host: Option<String>,
    path: Option<String>,
    tls_termination: Option<String>,
    service_name: Option<String>,
    service_port: Option<String>,
    admitted: bool,
}

impl Resource for Route {
    fn from(manifest: Manifest) -> Route {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let host = yaml["spec"]["host"].as_str().map(|s| s.to_string());
        let path = yaml["spec"]["path"].as_str().map(|s| s.to_string());
        let tls_termination = yaml["spec"]["tls"]["termination"]
            .as_str()
            .map(|s| s.to_string());
        let service_name = yaml["spec"]["to"]["name"].as_str().map(|s| s.to_string());

        // Service port can be string or integer
        let service_port = if let Some(port_str) = yaml["spec"]["port"]["targetPort"].as_str() {
            Some(port_str.to_string())
        } else if let Some(port_int) = yaml["spec"]["port"]["targetPort"].as_i64() {
            Some(port_int.to_string())
        } else {
            None
        };

        // Check if route is admitted by looking at ingress status
        let mut admitted = false;
        if let Some(ingress) = yaml["status"]["ingress"].as_vec() {
            for ing in ingress {
                if let Some(conditions) = ing["conditions"].as_vec() {
                    for cond in conditions {
                        if cond["type"].as_str() == Some("Admitted")
                            && cond["status"].as_str() == Some("True")
                        {
                            admitted = true;
                            break;
                        }
                    }
                }
                if admitted {
                    break;
                }
            }
        }

        Route {
            manifest,
            namespace,
            host,
            path,
            tls_termination,
            service_name,
            service_port,
            admitted,
        }
    }

    fn is_error(&self) -> bool {
        !self.admitted
    }

    fn is_warning(&self) -> bool {
        false
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for Route {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "Route"
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
        if self.admitted {
            HealthStatus::Healthy
        } else {
            HealthStatus::Error
        }
    }

    fn conditions(&self) -> Vec<Condition> {
        let mut conditions = Vec::new();
        let yaml = self.manifest.as_yaml();

        // Extract conditions from ingress status
        if let Some(ingress) = yaml["status"]["ingress"].as_vec() {
            for ing in ingress {
                if let Some(conds) = ing["conditions"].as_vec() {
                    for cond in conds {
                        let type_ = cond["type"].as_str().unwrap_or("").to_string();
                        let status = cond["status"].as_str().unwrap_or("").to_string();
                        let reason = cond["reason"].as_str().map(|s| s.to_string());
                        let message = cond["message"].as_str().map(|s| s.to_string());
                        conditions.push(Condition {
                            type_,
                            status,
                            reason,
                            message,
                            last_transition: None,
                        });
                    }
                }
            }
        }

        conditions
    }

    fn warnings(&self) -> Vec<String> {
        Vec::new()
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !self.admitted {
            errors.push("Route is not admitted - traffic cannot reach this route".to_string());

            // Try to get the reason from conditions
            for cond in ResourceV2::conditions(self) {
                if cond.type_ == "Admitted" && cond.status == "False" {
                    if let Some(msg) = cond.message {
                        errors.push(format!("Admission failed: {}", msg));
                    }
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
        Some(format!(
            "Route {} - Host: {} - Service: {} - Admitted: {}",
            ResourceV2::name(self),
            self.host.as_deref().unwrap_or("N/A"),
            self.service_name.as_deref().unwrap_or("N/A"),
            self.admitted
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        if let Some(host) = &self.host {
            fields.insert("host".to_string(), host.clone());
        }
        if let Some(path) = &self.path {
            fields.insert("path".to_string(), path.clone());
        }
        if let Some(tls) = &self.tls_termination {
            fields.insert("tls_termination".to_string(), tls.clone());
        }
        if let Some(svc) = &self.service_name {
            fields.insert("service_name".to_string(), svc.clone());
        }
        if let Some(port) = &self.service_port {
            fields.insert("service_port".to_string(), port.clone());
        }
        fields.insert("admitted".to_string(), self.admitted.to_string());
        fields
    }
}
