// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{HashMap, HashSet};
use yaml_rust::Yaml;

pub mod baremetalhost;
pub mod certificatesigningrequest;
pub mod clusterautoscaler;
pub mod clusteroperator;
pub mod clusterrole;
pub mod clusterrolebinding;
pub mod configmap;
pub mod controlplanemachineset;
pub mod cronjob;
pub mod daemonset;
pub mod deployment;
pub mod endpoints;
pub mod event;
pub mod generic;
pub mod ingresscontroller;
pub mod job;
pub mod machine;
pub mod machineautoscaler;
pub mod machineconfig;
pub mod machineconfigpool;
pub mod machineconfiguration;
pub mod machinehealthcheck;
pub mod machineset;
pub mod namespace;
pub mod networkpolicy;
pub mod node;
pub mod pod;
pub mod pv;
pub mod pvc;
pub mod registry;
pub mod replicaset;
pub mod route;
pub mod secret;
pub mod securitycontextconstraint;
pub mod service;
pub mod statefulset;
pub mod storageclass;
pub mod volumeattachment;

use crate::Manifest;
pub use crate::resources::baremetalhost::BareMetalHost;
pub use crate::resources::certificatesigningrequest::CertificateSigningRequest;
pub use crate::resources::clusterautoscaler::ClusterAutoscaler;
pub use crate::resources::clusteroperator::ClusterOperator;
pub use crate::resources::clusterrole::ClusterRole;
pub use crate::resources::clusterrolebinding::ClusterRoleBinding;
pub use crate::resources::configmap::ConfigMap;
pub use crate::resources::controlplanemachineset::ControlPlaneMachineSet;
pub use crate::resources::cronjob::CronJob;
pub use crate::resources::daemonset::DaemonSet;
pub use crate::resources::deployment::Deployment;
pub use crate::resources::endpoints::Endpoints;
pub use crate::resources::event::Event;
pub use crate::resources::generic::GenericResource;
pub use crate::resources::ingresscontroller::IngressController;
pub use crate::resources::job::Job;
pub use crate::resources::machine::Machine;
pub use crate::resources::machineautoscaler::MachineAutoscaler;
pub use crate::resources::machineconfig::MachineConfig;
pub use crate::resources::machineconfigpool::MachineConfigPool;
pub use crate::resources::machineconfiguration::MachineConfiguration;
pub use crate::resources::machinehealthcheck::MachineHealthCheck;
pub use crate::resources::machineset::MachineSet;
pub use crate::resources::namespace::Namespace;
pub use crate::resources::networkpolicy::NetworkPolicy;
pub use crate::resources::node::Node;
pub use crate::resources::pod::Container;
pub use crate::resources::pod::Pod;
pub use crate::resources::pv::PersistentVolume;
pub use crate::resources::pvc::PersistentVolumeClaim;
pub use crate::resources::replicaset::ReplicaSet;
pub use crate::resources::route::Route;
pub use crate::resources::secret::Secret;
pub use crate::resources::securitycontextconstraint::SecurityContextConstraint;
pub use crate::resources::service::Service;
pub use crate::resources::statefulset::StatefulSet;
pub use crate::resources::storageclass::StorageClass;
pub use crate::resources::volumeattachment::VolumeAttachment;

/// Health status of a resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Error,
    Unknown,
}

/// A condition from a resource's status
#[derive(Debug, Clone)]
pub struct Condition {
    pub type_: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    #[allow(dead_code)]
    pub last_transition: Option<String>,
}

/// Metadata extracted from a resource
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    pub uid: String,
    pub namespace: Option<String>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub creation_timestamp: Option<String>,
}

/// Type of relationship between resources
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum RelationshipType {
    Owns,
    OwnedBy,
    References,
    ReferencedBy,
    Controls,
    ControlledBy,
}

/// A link to a related resource
#[derive(Debug, Clone)]
pub struct ResourceLink {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub relationship: RelationshipType,
}

fn push_resource_link(
    links: &mut Vec<ResourceLink>,
    seen: &mut HashSet<(String, String, Option<String>, RelationshipType)>,
    kind: &str,
    name: &str,
    namespace: Option<String>,
    relationship: RelationshipType,
) {
    if kind.is_empty() || name.is_empty() {
        return;
    }

    let key = (
        kind.to_string(),
        name.to_string(),
        namespace.clone(),
        relationship.clone(),
    );

    if seen.insert(key) {
        links.push(ResourceLink {
            kind: kind.to_string(),
            name: name.to_string(),
            namespace,
            relationship,
        });
    }
}

fn collect_container_reference_links(
    container: &Yaml,
    namespace: Option<&str>,
    links: &mut Vec<ResourceLink>,
    seen: &mut HashSet<(String, String, Option<String>, RelationshipType)>,
) {
    if let Some(env) = container["env"].as_vec() {
        for entry in env {
            if let Some(name) = entry["valueFrom"]["configMapKeyRef"]["name"].as_str() {
                push_resource_link(
                    links,
                    seen,
                    "ConfigMap",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }
            if let Some(name) = entry["valueFrom"]["secretKeyRef"]["name"].as_str() {
                push_resource_link(
                    links,
                    seen,
                    "Secret",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }
        }
    }

    if let Some(env_from) = container["envFrom"].as_vec() {
        for entry in env_from {
            if let Some(name) = entry["configMapRef"]["name"].as_str() {
                push_resource_link(
                    links,
                    seen,
                    "ConfigMap",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }
            if let Some(name) = entry["secretRef"]["name"].as_str() {
                push_resource_link(
                    links,
                    seen,
                    "Secret",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }
        }
    }
}

fn collect_pod_spec_reference_links(pod_spec: &Yaml, namespace: Option<&str>) -> Vec<ResourceLink> {
    let mut links = Vec::new();
    let mut seen = HashSet::new();

    if let Some(volumes) = pod_spec["volumes"].as_vec() {
        for volume in volumes {
            if let Some(name) = volume["persistentVolumeClaim"]["claimName"].as_str() {
                push_resource_link(
                    &mut links,
                    &mut seen,
                    "PersistentVolumeClaim",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }

            if let Some(name) = volume["configMap"]["name"].as_str() {
                push_resource_link(
                    &mut links,
                    &mut seen,
                    "ConfigMap",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }

            if let Some(name) = volume["secret"]["secretName"].as_str() {
                push_resource_link(
                    &mut links,
                    &mut seen,
                    "Secret",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }

            if let Some(sources) = volume["projected"]["sources"].as_vec() {
                for source in sources {
                    if let Some(name) = source["configMap"]["name"].as_str() {
                        push_resource_link(
                            &mut links,
                            &mut seen,
                            "ConfigMap",
                            name,
                            namespace.map(|ns| ns.to_string()),
                            RelationshipType::References,
                        );
                    }

                    if let Some(name) = source["secret"]["name"].as_str() {
                        push_resource_link(
                            &mut links,
                            &mut seen,
                            "Secret",
                            name,
                            namespace.map(|ns| ns.to_string()),
                            RelationshipType::References,
                        );
                    }
                }
            }
        }
    }

    if let Some(image_pull_secrets) = pod_spec["imagePullSecrets"].as_vec() {
        for secret in image_pull_secrets {
            if let Some(name) = secret["name"].as_str() {
                push_resource_link(
                    &mut links,
                    &mut seen,
                    "Secret",
                    name,
                    namespace.map(|ns| ns.to_string()),
                    RelationshipType::References,
                );
            }
        }
    }

    for container_set in ["containers", "initContainers", "ephemeralContainers"] {
        if let Some(containers) = pod_spec[container_set].as_vec() {
            for container in containers {
                collect_container_reference_links(container, namespace, &mut links, &mut seen);
            }
        }
    }

    links
}

pub fn workload_dependency_relationships(manifest: &Manifest) -> Vec<ResourceLink> {
    let yaml = manifest.as_yaml();
    let namespace = manifest.namespace();

    let pod_spec = match yaml["kind"].as_str() {
        Some("Pod") => Some(&yaml["spec"]),
        Some("Deployment") | Some("StatefulSet") | Some("DaemonSet") | Some("ReplicaSet") => {
            Some(&yaml["spec"]["template"]["spec"])
        }
        Some("Job") => Some(&yaml["spec"]["template"]["spec"]),
        Some("CronJob") => Some(&yaml["spec"]["jobTemplate"]["spec"]["template"]["spec"]),
        _ => None,
    };

    pod_spec
        .map(|spec| collect_pod_spec_reference_links(spec, namespace.as_deref()))
        .unwrap_or_default()
}

pub trait Resource: Send + Sync {
    fn from(manifest: Manifest) -> Self
    where
        Self: Sized;
    fn name(&self) -> &String;
    fn raw(&self) -> &String;

    fn is_error(&self) -> bool {
        false
    }

    fn is_warning(&self) -> bool {
        false
    }

    fn conditions(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Trait for all Kubernetes/OpenShift resources with enhanced structured data extraction
#[allow(dead_code)]
pub trait ResourceV2: Send + Sync {
    // Basic identification
    fn name(&self) -> &str;
    fn kind(&self) -> &str;
    fn namespace(&self) -> Option<&str>;
    fn uid(&self) -> &str;

    // Raw YAML access
    fn raw(&self) -> &str;

    // Health and status
    fn health_status(&self) -> HealthStatus;
    fn conditions(&self) -> Vec<Condition>;
    fn warnings(&self) -> Vec<String>;
    fn errors(&self) -> Vec<String>;

    // Structured metadata
    fn metadata(&self) -> ResourceMetadata;

    // Relationships (default implementation returns empty)
    fn relationships(&self) -> Vec<ResourceLink> {
        Vec::new()
    }

    // Analysis and summary
    fn summary(&self) -> Option<String> {
        None
    }

    fn key_fields(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn owner_references(&self) -> Vec<(String, String, Option<String>, Option<bool>)> {
        Vec::new()
    }

    // Optional log access for resources that carry local must-gather logs.
    fn logs(&self) -> Vec<(String, String, Option<String>)> {
        Vec::new()
    }

    // Legacy compatibility methods (deprecated but kept for migration)
    fn is_error(&self) -> bool {
        matches!(self.health_status(), HealthStatus::Error)
    }

    fn is_warning(&self) -> bool {
        matches!(self.health_status(), HealthStatus::Warning)
    }
}
