//! Health analyzers for OpenShift resources
//!
//! This module provides automated health analysis and issue detection
//! for various OpenShift resource types.

pub mod machine_analyzer;
pub mod networking_analyzer;
pub mod node_analyzer;
pub mod operator_analyzer;
pub mod pod_analyzer;
pub mod storage_analyzer;
pub mod types;
pub mod workload_analyzer;

#[cfg(test)]
mod tests;

pub use types::{HealthAnalysis, Issue, IssueCategory, IssueSeverity, Recommendation};

use crate::resources::ResourceV2;
use anyhow::Result;

/// Trait for resource health analyzers
pub trait HealthAnalyzer {
    /// Analyze a resource and return health analysis
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis>;

    /// Get the resource types this analyzer supports
    fn supported_kinds(&self) -> Vec<&'static str>;
}

/// Registry of health analyzers
pub struct AnalyzerRegistry {
    analyzers: Vec<Box<dyn HealthAnalyzer + Send + Sync>>,
}

impl AnalyzerRegistry {
    /// Create a new analyzer registry with default analyzers
    pub fn new() -> Self {
        let mut registry = Self {
            analyzers: Vec::new(),
        };

        // Register default analyzers
        registry.register(Box::new(node_analyzer::NodeAnalyzer::new()));
        registry.register(Box::new(pod_analyzer::PodAnalyzer::new()));
        registry.register(Box::new(operator_analyzer::OperatorAnalyzer::new()));
        registry.register(Box::new(machine_analyzer::MachineAnalyzer::new()));
        registry.register(Box::new(workload_analyzer::WorkloadAnalyzer::new()));
        registry.register(Box::new(storage_analyzer::StorageAnalyzer::new()));
        registry.register(Box::new(networking_analyzer::NetworkingAnalyzer::new()));

        registry
    }

    /// Register a new analyzer
    pub fn register(&mut self, analyzer: Box<dyn HealthAnalyzer + Send + Sync>) {
        self.analyzers.push(analyzer);
    }

    /// Analyze a resource using the appropriate analyzer
    pub fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let kind = resource.kind();

        for analyzer in &self.analyzers {
            if analyzer.supported_kinds().contains(&kind.as_ref()) {
                return analyzer.analyze(resource);
            }
        }

        // Return default analysis if no analyzer found
        Ok(HealthAnalysis::default())
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
