//! Common types for health analysis

use serde::{Deserialize, Serialize};

/// Severity level of an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Informational message
    Info,
    /// Warning that should be investigated
    Warning,
    /// Error that needs attention
    Error,
    /// Critical issue requiring immediate action
    Critical,
}

/// Category of issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    /// Resource availability issues
    Availability,
    /// Performance degradation
    Performance,
    /// Configuration problems
    Configuration,
    /// Capacity or resource limits
    Capacity,
    /// Security concerns
    Security,
    /// Networking issues
    Network,
    /// Storage problems
    Storage,
    /// General health check
    Health,
}

/// An identified issue with a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Category of the issue
    pub category: IssueCategory,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Affected component or field
    pub affected_component: Option<String>,
    /// When the issue was detected (if available)
    pub detected_at: Option<String>,
}

impl Issue {
    /// Create a new issue
    pub fn new(
        severity: IssueSeverity,
        category: IssueCategory,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            title: title.into(),
            description: description.into(),
            affected_component: None,
            detected_at: None,
        }
    }

    /// Set the affected component
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.affected_component = Some(component.into());
        self
    }

    /// Set the detection time
    #[allow(dead_code)]
    pub fn with_detected_at(mut self, time: impl Into<String>) -> Self {
        self.detected_at = Some(time.into());
        self
    }
}

/// A recommendation for fixing an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Short title of the recommendation
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Command or action to take
    pub action: Option<String>,
    /// Link to documentation
    pub documentation_url: Option<String>,
}

impl Recommendation {
    /// Create a new recommendation
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            action: None,
            documentation_url: None,
        }
    }

    /// Set the action command
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Set the documentation URL
    pub fn with_docs(mut self, url: impl Into<String>) -> Self {
        self.documentation_url = Some(url.into());
        self
    }
}

/// Complete health analysis of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAnalysis {
    /// Overall health score (0-100)
    pub health_score: u8,
    /// List of identified issues
    pub issues: Vec<Issue>,
    /// List of recommendations
    pub recommendations: Vec<Recommendation>,
    /// Summary of the analysis
    pub summary: String,
}

impl HealthAnalysis {
    /// Create a new health analysis
    pub fn new(health_score: u8, summary: impl Into<String>) -> Self {
        Self {
            health_score: health_score.min(100),
            issues: Vec::new(),
            recommendations: Vec::new(),
            summary: summary.into(),
        }
    }

    /// Add an issue to the analysis
    pub fn add_issue(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    /// Add a recommendation to the analysis
    pub fn add_recommendation(&mut self, recommendation: Recommendation) {
        self.recommendations.push(recommendation);
    }

    /// Check if the resource is healthy (no errors or critical issues)
    #[allow(dead_code)]
    pub fn is_healthy(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Error | IssueSeverity::Critical))
    }

    /// Get the highest severity issue
    #[allow(dead_code)]
    pub fn max_severity(&self) -> Option<IssueSeverity> {
        self.issues.iter().map(|i| i.severity).max()
    }

    /// Count issues by severity
    #[allow(dead_code)]
    pub fn count_by_severity(&self, severity: IssueSeverity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }
}

impl Default for HealthAnalysis {
    fn default() -> Self {
        Self {
            health_score: 100,
            issues: Vec::new(),
            recommendations: Vec::new(),
            summary: "No analysis performed".to_string(),
        }
    }
}
