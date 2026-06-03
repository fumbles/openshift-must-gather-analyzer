// Copyright (C) 2024 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::prelude::*;
use crate::resources::{Condition, HealthStatus, Resource, ResourceMetadata, ResourceV2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PersistentVolume {
    manifest: Manifest,
    phase: String,
    claim_ref: Option<String>,
    storage_class: Option<String>,
    capacity: Option<String>,
    access_modes: Vec<String>,
    reclaim_policy: Option<String>,
}

impl Resource for PersistentVolume {
    fn from(manifest: Manifest) -> PersistentVolume {
        let yaml = manifest.as_yaml();

        let phase = yaml["status"]["phase"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let storage_class = yaml["spec"]["storageClassName"]
            .as_str()
            .map(|s| s.to_string());
        let reclaim_policy = yaml["spec"]["persistentVolumeReclaimPolicy"]
            .as_str()
            .map(|s| s.to_string());

        // Extract claim reference
        let claim_ref = if let Some(ns) = yaml["spec"]["claimRef"]["namespace"].as_str() {
            if let Some(name) = yaml["spec"]["claimRef"]["name"].as_str() {
                Some(format!("{}/{}", ns, name))
            } else {
                None
            }
        } else {
            None
        };

        // Extract capacity
        let capacity = yaml["spec"]["capacity"]["storage"]
            .as_str()
            .map(|s| s.to_string());

        // Extract access modes
        let mut access_modes = Vec::new();
        if let Some(modes) = yaml["spec"]["accessModes"].as_vec() {
            for mode in modes {
                if let Some(m) = mode.as_str() {
                    access_modes.push(m.to_string());
                }
            }
        }

        PersistentVolume {
            manifest,
            phase,
            claim_ref,
            storage_class,
            capacity,
            access_modes,
            reclaim_policy,
        }
    }

    fn is_error(&self) -> bool {
        self.phase == "Failed"
    }

    fn is_warning(&self) -> bool {
        self.phase == "Released"
    }

    fn name(&self) -> &String {
        &self.manifest.name
    }

    fn raw(&self) -> &String {
        self.manifest.as_raw()
    }
}

impl ResourceV2 for PersistentVolume {
    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn kind(&self) -> &str {
        "PersistentVolume"
    }

    fn namespace(&self) -> Option<&str> {
        None // PVs are cluster-scoped
    }

    fn uid(&self) -> &str {
        &self.manifest.name
    }

    fn raw(&self) -> &str {
        &self.manifest.raw
    }

    fn health_status(&self) -> HealthStatus {
        match self.phase.as_str() {
            "Available" | "Bound" => HealthStatus::Healthy,
            "Released" => HealthStatus::Warning,
            "Failed" => HealthStatus::Error,
            _ => HealthStatus::Unknown,
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
        if self.phase == "Released" {
            warnings
                .push("PV is released - claim was deleted but volume not reclaimed".to_string());
        }
        if self.phase == "Available" && self.claim_ref.is_some() {
            warnings.push("PV is available but has a claim reference".to_string());
        }
        warnings
    }

    fn errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.phase == "Failed" {
            errors.push("PV is in Failed state".to_string());
        }
        errors
    }

    fn metadata(&self) -> ResourceMetadata {
        ResourceMetadata {
            uid: self
                .manifest
                .uid()
                .unwrap_or_else(|| self.manifest.name.clone()),
            namespace: None,
            labels: self.manifest.labels(),
            annotations: self.manifest.annotations(),
            creation_timestamp: self.manifest.creation_timestamp(),
        }
    }

    fn summary(&self) -> Option<String> {
        Some(format!(
            "PV {} - Phase: {} - Capacity: {} - Claim: {}",
            ResourceV2::name(self),
            self.phase,
            self.capacity.as_deref().unwrap_or("N/A"),
            self.claim_ref.as_deref().unwrap_or("None")
        ))
    }

    fn key_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("phase".to_string(), self.phase.clone());
        if let Some(claim) = &self.claim_ref {
            fields.insert("claim_ref".to_string(), claim.clone());
        }
        if let Some(sc) = &self.storage_class {
            fields.insert("storage_class".to_string(), sc.clone());
        }
        if let Some(cap) = &self.capacity {
            fields.insert("capacity".to_string(), cap.clone());
        }
        if let Some(policy) = &self.reclaim_policy {
            fields.insert("reclaim_policy".to_string(), policy.clone());
        }
        fields.insert("access_modes".to_string(), self.access_modes.join(", "));
        fields
    }
}
