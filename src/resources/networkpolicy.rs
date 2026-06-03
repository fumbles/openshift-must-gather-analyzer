// Copyright (C) 2026 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NetworkPolicy {
    manifest: Manifest,
    namespace: Option<String>,
    policy_types: Vec<String>,
    ingress_rules: usize,
    egress_rules: usize,
    pod_selector: String,
}

impl Resource for NetworkPolicy {
    fn from(manifest: Manifest) -> NetworkPolicy {
        let namespace = manifest.namespace();
        let yaml = manifest.as_yaml();

        let policy_types = yaml["spec"]["policyTypes"]
            .as_vec()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let ingress_rules = yaml["spec"]["ingress"]
            .as_vec()
            .map(|items| items.len())
            .unwrap_or(0);
        let egress_rules = yaml["spec"]["egress"]
            .as_vec()
            .map(|items| items.len())
            .unwrap_or(0);

        let pod_selector =
            if let Some(selector) = yaml["spec"]["podSelector"]["matchLabels"].as_hash() {
                let mut labels = selector
                    .iter()
                    .filter_map(|(k, v)| Some((k.as_str()?, v.as_str()?)))
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>();
                labels.sort();
                if labels.is_empty() {
                    "all pods".to_string()
                } else {
                    labels.join(", ")
                }
            } else {
                "all pods".to_string()
            };

        NetworkPolicy {
            manifest,
            namespace,
            policy_types,
            ingress_rules,
            egress_rules,
            pod_selector,
        }
    }

    fn is_error(&self) -> bool {
        false
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

impl ResourceV2 for NetworkPolicy {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "NetworkPolicy"
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
        HealthStatus::Healthy
    }

    fn conditions(&self) -> Vec<Condition> {
        Vec::new()
    }

    fn warnings(&self) -> Vec<String> {
        Vec::new()
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
            "NetworkPolicy {} - Types: {} - Ingress rules: {} - Egress rules: {}",
            ResourceV2::name(self),
            if self.policy_types.is_empty() {
                "unspecified".to_string()
            } else {
                self.policy_types.join(", ")
            },
            self.ingress_rules,
            self.egress_rules
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert(
            "policy_types".to_string(),
            if self.policy_types.is_empty() {
                "unspecified".to_string()
            } else {
                self.policy_types.join(", ")
            },
        );
        fields.insert("ingress_rules".to_string(), self.ingress_rules.to_string());
        fields.insert("egress_rules".to_string(), self.egress_rules.to_string());
        fields.insert("pod_selector".to_string(), self.pod_selector.clone());
        fields
    }
}
