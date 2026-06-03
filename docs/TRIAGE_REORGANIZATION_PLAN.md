# Must-Gather Explorer Triage Reorganization - Implementation Plan

## Executive Summary

This document outlines the comprehensive plan to reorganize the must-gather explorer from a taxonomy-based structure (Namespaces, Cluster Operators, Nodes, Pods) to a triage-based workflow that mirrors how support engineers actually debug OpenShift clusters.

**Goal**: Transform the tool from a YAML browser into a true support engineer triage tool with actionable insights, smart filtering, and workflow-oriented navigation.

---

## Current State Analysis

### Existing Architecture

**Backend (Rust)**:
- `MustGather` struct in `src/mustgather.rs` - Main data structure
- Resource types: ClusterOperator, Machine, Node, Pod, Namespace, MachineConfigPool, etc.
- Basic analyzers in `src/analyzers/` for nodes, pods, operators, machines
- `ResourceV2` trait with health status, conditions, warnings, errors
- HTML generation in `src/html_v2.rs` with React frontend

**Frontend (React)**:
- Simple sidebar with flat resource list
- Basic filtering by status
- YAML viewer for resources
- No derived health views or smart filters
- No platform detection

### Current Limitations

1. **Taxonomy-focused**: Organized by Kubernetes object types, not triage workflows
2. **No derived insights**: Just lists resources without actionable summaries
3. **Limited filtering**: Basic status filters, no smart filters like "CrashLoop" or "Pending"
4. **No platform awareness**: Doesn't detect or surface platform-specific resources (Fusion, ODF, etc.)
5. **No search facets**: Can't search by `kind:pod ns:openshift-ingress status:degraded`
6. **Missing resource types**: No Deployments, StatefulSets, Routes, Services, PVCs, etc.

---

## Target Architecture

### New Navigation Structure

```
OVERVIEW
├── Dashboard (NEW)
├── Cluster Health (NEW)
│   ├── Cluster Operators
│   ├── ClusterVersion
│   ├── Alerts (if present)
│   ├── Insights (if present)
│   └── Degraded Conditions Summary
├── Alerts (NEW - dedicated)
└── Events (NEW - dedicated)

CORE
├── Nodes (ENHANCED)
│   ├── Node list with filters
│   ├── MachineConfigPools
│   ├── MachineConfigs
│   └── CSRs
├── Workloads (NEW - replaces Pods)
│   ├── Pods
│   ├── Deployments (NEW)
│   ├── StatefulSets (NEW)
│   ├── DaemonSets (NEW)
│   ├── Jobs (NEW)
│   ├── CronJobs (NEW)
│   └── ReplicaSets (NEW)
├── Namespaces (MOVED)
└── Operators (NEW)
    ├── CSVs
    ├── InstallPlans
    ├── Subscriptions
    └── OperatorGroups

INFRASTRUCTURE
├── Networking (NEW)
│   ├── Routes (NEW)
│   ├── Services (NEW)
│   ├── Endpoints (NEW)
│   ├── IngressControllers
│   └── NetworkPolicies
├── Storage (NEW)
│   ├── PVCs (NEW)
│   ├── PVs (NEW)
│   ├── StorageClasses (NEW)
│   └── VolumeAttachments (NEW)
└── Security/Auth (NEW)
    ├── OAuth
    ├── RBAC
    └── CSRs

LOGS (NEW)
└── Organized by operator/namespace/node

PLATFORM (DYNAMIC - NEW)
├── Fusion (if detected)
├── ODF (if detected)
├── Service Mesh (if detected)
└── Virtualization (if detected)
```

---

## Implementation Phases

## Phase 1: Backend Foundation (2-3 weeks)

### 1.1 New Resource Types

**Priority: HIGH** - Required for workload and infrastructure sections

#### Workload Resources

Create new resource files in `src/resources/`:

1. **`deployment.rs`**
   ```rust
   pub struct Deployment {
       pub manifest: Manifest,
   }

   impl ResourceV2 for Deployment {
       // Health checks:
       // - Available replicas vs desired
       // - Rollout status
       // - Image pull errors
       // - Pod failures
   }
   ```

2. **`statefulset.rs`**
   ```rust
   pub struct StatefulSet {
       pub manifest: Manifest,
   }

   impl ResourceV2 for StatefulSet {
       // Health checks:
       // - Ready replicas vs desired
       // - PVC binding issues
       // - Ordered pod failures
   }
   ```

3. **`daemonset.rs`**
   ```rust
   pub struct DaemonSet {
       pub manifest: Manifest,
   }

   impl ResourceV2 for DaemonSet {
       // Health checks:
       // - Desired vs current vs ready
       // - Node selector issues
       // - Scheduling failures
   }
   ```

4. **`job.rs`** and **`cronjob.rs`**
   ```rust
   pub struct Job {
       pub manifest: Manifest,
   }

   impl ResourceV2 for Job {
       // Health checks:
       // - Completion status
       // - Failed pods
       // - Backoff limit reached
   }
   ```

5. **`replicaset.rs`**
   ```rust
   pub struct ReplicaSet {
       pub manifest: Manifest,
   }
   ```

#### Networking Resources

6. **`route.rs`**
   ```rust
   pub struct Route {
       pub manifest: Manifest,
   }

   impl ResourceV2 for Route {
       // Health checks:
       // - Admitted status
       // - TLS configuration
       // - Backend service exists
   }
   ```

7. **`service.rs`**
   ```rust
   pub struct Service {
       pub manifest: Manifest,
   }

   impl ResourceV2 for Service {
       // Health checks:
       // - Endpoints exist
       // - LoadBalancer provisioned (if type=LoadBalancer)
       // - Selector matches pods
   }
   ```

8. **`endpoints.rs`** and **`endpointslice.rs`**
   ```rust
   pub struct Endpoints {
       pub manifest: Manifest,
   }

   impl ResourceV2 for Endpoints {
       // Health checks:
       // - Has addresses
       // - Not empty
   }
   ```

#### Storage Resources

9. **`persistentvolumeclaim.rs`**
   ```rust
   pub struct PersistentVolumeClaim {
       pub manifest: Manifest,
   }

   impl ResourceV2 for PersistentVolumeClaim {
       // Health checks:
       // - Bound status
       // - Storage class exists
       // - Capacity matches request
   }
   ```

10. **`persistentvolume.rs`**
    ```rust
    pub struct PersistentVolume {
        pub manifest: Manifest,
    }

    impl ResourceV2 for PersistentVolume {
        // Health checks:
        // - Bound vs Available
        // - Reclaim policy
    }
    ```

11. **`storageclass.rs`**
    ```rust
    pub struct StorageClass {
        pub manifest: Manifest,
    }
    ```

12. **`volumeattachment.rs`**
    ```rust
    pub struct VolumeAttachment {
        pub manifest: Manifest,
    }

    impl ResourceV2 for VolumeAttachment {
        // Health checks:
        // - Attached status
        // - Attach errors
    }
    ```

#### Operator Resources

13. **`clusterserviceversion.rs`** (CSV)
14. **`installplan.rs`**
15. **`subscription.rs`**
16. **`operatorgroup.rs`**

#### Configuration Resources

17. **`infrastructure.rs`**
18. **`dns.rs`**
19. **`proxy.rs`**
20. **`imageconfig.rs`**
21. **`apiserver.rs`**
22. **`authentication.rs`**

### 1.2 Update MustGather Struct

**File**: `src/mustgather.rs`

```rust
pub struct MustGather {
    // Existing fields...
    pub title: String,
    pub version: String,
    pub platformtype: String,
    pub clusteroperators: Vec<ClusterOperator>,
    pub nodes: Vec<Node>,
    pub namespaces: Vec<Namespace>,

    // NEW: Workload resources
    pub deployments: Vec<Deployment>,
    pub statefulsets: Vec<StatefulSet>,
    pub daemonsets: Vec<DaemonSet>,
    pub jobs: Vec<Job>,
    pub cronjobs: Vec<CronJob>,
    pub replicasets: Vec<ReplicaSet>,

    // NEW: Networking resources
    pub routes: Vec<Route>,
    pub services: Vec<Service>,
    pub endpoints: Vec<Endpoints>,
    pub endpointslices: Vec<EndpointSlice>,
    pub ingresscontrollers: Vec<IngressController>,
    pub networkpolicies: Vec<NetworkPolicy>,

    // NEW: Storage resources
    pub pvcs: Vec<PersistentVolumeClaim>,
    pub pvs: Vec<PersistentVolume>,
    pub storageclasses: Vec<StorageClass>,
    pub volumeattachments: Vec<VolumeAttachment>,

    // NEW: Operator resources
    pub csvs: Vec<ClusterServiceVersion>,
    pub installplans: Vec<InstallPlan>,
    pub subscriptions: Vec<Subscription>,
    pub operatorgroups: Vec<OperatorGroup>,

    // NEW: Configuration resources
    pub infrastructure: Option<Infrastructure>,
    pub dns: Option<DNS>,
    pub proxy: Option<Proxy>,
    pub imageconfig: Option<ImageConfig>,
    pub apiserver: Option<APIServer>,
    pub authentication: Option<Authentication>,

    // NEW: Platform detection
    pub detected_platforms: Vec<DetectedPlatform>,
}
```

### 1.3 Enhanced Analyzers

**Priority: HIGH** - Required for derived health views

Create new analyzers in `src/analyzers/`:

1. **`workload_analyzer.rs`**
   ```rust
   pub struct WorkloadAnalyzer;

   impl WorkloadAnalyzer {
       pub fn analyze_deployments(deployments: &[Deployment]) -> WorkloadSummary {
           // Count: CrashLoop, Pending, ImagePullBackOff, OOMKilled
       }

       pub fn analyze_statefulsets(statefulsets: &[StatefulSet]) -> WorkloadSummary {
           // Count: Not ready, PVC issues
       }

       pub fn analyze_daemonsets(daemonsets: &[DaemonSet]) -> WorkloadSummary {
           // Count: Not scheduled, node selector issues
       }
   }
   ```

2. **`storage_analyzer.rs`**
   ```rust
   pub struct StorageAnalyzer;

   impl StorageAnalyzer {
       pub fn analyze_pvcs(pvcs: &[PersistentVolumeClaim]) -> StorageSummary {
           // Count: Pending, Unbound, Lost
       }

       pub fn analyze_volume_attachments(attachments: &[VolumeAttachment]) -> StorageSummary {
           // Count: Attach failures
       }
   }
   ```

3. **`networking_analyzer.rs`**
   ```rust
   pub struct NetworkingAnalyzer;

   impl NetworkingAnalyzer {
       pub fn analyze_routes(routes: &[Route]) -> NetworkingSummary {
           // Count: Not admitted, TLS issues, missing backends
       }

       pub fn analyze_services(services: &[Service]) -> NetworkingSummary {
           // Count: Missing endpoints, LoadBalancer issues
       }
   }
   ```

4. **`cluster_health_analyzer.rs`**
   ```rust
   pub struct ClusterHealthAnalyzer;

   impl ClusterHealthAnalyzer {
       pub fn analyze(mg: &MustGather) -> ClusterHealthSummary {
           ClusterHealthSummary {
               degraded_operators: count_degraded_operators(&mg.clusteroperators),
               unavailable_operators: count_unavailable_operators(&mg.clusteroperators),
               not_ready_nodes: count_not_ready_nodes(&mg.nodes),
               crashloop_pods: count_crashloop_pods(&mg.pods),
               pending_pvcs: count_pending_pvcs(&mg.pvcs),
               failed_routes: count_failed_routes(&mg.routes),
               overall_status: determine_overall_status(),
               warnings: collect_warnings(),
               errors: collect_errors(),
           }
       }
   }
   ```

### 1.4 Platform Detection

**Priority: MEDIUM** - Nice to have for Phase 1

**File**: `src/platform_detection.rs`

```rust
pub enum DetectedPlatform {
    Fusion,
    ODF,
    ServiceMesh,
    ACM,
    Virtualization,
    CloudPak,
}

pub struct PlatformDetector;

impl PlatformDetector {
    pub fn detect(mg: &MustGather) -> Vec<DetectedPlatform> {
        let mut platforms = Vec::new();

        // Detect Fusion
        if mg.namespaces.iter().any(|ns| ns.name.contains("ibm-spectrum-fusion")) {
            platforms.push(DetectedPlatform::Fusion);
        }

        // Detect ODF
        if mg.namespaces.iter().any(|ns| ns.name == "openshift-storage") {
            platforms.push(DetectedPlatform::ODF);
        }

        // Detect Service Mesh
        if mg.namespaces.iter().any(|ns| ns.name == "istio-system") {
            platforms.push(DetectedPlatform::ServiceMesh);
        }

        // Detect ACM
        if mg.namespaces.iter().any(|ns| ns.name.contains("open-cluster-management")) {
            platforms.push(DetectedPlatform::ACM);
        }

        // Detect Virtualization
        if mg.namespaces.iter().any(|ns| ns.name == "openshift-cnv") {
            platforms.push(DetectedPlatform::Virtualization);
        }

        // Detect Cloud Pak / CPD
        if mg.namespaces.iter().any(|ns| ns.name.contains("zen") || ns.name.contains("cpd")) {
            platforms.push(DetectedPlatform::CloudPak);
        }

        platforms
    }
}
```

### 1.5 Search Facet System

**Priority: MEDIUM** - Can be added in Phase 3

**File**: `src/search.rs`

```rust
pub struct SearchQuery {
    pub kind: Option<String>,      // kind:pod
    pub namespace: Option<String>,  // ns:openshift-ingress
    pub status: Option<String>,     // status:degraded
    pub node: Option<String>,       // node:worker-1
    pub operator: Option<String>,   // operator:storage
    pub text: Option<String>,       // free text search
}

pub struct SearchEngine;

impl SearchEngine {
    pub fn parse_query(query: &str) -> SearchQuery {
        // Parse faceted search: "kind:pod ns:openshift-ingress status:crashloop"
    }

    pub fn search(mg: &MustGather, query: SearchQuery) -> Vec<SearchResult> {
        // Search across all resources
    }
}
```

---

## Phase 2: Frontend Redesign (2-3 weeks)

### 2.1 New Sidebar Component

**Priority: HIGH**

**File**: `frontend/src/components/Sidebar.jsx`

```jsx
export function Sidebar({ sections, activeSection, onSectionClick }) {
  return (
    <aside className="sidebar">
      {/* OVERVIEW Section */}
      <SidebarSection title="OVERVIEW" defaultOpen={true}>
        <SidebarItem
          id="dashboard"
          label="Dashboard"
          icon="📊"
          active={activeSection === 'dashboard'}
          onClick={() => onSectionClick('dashboard')}
        />
        <SidebarItem
          id="cluster-health"
          label="Cluster Health"
          icon="🏥"
          badge={{ count: 7, type: 'warning' }}
          active={activeSection === 'cluster-health'}
          onClick={() => onSectionClick('cluster-health')}
        />
        <SidebarItem
          id="alerts"
          label="Alerts"
          icon="🚨"
          badge={{ count: 3, type: 'error' }}
        />
        <SidebarItem
          id="events"
          label="Events"
          icon="📋"
        />
      </SidebarSection>

      {/* CORE Section */}
      <SidebarSection title="CORE" defaultOpen={true}>
        <SidebarItem
          id="nodes"
          label="Nodes"
          icon="🖥️"
          badge={{ count: 3, type: 'success' }}
        />
        <SidebarItem
          id="workloads"
          label="Workloads"
          icon="📦"
          badge={{ count: 45, type: 'warning' }}
          expandable={true}
        >
          <SidebarSubItem label="Pods" count={120} warnings={4} />
          <SidebarSubItem label="Deployments" count={25} />
          <SidebarSubItem label="StatefulSets" count={5} />
          <SidebarSubItem label="DaemonSets" count={8} />
          <SidebarSubItem label="Jobs" count={12} errors={2} />
        </SidebarItem>
        <SidebarItem id="namespaces" label="Namespaces" icon="📁" count={45} />
        <SidebarItem id="operators" label="Operators" icon="⚙️" count={28} />
      </SidebarSection>

      {/* INFRASTRUCTURE Section */}
      <SidebarSection title="INFRASTRUCTURE">
        <SidebarItem id="networking" label="Networking" icon="🌐" />
        <SidebarItem id="storage" label="Storage" icon="💾" warnings={3} />
        <SidebarItem id="security" label="Security/Auth" icon="🔒" />
      </SidebarSection>

      {/* LOGS Section */}
      <SidebarSection title="LOGS">
        <SidebarItem id="logs" label="Logs" icon="📄" />
      </SidebarSection>

      {/* PLATFORM Section (dynamic) */}
      {detectedPlatforms.length > 0 && (
        <SidebarSection title="PLATFORM">
          {detectedPlatforms.map(platform => (
            <SidebarItem
              key={platform.id}
              id={platform.id}
              label={platform.label}
              icon={platform.icon}
            />
          ))}
        </SidebarSection>
      )}
    </aside>
  );
}
```

### 2.2 Dashboard Component

**Priority: HIGH**

**File**: `frontend/src/components/Dashboard.jsx`

```jsx
export function Dashboard({ clusterHealth }) {
  return (
    <div className="dashboard">
      <h1>Cluster Health Overview</h1>

      {/* Overall Status Card */}
      <StatusCard
        status={clusterHealth.overall_status}
        title="Overall Cluster Health"
      />

      {/* Quick Stats Grid */}
      <div className="stats-grid">
        <StatCard
          label="Degraded Operators"
          value={clusterHealth.degraded_operators}
          severity="error"
          link="/cluster-health#operators"
        />
        <StatCard
          label="Not Ready Nodes"
          value={clusterHealth.not_ready_nodes}
          severity="warning"
          link="/nodes"
        />
        <StatCard
          label="CrashLoop Pods"
          value={clusterHealth.crashloop_pods}
          severity="error"
          link="/workloads?filter=crashloop"
        />
        <StatCard
          label="Pending PVCs"
          value={clusterHealth.pending_pvcs}
          severity="warning"
          link="/storage?filter=pending"
        />
      </div>

      {/* Top Issues */}
      <IssuesList
        title="Top Issues Requiring Attention"
        issues={clusterHealth.top_issues}
      />

      {/* Recent Events */}
      <EventsTimeline
        events={clusterHealth.recent_events}
        limit={10}
      />
    </div>
  );
}
```

### 2.3 Cluster Health Component

**Priority: HIGH**

**File**: `frontend/src/components/ClusterHealth.jsx`

```jsx
export function ClusterHealth({ operators, version, alerts, insights }) {
  return (
    <div className="cluster-health">
      <h1>Cluster Health</h1>

      {/* Cluster Version */}
      <Card title="Cluster Version">
        <VersionInfo version={version} />
      </Card>

      {/* Cluster Operators */}
      <Card
        title="Cluster Operators"
        badge={{
          text: `${degradedCount} degraded, ${unavailableCount} unavailable`,
          type: degradedCount > 0 ? 'error' : 'success'
        }}
      >
        <OperatorsList
          operators={operators}
          filters={['degraded', 'unavailable', 'progressing']}
        />
      </Card>

      {/* Alerts (if present) */}
      {alerts && (
        <Card title="Firing Alerts" badge={{ count: alerts.length, type: 'error' }}>
          <AlertsList alerts={alerts} />
        </Card>
      )}

      {/* Insights (if present) */}
      {insights && (
        <Card title="Insights Recommendations">
          <InsightsList insights={insights} />
        </Card>
      )}

      {/* Degraded Conditions Summary */}
      <Card title="Degraded Conditions Across Cluster">
        <ConditionsSummary conditions={getAllDegradedConditions()} />
      </Card>
    </div>
  );
}
```

### 2.4 Workloads Component

**Priority: HIGH**

**File**: `frontend/src/components/Workloads.jsx`

```jsx
export function Workloads({ pods, deployments, statefulsets, daemonsets, jobs }) {
  const [activeTab, setActiveTab] = useState('pods');
  const [filters, setFilters] = useState({
    crashloop: false,
    pending: false,
    imagepull: false,
    oom: false,
  });

  return (
    <div className="workloads">
      <h1>Workloads</h1>

      {/* Summary Cards */}
      <div className="summary-grid">
        <SummaryCard
          label="CrashLoop"
          count={countCrashLoop(pods)}
          severity="error"
          onClick={() => setFilters({ crashloop: true })}
        />
        <SummaryCard
          label="Pending"
          count={countPending(pods)}
          severity="warning"
          onClick={() => setFilters({ pending: true })}
        />
        <SummaryCard
          label="ImagePullBackOff"
          count={countImagePull(pods)}
          severity="error"
          onClick={() => setFilters({ imagepull: true })}
        />
        <SummaryCard
          label="OOMKilled"
          count={countOOM(pods)}
          severity="error"
          onClick={() => setFilters({ oom: true })}
        />
      </div>

      {/* Tabs */}
      <Tabs
        tabs={[
          { id: 'pods', label: 'Pods', count: pods.length },
          { id: 'deployments', label: 'Deployments', count: deployments.length },
          { id: 'statefulsets', label: 'StatefulSets', count: statefulsets.length },
          { id: 'daemonsets', label: 'DaemonSets', count: daemonsets.length },
          { id: 'jobs', label: 'Jobs', count: jobs.length },
        ]}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />

      {/* Resource List */}
      <ResourceList
        resources={getActiveResources(activeTab)}
        filters={filters}
      />
    </div>
  );
}
```

### 2.5 Smart Filters Component

**Priority: MEDIUM**

**File**: `frontend/src/components/SmartFilters.jsx`

```jsx
export function SmartFilters({ onFilterChange, activeFilters }) {
  const filters = [
    { id: 'degraded', label: 'Degraded', icon: '⚠️' },
    { id: 'progressing', label: 'Progressing', icon: '🔄' },
    { id: 'crashloop', label: 'CrashLoop', icon: '🔁' },
    { id: 'pending', label: 'Pending', icon: '⏳' },
    { id: 'unavailable', label: 'Unavailable', icon: '❌' },
    { id: 'cert-issues', label: 'Certificate Issues', icon: '🔒' },
    { id: 'storage-problems', label: 'Storage Problems', icon: '💾' },
    { id: 'network-problems', label: 'Network Problems', icon: '🌐' },
    { id: 'upgrade-blockers', label: 'Upgrade Blockers', icon: '🚫' },
  ];

  return (
    <div className="smart-filters">
      {filters.map(filter => (
        <FilterButton
          key={filter.id}
          label={filter.label}
          icon={filter.icon}
          active={activeFilters.includes(filter.id)}
          onClick={() => onFilterChange(filter.id)}
        />
      ))}
    </div>
  );
}
```

### 2.6 Search Facet UI

**Priority: LOW** - Phase 3

**File**: `frontend/src/components/SearchBar.jsx`

```jsx
export function SearchBar({ onSearch }) {
  const [query, setQuery] = useState('');
  const [suggestions, setSuggestions] = useState([]);

  const handleInput = (value) => {
    setQuery(value);

    // Parse facets and provide autocomplete
    const facets = parseFacets(value);
    const suggestions = generateSuggestions(facets);
    setSuggestions(suggestions);
  };

  return (
    <div className="search-bar">
      <input
        type="text"
        placeholder="Search: kind:pod ns:openshift-ingress status:degraded"
        value={query}
        onChange={(e) => handleInput(e.target.value)}
        onKeyPress={(e) => e.key === 'Enter' && onSearch(query)}
      />

      {/* Autocomplete Suggestions */}
      {suggestions.length > 0 && (
        <div className="suggestions">
          {suggestions.map(suggestion => (
            <SuggestionItem
              key={suggestion.id}
              suggestion={suggestion}
              onClick={() => applySuggestion(suggestion)}
            />
          ))}
        </div>
      )}

      {/* Facet Pills */}
      <div className="facet-pills">
        {parseFacets(query).map(facet => (
          <FacetPill
            key={facet.key}
            facet={facet}
            onRemove={() => removeFacet(facet)}
          />
        ))}
      </div>
    </div>
  );
}
```

---

## Phase 3: Data Integration (1-2 weeks)

### 3.1 Update HTML Generation

**Priority: HIGH**

**File**: `src/html_v2.rs`

Update `MustGatherData::from_must_gather()` to include new sections:

```rust
impl MustGatherData {
    pub fn from_must_gather(mg: &MustGather) -> Self {
        let analyzer_registry = AnalyzerRegistry::new();
        let cluster_health = ClusterHealthAnalyzer::analyze(mg);
        let detected_platforms = PlatformDetector::detect(mg);

        let mut sections = Vec::new();

        // OVERVIEW sections
        sections.push(Self::create_dashboard_section(&cluster_health));
        sections.push(Self::create_cluster_health_section(mg, &cluster_health));

        if !mg.alerts.is_empty() {
            sections.push(Self::create_alerts_section(&mg.alerts));
        }

        if !mg.events.is_empty() {
            sections.push(Self::create_events_section(&mg.events));
        }

        // CORE sections
        sections.push(Self::create_nodes_section(mg));
        sections.push(Self::create_workloads_section(mg));
        sections.push(Self::create_namespaces_section(mg));
        sections.push(Self::create_operators_section(mg));

        // INFRASTRUCTURE sections
        sections.push(Self::create_networking_section(mg));
        sections.push(Self::create_storage_section(mg));
        sections.push(Self::create_security_section(mg));

        // LOGS section
        if has_logs(mg) {
            sections.push(Self::create_logs_section(mg));
        }

        // PLATFORM sections (dynamic)
        for platform in &detected_platforms {
            sections.push(Self::create_platform_section(mg, platform));
        }

        MustGatherData {
            title: mg.title.clone(),
            version: Some(mg.version.clone()),
            platform: Some(mg.platformtype.clone()),
            status: cluster_health.overall_status.to_string(),
            stats: Self::create_stats(&cluster_health),
            sections,
            cluster_health: Some(cluster_health),
            detected_platforms,
        }
    }

    fn create_workloads_section(mg: &MustGather) -> SectionData {
        let workload_summary = WorkloadAnalyzer::analyze_all(mg);

        SectionData {
            id: "workloads".to_string(),
            label: format!(
                "Workloads - CrashLoop ({}), Pending ({}), OOMKilled ({})",
                workload_summary.crashloop_count,
                workload_summary.pending_count,
                workload_summary.oom_count
            ),
            count: workload_summary.total_count,
            error_count: workload_summary.error_count,
            warning_count: workload_summary.warning_count,
            subsections: vec![
                Self::create_subsection("pods", "Pods", &mg.pods),
                Self::create_subsection("deployments", "Deployments", &mg.deployments),
                Self::create_subsection("statefulsets", "StatefulSets", &mg.statefulsets),
                Self::create_subsection("daemonsets", "DaemonSets", &mg.daemonsets),
                Self::create_subsection("jobs", "Jobs", &mg.jobs),
                Self::create_subsection("cronjobs", "CronJobs", &mg.cronjobs),
            ],
        }
    }
}
```

### 3.2 Serialization Updates

Add new fields to serialization structs:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MustGatherData {
    pub title: String,
    pub version: Option<String>,
    pub platform: Option<String>,
    pub status: String,
    pub stats: Vec<StatData>,
    pub sections: Vec<SectionData>,

    // NEW
    pub cluster_health: Option<ClusterHealthSummary>,
    pub detected_platforms: Vec<DetectedPlatform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectionData {
    pub id: String,
    pub label: String,
    pub count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub resources: Vec<ResourceData>,

    // NEW
    pub subsections: Vec<SubsectionData>,
    pub summary: Option<String>,
    pub filters: Vec<FilterOption>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterHealthSummary {
    pub overall_status: HealthStatus,
    pub degraded_operators: usize,
    pub unavailable_operators: usize,
    pub not_ready_nodes: usize,
    pub crashloop_pods: usize,
    pub pending_pvcs: usize,
    pub failed_routes: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub top_issues: Vec<Issue>,
}
```

---

## Phase 4: Testing & Documentation (1 week)

### 4.1 Testing Strategy

1. **Unit Tests**
   - Test each new resource type's health checks
   - Test analyzers with various scenarios
   - Test platform detection logic
   - Test search facet parsing

2. **Integration Tests**
   - Test full must-gather parsing with new resources
   - Test HTML generation with all sections
   - Test frontend rendering with mock data

3. **Manual Testing**
   - Test with real must-gather archives
   - Test with Fusion must-gathers
   - Test with ODF must-gathers
   - Test with various cluster states (healthy, degraded, failing)

### 4.2 Documentation

1. **User Guide** (`docs/USER_GUIDE.md`)
   - How to navigate the new structure
   - Understanding derived health views
   - Using smart filters
   - Using search facets
   - Platform-specific sections

2. **Developer Guide** (`docs/DEVELOPER_GUIDE.md`)
   - Adding new resource types
   - Creating analyzers
   - Adding platform detection
   - Extending the UI

3. **README Updates**
   - Update screenshots
   - Update feature list
   - Add examples

---

## Implementation Timeline

### Week 1-2: Backend Foundation
- [ ] Create 20+ new resource types
- [ ] Update MustGather struct
- [ ] Implement basic analyzers

### Week 3-4: Enhanced Analysis
- [ ] Complete all analyzers
- [ ] Implement platform detection
- [ ] Add search facet system

### Week 5-6: Frontend Redesign
- [ ] New sidebar with collapsible sections
- [ ] Dashboard component
- [ ] Cluster Health component
- [ ] Workloads component

### Week 7-8: Integration & Polish
- [ ] Wire up backend to frontend
- [ ] Smart filters
- [ ] Search facet UI
- [ ] Badge system

### Week 9: Testing & Documentation
- [ ] Comprehensive testing
- [ ] Documentation
- [ ] User guide

---

## Success Criteria

1. **Navigation**: Support engineers can quickly find what they need following their natural triage workflow
2. **Insights**: Derived health views show actionable summaries (e.g., "CrashLoop (4), Pending (7)")
3. **Filtering**: Smart filters work intuitively (CrashLoop, Pending, Degraded, etc.)
4. **Platform Awareness**: Automatically detects and surfaces Fusion, ODF, Service Mesh, etc.
5. **Search**: Faceted search works (kind:pod ns:openshift-ingress status:degraded)
6. **Performance**: Loads large must-gathers (1000+ resources) in <5 seconds

---

## Risk Mitigation

### Risk: Breaking Existing Functionality
**Mitigation**: Keep old structure alongside new, add feature flag

### Risk: Performance with Large Must-Gathers
**Mitigation**: Implement lazy loading, pagination, virtual scrolling

### Risk: Complex State Management
**Mitigation**: Use React Context or state management library

### Risk: Incomplete Must-Gathers
**Mitigation**: Gracefully handle missing resources, show warnings

---

## Future Enhancements (Post-MVP)

1. **AI-Powered Insights**: Use LLM to analyze logs and suggest fixes
2. **Comparison Mode**: Compare two must-gathers side-by-side
3. **Export Reports**: Generate PDF/HTML reports
4. **Collaboration**: Share annotations and findings
5. **Integration**: Connect to Jira, ServiceNow, etc.
6. **Real-time Mode**: Connect to live cluster (not just must-gather)

---

## Appendix A: Resource Type Mapping

| Resource Type | API Group | Namespace Scoped | Must-Gather Path |
|--------------|-----------|------------------|------------------|
| Deployment | apps | Yes | namespaces/{ns}/apps/deployments.yaml |
| StatefulSet | apps | Yes | namespaces/{ns}/apps/statefulsets.yaml |
| DaemonSet | apps | Yes | namespaces/{ns}/apps/daemonsets.yaml |
| Job | batch | Yes | namespaces/{ns}/batch/jobs.yaml |
| CronJob | batch | Yes | namespaces/{ns}/batch/cronjobs.yaml |
| ReplicaSet | apps | Yes | namespaces/{ns}/apps/replicasets.yaml |
| Route | route.openshift.io | Yes | namespaces/{ns}/route.openshift.io/routes.yaml |
| Service | core | Yes | namespaces/{ns}/core/services.yaml |
| Endpoints | core | Yes | namespaces/{ns}/core/endpoints.yaml |
| EndpointSlice | discovery.k8s.io | Yes | namespaces/{ns}/discovery.k8s.io/endpointslices.yaml |
| PVC | core | Yes | namespaces/{ns}/core/persistentvolumeclaims.yaml |
| PV | core | No | cluster-scoped-resources/core/persistentvolumes/ |
| StorageClass | storage.k8s.io | No | cluster-scoped-resources/storage.k8s.io/storageclasses/ |
| VolumeAttachment | storage.k8s.io | No | cluster-scoped-resources/storage.k8s.io/volumeattachments/ |

---

## Appendix B: Smart Filter Definitions

| Filter | Applies To | Logic |
|--------|-----------|-------|
| Degraded | Operators, Nodes, MCPs | condition.type == "Degraded" && condition.status == "True" |
| Progressing | Operators, MCPs | condition.type == "Progressing" && condition.status == "True" |
| CrashLoop | Pods | status.containerStatuses[].state.waiting.reason == "CrashLoopBackOff" |
| Pending | Pods, PVCs | status.phase == "Pending" |
| Unavailable | Operators | condition.type == "Available" && condition.status == "False" |
| Certificate Issues | CSRs, Secrets | CSR not approved, cert expired |
| Storage Problems | PVCs, PVs | Pending PVCs, unbound PVs, attach failures |
| Network Problems | Routes, Services | Route not admitted, service no endpoints |
| Upgrade Blockers | Operators, Nodes | Degraded operators, not ready nodes during upgrade |

---

**Document Version**: 1.0
**Last Updated**: 2026-05-27
**Author**: Bob (AI Planning Mode)