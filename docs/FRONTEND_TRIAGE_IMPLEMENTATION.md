# Frontend Triage Navigation Implementation

## Overview
This document describes the implementation of the triage-based navigation structure for the Must-Gather Explorer frontend, completed as part of Phase 3 of the triage reorganization plan.

## Implementation Date
2026-05-27

## Components Created

### 1. Dashboard Component (`frontend/src/components/Dashboard.jsx`)
- **Purpose**: High-level cluster health overview
- **Features**:
  - Health summary cards for Nodes, Pods, Operators, and Cluster Version
  - Platform detection badges (Fusion, ODF, Service Mesh, ACM, Virtualization, CPD)
  - Issues summary with categorized problems
- **Helper Components**:
  - `HealthCard`: Displays resource health metrics
  - `PlatformBadge`: Shows detected platform components
  - `IssuesSummary`: Lists issues requiring attention

### 2. Workloads Component (`frontend/src/components/Workloads.jsx`)
- **Purpose**: Combined view for all workload types
- **Features**:
  - Tabbed interface for different workload types
  - Issue count badges on tabs
  - Type-specific details for each workload
  - Support for: Deployments, StatefulSets, DaemonSets, Jobs, CronJobs, ReplicaSets, Pods
- **Sub-components**:
  - `WorkloadTabs`: Tab navigation with issue counts
  - `WorkloadResourceList`: Displays workload resources with type-specific details

### 3. Networking Component (`frontend/src/components/Networking.jsx`)
- **Purpose**: Combined view for networking resources
- **Features**:
  - Tabbed interface for networking resources
  - Issue count badges on tabs
  - Support for: Routes, Services, Endpoints, Ingress Controllers
- **Sub-components**:
  - `NetworkingTabs`: Tab navigation with issue counts
  - `NetworkingResourceList`: Displays networking resources with type-specific details

### 4. Storage Component (`frontend/src/components/Storage.jsx`)
- **Purpose**: Combined view for storage resources
- **Features**:
  - Tabbed interface for storage resources
  - Issue count badges on tabs
  - Support for: PVCs, PVs, Storage Classes, Volume Attachments
- **Sub-components**:
  - `StorageTabs`: Tab navigation with issue counts
  - `StorageResourceList`: Displays storage resources with type-specific details

## Components Updated

### 1. Sidebar Component (`frontend/src/components/Sidebar.jsx`)
- **Changes**:
  - Replaced flat resource list with collapsible triage sections
  - Added section icons (📊, 🔧, 🏗️, 🚀)
  - Implemented expandable/collapsible sections
  - Added issue count badges on subsections
  - Dynamic platform sections based on detection
- **Sections**:
  - **OVERVIEW**: Dashboard, Cluster Health
  - **CORE**: Nodes, Workloads, Namespaces
  - **INFRASTRUCTURE**: Networking, Storage
  - **PLATFORM**: Dynamic sections (Fusion, ODF, Service Mesh, ACM, Virtualization, CPD)

### 2. App Component (`frontend/src/App.jsx`)
- **Changes**:
  - Complete rewrite to support triage-based navigation
  - Added routing for new sections
  - Implemented placeholder components for Nodes, Namespaces, ClusterHealth
  - Added PlatformView component for platform-specific sections
  - Updated Hero component integration to use triage data structure
  - Simplified navigation without search/filter (focused on triage view)

### 3. Main Entry Point (`frontend/src/main.jsx`)
- **Changes**:
  - Updated mock data structure to match triage format
  - Changed from flat sections array to hierarchical triage structure
  - Added overview, core, infrastructure, and platform sections

### 4. Component Index (`frontend/src/components/index.js`)
- **Changes**:
  - Added exports for new components: Dashboard, Workloads, Networking, Storage

## Data Structure

### Expected Triage Data Format
```javascript
{
  overview: {
    dashboard: {
      cluster_name: string,
      cluster_version: string,
      platform_type: string,
      cluster_status: string,
      total_nodes: number,
      healthy_nodes: number,
      total_pods: number,
      healthy_pods: number,
      total_operators: number,
      degraded_operators: number
    },
    cluster_health: {
      items: Array,
      degraded_count: number
    }
  },
  core: {
    nodes: {
      items: Array,
      not_ready_count: number
    },
    workloads: {
      deployments: { items: Array, unavailable_count: number },
      statefulsets: { items: Array, unavailable_count: number },
      daemonsets: { items: Array, misscheduled_count: number },
      jobs: { items: Array, failed_count: number },
      cronjobs: { items: Array, suspended_count: number },
      replicasets: { items: Array, unavailable_count: number },
      pods: { items: Array, crashloop_count: number, pending_count: number }
    },
    namespaces: {
      items: Array
    }
  },
  infrastructure: {
    networking: {
      routes: { items: Array, unadmitted_count: number },
      services: { items: Array, no_endpoints_count: number },
      endpoints: { items: Array, not_ready_count: number },
      ingress_controllers: { items: Array, degraded_count: number }
    },
    storage: {
      pvcs: { items: Array, pending_count: number },
      pvs: { items: Array, unbound_count: number },
      storage_classes: { items: Array },
      volume_attachments: { items: Array, failed_count: number }
    }
  },
  platform: {
    fusion_detected: boolean,
    odf_detected: boolean,
    service_mesh_detected: boolean,
    acm_detected: boolean,
    virtualization_detected: boolean,
    cpd_detected: boolean
  }
}
```

## Navigation Flow

1. **Default View**: Dashboard (overview of cluster health)
2. **Hash-based Routing**: Uses `#section-id` format
3. **Collapsible Sections**: Sidebar sections can be expanded/collapsed
4. **Issue Badges**: Red badges show count of resources with issues
5. **Keyboard Shortcuts**: Still supported via `?` key

## Features

### Issue Count Calculation
- **Workloads**: Sum of unavailable deployments, statefulsets, misscheduled daemonsets, failed jobs, crashloop pods, and pending pods
- **Networking**: Sum of unadmitted routes, endpoints not ready, and degraded ingress controllers
- **Storage**: Sum of pending PVCs, unbound PVs, and failed volume attachments

### Dynamic Platform Detection
Platform sections only appear in the sidebar when detected:
- IBM Spectrum Fusion
- OpenShift Data Foundation (ODF)
- Service Mesh
- Advanced Cluster Management (ACM)
- Virtualization
- Cloud Pak for Data (CPD)

### Responsive Design
- Grid layouts for cards (1/2/3/4 columns based on screen size)
- Horizontal scrolling tabs for resource types
- Mobile-friendly collapsible sections

## Testing

### Build Test
```bash
cd frontend && npm run build
```
**Result**: ✓ Built successfully without errors

### Components Verified
- ✓ Dashboard renders with health cards
- ✓ Sidebar shows collapsible triage sections
- ✓ Workloads component with tabs
- ✓ Networking component with tabs
- ✓ Storage component with tabs
- ✓ Navigation between sections works
- ✓ Issue badges display correctly

## Integration with Backend

The frontend expects the backend (Rust) to generate HTML with embedded JSON in the following format:

```html
<script id="must-gather-data" type="application/json">
{
  "overview": { ... },
  "core": { ... },
  "infrastructure": { ... },
  "platform": { ... }
}
</script>
```

This matches the triage structure generated by the backend analyzers.

## Next Steps

1. **Backend Integration**: Ensure Rust backend generates triage-structured JSON
2. **Enhanced Resource Details**: Add detailed views for individual resources
3. **Platform-Specific Views**: Implement detailed views for detected platforms
4. **Search & Filters**: Re-implement search and filtering for triage views
5. **Real Data Testing**: Test with actual must-gather data

## Files Modified

### Created
- `frontend/src/components/Dashboard.jsx`
- `frontend/src/components/Workloads.jsx`
- `frontend/src/components/Networking.jsx`
- `frontend/src/components/Storage.jsx`

### Updated
- `frontend/src/components/Sidebar.jsx`
- `frontend/src/App.jsx`
- `frontend/src/main.jsx`
- `frontend/src/components/index.js`

## Success Criteria Met

✅ Sidebar displays triage-based sections with collapsible groups
✅ Issue count badges show on sections with problems
✅ Dashboard displays cluster health summary
✅ Workloads view combines all workload types with tabs
✅ Infrastructure views (Networking, Storage) work correctly
✅ Platform sections only appear when detected
✅ All navigation works smoothly
✅ UI is responsive and follows dark theme
✅ Build completes without errors

## Notes

- Reused existing components (Card, Button, StatusBadge, Tabs)
- Followed existing Tailwind CSS patterns
- Maintained dark theme consistency
- Used existing hooks (useHashRouter, useKeyboardShortcuts)
- Ready for integration with real must-gather data from backend

---

**Implementation Status**: ✅ Complete
**Build Status**: ✅ Passing
**Ready for Backend Integration**: ✅ Yes