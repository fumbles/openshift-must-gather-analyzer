//! Health analyzer for Networking resources (Route, Service, Endpoints, IngressController)

use super::{HealthAnalysis, HealthAnalyzer, Issue, IssueCategory, IssueSeverity, Recommendation};
use crate::resources::{HealthStatus, ResourceV2};
use anyhow::Result;

pub struct NetworkingAnalyzer;

impl NetworkingAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn calculate_health_score(&self, resource: &dyn ResourceV2) -> u8 {
        let mut score = 100u8;

        match resource.health_status() {
            HealthStatus::Healthy => {}
            HealthStatus::Warning => score = score.saturating_sub(25),
            HealthStatus::Error => score = score.saturating_sub(60),
            HealthStatus::Unknown => score = score.saturating_sub(10),
        }

        score
    }

    fn analyze_route(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        if let Some(admitted) = key_fields
            .get("admitted")
            .and_then(|s| s.parse::<bool>().ok())
        {
            if !admitted {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Critical,
                    IssueCategory::Network,
                    "Route Not Admitted",
                    format!(
                        "Route {} is not admitted - traffic cannot reach this route",
                        resource.name()
                    ),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check Route Configuration",
                        "Verify route host, path, and service configuration",
                    )
                    .with_action(format!(
                        "oc describe route {} -n {}",
                        resource.name(),
                        resource.namespace().unwrap_or("default")
                    )),
                );

                if let Some(host) = key_fields.get("host") {
                    analysis.add_recommendation(
                        Recommendation::new(
                            "Verify Route Host",
                            format!(
                                "Check if hostname '{}' conflicts with existing routes",
                                host
                            ),
                        )
                        .with_action(format!(
                            "oc get routes --all-namespaces -o wide | grep {}",
                            host
                        )),
                    );
                }

                if let Some(service) = key_fields.get("service_name") {
                    analysis.add_recommendation(
                        Recommendation::new(
                            "Check Backend Service",
                            format!("Verify that service '{}' exists and has endpoints", service),
                        )
                        .with_action(format!(
                            "oc get service {} -n {} && oc get endpoints {} -n {}",
                            service,
                            resource.namespace().unwrap_or("default"),
                            service,
                            resource.namespace().unwrap_or("default")
                        )),
                    );
                }
            }
        }
    }

    fn analyze_service(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        // Check if service has a selector
        if let Some(selector) = key_fields.get("selector") {
            if selector.is_empty() {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Configuration,
                    "Service Has No Selector",
                    format!(
                        "Service {} has no selector - will not automatically route to pods",
                        resource.name()
                    ),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Add Selector or Manual Endpoints",
                        "Either add a selector to automatically target pods, or manually create endpoints",
                    )
                    .with_action(format!("oc edit service {} -n {}",
                        resource.name(),
                        resource.namespace().unwrap_or("default")))
                );
            }
        }

        // Note: We can't directly check if service has endpoints here without cross-referencing
        // The analyzer would need access to all resources to do that correlation
        // For now, we'll just note that endpoints should be checked
        analysis.add_recommendation(
            Recommendation::new(
                "Verify Service Endpoints",
                "Check that the service has ready endpoints",
            )
            .with_action(format!(
                "oc get endpoints {} -n {}",
                resource.name(),
                resource.namespace().unwrap_or("default")
            )),
        );
    }

    fn analyze_endpoints(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        let ready_count = key_fields
            .get("ready_addresses")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let not_ready_count = key_fields
            .get("not_ready_addresses")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        if ready_count == 0 {
            if not_ready_count > 0 {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Availability,
                    "No Ready Endpoints",
                    format!(
                        "Endpoints {} has no ready addresses - {} not ready",
                        resource.name(),
                        not_ready_count
                    ),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check Pod Readiness",
                        "Investigate why pods are not ready",
                    )
                    .with_action(format!(
                        "oc get pods -n {} -o wide",
                        resource.namespace().unwrap_or("default")
                    )),
                );
            } else {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Error,
                    IssueCategory::Availability,
                    "No Endpoints Available",
                    format!(
                        "Endpoints {} has no addresses - service has no backing pods",
                        resource.name()
                    ),
                ));

                analysis.add_recommendation(
                    Recommendation::new(
                        "Check Service Selector",
                        "Verify that the service selector matches existing pods",
                    )
                    .with_action(format!(
                        "oc describe service {} -n {} && oc get pods -n {} --show-labels",
                        resource.name(),
                        resource.namespace().unwrap_or("default"),
                        resource.namespace().unwrap_or("default")
                    )),
                );
            }
        } else if not_ready_count > 0 {
            analysis.add_issue(Issue::new(
                IssueSeverity::Info,
                IssueCategory::Availability,
                "Some Endpoints Not Ready",
                format!(
                    "Endpoints {} has {} ready and {} not ready addresses",
                    resource.name(),
                    ready_count,
                    not_ready_count
                ),
            ));
        }
    }

    fn analyze_ingress_controller(&self, resource: &dyn ResourceV2, analysis: &mut HealthAnalysis) {
        let key_fields = resource.key_fields();

        let replicas = key_fields
            .get("replicas")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let available = key_fields
            .get("available_replicas")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        if available == 0 {
            analysis.add_issue(Issue::new(
                IssueSeverity::Critical,
                IssueCategory::Availability,
                "IngressController Unavailable",
                format!(
                    "IngressController {} has no available replicas - ingress is down",
                    resource.name()
                ),
            ));

            analysis.add_recommendation(
                Recommendation::new(
                    "Check IngressController Pods",
                    "Investigate why ingress controller pods are not available",
                )
                .with_action(format!("oc get pods -n openshift-ingress-operator && oc describe ingresscontroller {} -n openshift-ingress-operator",
                    resource.name()))
            );
        } else if available < replicas {
            analysis.add_issue(Issue::new(
                IssueSeverity::Warning,
                IssueCategory::Availability,
                "IngressController Degraded",
                format!(
                    "IngressController {} has only {}/{} replicas available",
                    resource.name(),
                    available,
                    replicas
                ),
            ));

            analysis.add_recommendation(
                Recommendation::new(
                    "Scale IngressController",
                    "Check why not all replicas are available",
                )
                .with_action(format!("oc get pods -n openshift-ingress -l ingresscontroller.operator.openshift.io/deployment-ingresscontroller={}",
                    resource.name()))
            );
        }

        // Check for degraded condition
        for condition in resource.conditions() {
            if condition.type_ == "Degraded" && condition.status == "True" {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Warning,
                    IssueCategory::Health,
                    "IngressController Degraded",
                    condition
                        .message
                        .as_deref()
                        .unwrap_or("IngressController is degraded")
                        .to_string(),
                ));
            }

            if condition.type_ == "Available" && condition.status == "False" {
                analysis.add_issue(Issue::new(
                    IssueSeverity::Critical,
                    IssueCategory::Availability,
                    "IngressController Not Available",
                    condition
                        .message
                        .as_deref()
                        .unwrap_or("IngressController is not available")
                        .to_string(),
                ));
            }
        }
    }
}

impl HealthAnalyzer for NetworkingAnalyzer {
    fn analyze(&self, resource: &dyn ResourceV2) -> Result<HealthAnalysis> {
        let health_score = self.calculate_health_score(resource);

        let summary = match resource.kind() {
            "Route" => format!("Route {} health analysis", resource.name()),
            "Service" => format!("Service {} health analysis", resource.name()),
            "Endpoints" => format!("Endpoints {} health analysis", resource.name()),
            "IngressController" => format!("IngressController {} health analysis", resource.name()),
            _ => format!("Networking resource {} health analysis", resource.name()),
        };

        let mut analysis = HealthAnalysis::new(health_score, summary);

        // Perform kind-specific analysis
        match resource.kind() {
            "Route" => self.analyze_route(resource, &mut analysis),
            "Service" => self.analyze_service(resource, &mut analysis),
            "Endpoints" => self.analyze_endpoints(resource, &mut analysis),
            "IngressController" => self.analyze_ingress_controller(resource, &mut analysis),
            _ => {}
        }

        // Add general recommendations if there are critical issues
        if analysis
            .issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::Error))
        {
            analysis.add_recommendation(
                Recommendation::new(
                    "Review Network Events",
                    "Check cluster events for networking-related issues",
                )
                .with_action(format!(
                    "oc get events -n {} --field-selector involvedObject.name={}",
                    resource.namespace().unwrap_or("default"),
                    resource.name()
                )),
            );
        }

        Ok(analysis)
    }

    fn supported_kinds(&self) -> Vec<&'static str> {
        vec!["Route", "Service", "Endpoints", "IngressController"]
    }
}

impl Default for NetworkingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
