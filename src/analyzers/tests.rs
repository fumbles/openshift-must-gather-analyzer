//! Tests for health analyzers

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::manifest::Manifest;
    use crate::resources::{Container, Node, Pod, Resource, ResourceV2};

    #[test]
    fn test_analyzer_registry_creation() {
        let registry = AnalyzerRegistry::new();
        // Registry should have analyzers registered
        assert!(registry.analyzers.len() > 0);
    }

    #[test]
    fn test_node_analyzer_healthy_node() {
        use std::path::PathBuf;

        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let node = <Node as Resource>::from(manifest);
        let analyzer = node_analyzer::NodeAnalyzer::new();

        let analysis = analyzer.analyze(&node as &dyn ResourceV2).unwrap();

        // Healthy node should have high score
        assert!(analysis.health_score >= 80);
        // The test node is Ready, so it should be healthy
        assert!(analysis.is_healthy() || analysis.health_score >= 70);
    }

    #[test]
    fn test_health_analysis_severity_counting() {
        let mut analysis = HealthAnalysis::new(80, "Test analysis");

        analysis.add_issue(Issue::new(
            IssueSeverity::Warning,
            IssueCategory::Performance,
            "Test Warning",
            "This is a test warning",
        ));

        analysis.add_issue(Issue::new(
            IssueSeverity::Error,
            IssueCategory::Availability,
            "Test Error",
            "This is a test error",
        ));

        assert_eq!(analysis.count_by_severity(IssueSeverity::Warning), 1);
        assert_eq!(analysis.count_by_severity(IssueSeverity::Error), 1);
        assert_eq!(analysis.count_by_severity(IssueSeverity::Critical), 0);

        assert_eq!(analysis.max_severity(), Some(IssueSeverity::Error));
        assert!(!analysis.is_healthy());
    }

    #[test]
    fn test_issue_builder() {
        let issue = Issue::new(
            IssueSeverity::Warning,
            IssueCategory::Configuration,
            "Test Issue",
            "Test Description",
        )
        .with_component("test-component")
        .with_detected_at("2024-01-01T00:00:00Z");

        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.category, IssueCategory::Configuration);
        assert_eq!(issue.title, "Test Issue");
        assert_eq!(issue.affected_component, Some("test-component".to_string()));
        assert_eq!(issue.detected_at, Some("2024-01-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_recommendation_builder() {
        let rec = Recommendation::new("Fix the issue", "Run this command to fix")
            .with_action("oc fix --all")
            .with_docs("https://docs.example.com");

        assert_eq!(rec.title, "Fix the issue");
        assert_eq!(rec.action, Some("oc fix --all".to_string()));
        assert_eq!(
            rec.documentation_url,
            Some("https://docs.example.com".to_string())
        );
    }

    #[test]
    fn test_analyzer_registry_dispatch() {
        use std::path::PathBuf;

        let manifest = Manifest::from(PathBuf::from(
            "testdata/must-gather-valid/sample-openshift-release/cluster-scoped-resources/core/nodes/ip-10-0-0-1.control.plane.yaml"
        )).unwrap();
        let node = <Node as Resource>::from(manifest);
        let registry = AnalyzerRegistry::new();

        let analysis = registry.analyze(&node as &dyn ResourceV2).unwrap();

        // Should successfully analyze the node
        assert!(analysis.health_score > 0);
        assert!(analysis.health_score <= 100);
    }

    #[test]
    fn test_pod_analyzer_detects_glibc_runtime_failure_from_logs() {
        let manifest = Manifest::new();
        let mut pod = <Pod as Resource>::from(manifest);
        pod.push_container(Container {
            name: "training-operator".to_string(),
            current_log: "2026-05-22T07:47:21Z /manager: /lib64/libc.so.6: version `GLIBC_2.34' not found (required by /manager)\n".to_string(),
            current_log_path: None,
        });

        let analyzer = pod_analyzer::PodAnalyzer::new();
        let analysis = analyzer.analyze(&pod as &dyn ResourceV2).unwrap();

        assert!(
            analysis
                .issues
                .iter()
                .any(|issue| issue.title == "Runtime Library Mismatch")
        );
        assert!(analysis.health_score < 100);
    }
}
