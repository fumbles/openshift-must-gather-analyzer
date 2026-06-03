// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(dead_code)]

use crate::manifest::Manifest;
use crate::resources::Resource;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Describes how to load and analyze a resource type.
///
/// A ResourceDescriptor contains all the metadata and functions needed to work with
/// a specific Kubernetes/OpenShift resource type. It includes:
/// - Identification (kind, group, namespace scope)
/// - Display information
/// - Path building logic for locating resources in must-gather archives
/// - Parsing logic for converting manifests into typed Resource objects
pub struct ResourceDescriptor {
    /// The Kubernetes kind (e.g., "Node", "Machine", "Pod")
    pub kind: String,

    /// The API group (e.g., "core", "machine.openshift.io")
    pub group: String,

    /// Whether this resource is namespace-scoped (true) or cluster-scoped (false)
    pub namespace_scoped: bool,

    /// Human-readable display name for UI purposes
    pub display_name: String,

    /// Function that builds the path to resources of this type in a must-gather archive.
    /// Takes the must-gather root path and returns the full path to the resource directory.
    pub path_builder: Box<dyn Fn(&str) -> String + Send + Sync>,

    /// Function that parses a Manifest into a concrete Resource implementation.
    /// Returns a boxed trait object for polymorphic handling.
    pub parser: Box<dyn Fn(Manifest) -> Result<Box<dyn Resource>> + Send + Sync>,
}

/// Central registry for all resource types.
///
/// The ResourceRegistry maintains a collection of ResourceDescriptors, indexed by
/// their group/kind combination. This allows the system to dynamically discover
/// and work with different resource types without hard-coding logic for each one.
///
/// # Example
/// ```no_run
/// use camgi::resources::registry::{ResourceRegistry, RegistryBuilder};
///
/// let registry = RegistryBuilder::new()
///     .with_nodes()
///     .with_machines()
///     .build();
///
/// if let Some(descriptor) = registry.get("core", "Node") {
///     let path = (descriptor.path_builder)("/path/to/must-gather");
///     println!("Nodes are located at: {}", path);
/// }
/// ```
pub struct ResourceRegistry {
    descriptors: HashMap<String, Arc<ResourceDescriptor>>,
}

impl ResourceRegistry {
    /// Creates a new empty registry.
    ///
    /// Typically you'll want to use `RegistryBuilder` instead to create a
    /// pre-populated registry with all known resource types.
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
        }
    }

    /// Registers a new resource descriptor.
    ///
    /// The descriptor is indexed by "group/kind" for efficient lookup.
    /// If a descriptor with the same group/kind already exists, it will be replaced.
    pub fn register(&mut self, descriptor: ResourceDescriptor) {
        let key = format!("{}/{}", descriptor.group, descriptor.kind);
        self.descriptors.insert(key, Arc::new(descriptor));
    }

    /// Retrieves a resource descriptor by group and kind.
    ///
    /// Returns `None` if no descriptor is registered for the given group/kind combination.
    ///
    /// # Arguments
    /// * `group` - The API group (e.g., "core", "machine.openshift.io")
    /// * `kind` - The resource kind (e.g., "Node", "Machine")
    pub fn get(&self, group: &str, kind: &str) -> Option<Arc<ResourceDescriptor>> {
        let key = format!("{}/{}", group, kind);
        self.descriptors.get(&key).cloned()
    }

    /// Returns all registered resource descriptors.
    ///
    /// Useful for iterating over all known resource types, for example when
    /// building a UI or scanning a must-gather archive.
    pub fn all_descriptors(&self) -> Vec<Arc<ResourceDescriptor>> {
        self.descriptors.values().cloned().collect()
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating a registry with all known resource types.
///
/// The RegistryBuilder provides a fluent API for constructing a ResourceRegistry
/// with the desired resource types. Each `with_*` method adds support for a
/// specific resource type.
///
/// # Example
/// ```no_run
/// use camgi::resources::registry::RegistryBuilder;
///
/// let registry = RegistryBuilder::new()
///     .with_nodes()
///     .with_machines()
///     .with_pods()
///     .build();
/// ```
pub struct RegistryBuilder {
    registry: ResourceRegistry,
}

impl RegistryBuilder {
    /// Creates a new builder with an empty registry.
    pub fn new() -> Self {
        Self {
            registry: ResourceRegistry::new(),
        }
    }

    /// Consumes the builder and returns the constructed registry.
    pub fn build(self) -> ResourceRegistry {
        self.registry
    }

    /// Registers support for Node resources.
    ///
    /// Nodes are cluster-scoped resources in the "core" API group.
    /// This method will be fully implemented when migrating the Node resource.
    pub fn with_nodes(self) -> Self {
        // TODO: Implement when migrating Node resource
        // Example implementation:
        // let descriptor = ResourceDescriptor {
        //     kind: "Node".to_string(),
        //     group: "core".to_string(),
        //     namespace_scoped: false,
        //     display_name: "Nodes".to_string(),
        //     path_builder: Box::new(|root| format!("{}/cluster-scoped-resources/core/nodes", root)),
        //     parser: Box::new(|manifest| Ok(Box::new(Node::from(manifest)))),
        // };
        // self.registry.register(descriptor);
        self
    }

    /// Registers support for Machine resources.
    ///
    /// Machines are namespace-scoped resources in the "machine.openshift.io" API group.
    /// This method will be fully implemented when migrating the Machine resource.
    pub fn with_machines(self) -> Self {
        // TODO: Implement when migrating Machine resource
        self
    }

    /// Registers support for Pod resources.
    ///
    /// Pods are namespace-scoped resources in the "core" API group.
    /// This method will be fully implemented when migrating the Pod resource.
    pub fn with_pods(self) -> Self {
        // TODO: Implement when migrating Pod resource
        self
    }

    /// Registers support for MachineSet resources.
    ///
    /// MachineSets are namespace-scoped resources in the "machine.openshift.io" API group.
    /// This method will be fully implemented when migrating the MachineSet resource.
    pub fn with_machinesets(self) -> Self {
        // TODO: Implement when migrating MachineSet resource
        self
    }

    /// Registers support for ClusterOperator resources.
    ///
    /// ClusterOperators are cluster-scoped resources in the "config.openshift.io" API group.
    /// This method will be fully implemented when migrating the ClusterOperator resource.
    pub fn with_clusteroperators(self) -> Self {
        // TODO: Implement when migrating ClusterOperator resource
        self
    }

    /// Registers support for MachineConfigPool resources.
    ///
    /// MachineConfigPools are cluster-scoped resources in the "machineconfiguration.openshift.io" API group.
    /// This method will be fully implemented when migrating the MachineConfigPool resource.
    pub fn with_machineconfigpools(self) -> Self {
        // TODO: Implement when migrating MachineConfigPool resource
        self
    }
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock resource for testing
    struct MockResource {
        name: String,
        raw: String,
    }

    impl Resource for MockResource {
        fn from(manifest: Manifest) -> Self {
            MockResource {
                name: manifest.name.clone(),
                raw: manifest.as_raw().clone(),
            }
        }

        fn name(&self) -> &String {
            &self.name
        }

        fn raw(&self) -> &String {
            &self.raw
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = ResourceRegistry::new();
        assert_eq!(registry.all_descriptors().len(), 0);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = ResourceRegistry::new();

        let descriptor = ResourceDescriptor {
            kind: "TestResource".to_string(),
            group: "test.example.com".to_string(),
            namespace_scoped: true,
            display_name: "Test Resources".to_string(),
            path_builder: Box::new(|root| format!("{}/test/path", root)),
            parser: Box::new(|manifest| Ok(Box::new(<MockResource as Resource>::from(manifest)))),
        };

        registry.register(descriptor);

        // Should be able to retrieve the descriptor
        let retrieved = registry.get("test.example.com", "TestResource");
        assert!(retrieved.is_some());

        let desc = retrieved.unwrap();
        assert_eq!(desc.kind, "TestResource");
        assert_eq!(desc.group, "test.example.com");
        assert_eq!(desc.namespace_scoped, true);
        assert_eq!(desc.display_name, "Test Resources");
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = ResourceRegistry::new();
        let result = registry.get("nonexistent", "Resource");
        assert!(result.is_none());
    }

    #[test]
    fn test_registry_all_descriptors() {
        let mut registry = ResourceRegistry::new();

        let descriptor1 = ResourceDescriptor {
            kind: "Resource1".to_string(),
            group: "group1".to_string(),
            namespace_scoped: false,
            display_name: "Resource 1".to_string(),
            path_builder: Box::new(|root| format!("{}/path1", root)),
            parser: Box::new(|manifest| Ok(Box::new(<MockResource as Resource>::from(manifest)))),
        };

        let descriptor2 = ResourceDescriptor {
            kind: "Resource2".to_string(),
            group: "group2".to_string(),
            namespace_scoped: true,
            display_name: "Resource 2".to_string(),
            path_builder: Box::new(|root| format!("{}/path2", root)),
            parser: Box::new(|manifest| Ok(Box::new(<MockResource as Resource>::from(manifest)))),
        };

        registry.register(descriptor1);
        registry.register(descriptor2);

        let all = registry.all_descriptors();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_registry_replace_descriptor() {
        let mut registry = ResourceRegistry::new();

        let descriptor1 = ResourceDescriptor {
            kind: "TestResource".to_string(),
            group: "test.example.com".to_string(),
            namespace_scoped: true,
            display_name: "First Version".to_string(),
            path_builder: Box::new(|root| format!("{}/path1", root)),
            parser: Box::new(|manifest| Ok(Box::new(<MockResource as Resource>::from(manifest)))),
        };

        let descriptor2 = ResourceDescriptor {
            kind: "TestResource".to_string(),
            group: "test.example.com".to_string(),
            namespace_scoped: false,
            display_name: "Second Version".to_string(),
            path_builder: Box::new(|root| format!("{}/path2", root)),
            parser: Box::new(|manifest| Ok(Box::new(<MockResource as Resource>::from(manifest)))),
        };

        registry.register(descriptor1);
        registry.register(descriptor2);

        // Should only have one descriptor (the second one)
        assert_eq!(registry.all_descriptors().len(), 1);

        let desc = registry.get("test.example.com", "TestResource").unwrap();
        assert_eq!(desc.display_name, "Second Version");
        assert_eq!(desc.namespace_scoped, false);
    }

    #[test]
    fn test_path_builder_function() {
        let mut registry = ResourceRegistry::new();

        let descriptor = ResourceDescriptor {
            kind: "Node".to_string(),
            group: "core".to_string(),
            namespace_scoped: false,
            display_name: "Nodes".to_string(),
            path_builder: Box::new(|root| format!("{}/cluster-scoped-resources/core/nodes", root)),
            parser: Box::new(|manifest| Ok(Box::new(<MockResource as Resource>::from(manifest)))),
        };

        registry.register(descriptor);

        let desc = registry.get("core", "Node").unwrap();
        let path = (desc.path_builder)("/must-gather");
        assert_eq!(path, "/must-gather/cluster-scoped-resources/core/nodes");
    }

    #[test]
    fn test_registry_builder_new() {
        let builder = RegistryBuilder::new();
        let registry = builder.build();
        assert_eq!(registry.all_descriptors().len(), 0);
    }

    #[test]
    fn test_registry_builder_chaining() {
        // Test that builder methods can be chained
        let registry = RegistryBuilder::new()
            .with_nodes()
            .with_machines()
            .with_pods()
            .build();

        // Currently these methods don't add anything, so registry should be empty
        // This will change when we implement the actual resource migrations
        assert_eq!(registry.all_descriptors().len(), 0);
    }

    #[test]
    fn test_registry_builder_default() {
        let builder = RegistryBuilder::default();
        let registry = builder.build();
        assert_eq!(registry.all_descriptors().len(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = ResourceRegistry::default();
        assert_eq!(registry.all_descriptors().len(), 0);
    }

    #[test]
    fn test_arc_sharing() {
        let mut registry = ResourceRegistry::new();

        let descriptor = ResourceDescriptor {
            kind: "SharedResource".to_string(),
            group: "test".to_string(),
            namespace_scoped: false,
            display_name: "Shared".to_string(),
            path_builder: Box::new(|root| format!("{}/shared", root)),
            parser: Box::new(|manifest| Ok(Box::new(<MockResource as Resource>::from(manifest)))),
        };

        registry.register(descriptor);

        // Get the descriptor multiple times
        let desc1 = registry.get("test", "SharedResource").unwrap();
        let desc2 = registry.get("test", "SharedResource").unwrap();

        // Both should point to the same Arc
        assert!(Arc::ptr_eq(&desc1, &desc2));
    }
}
