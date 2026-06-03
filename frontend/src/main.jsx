import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

function normalizeResource(resource) {
  if (!resource || typeof resource !== 'object') return resource

  return {
    ...resource,
    errors: Array.isArray(resource.errors) ? resource.errors : [],
    warnings: Array.isArray(resource.warnings) ? resource.warnings : [],
    logs: Array.isArray(resource.logs) ? resource.logs : [],
    labels: resource.labels || {},
    annotations: resource.annotations || {},
    owner_references: Array.isArray(resource.owner_references) ? resource.owner_references : [],
    relationships: Array.isArray(resource.relationships) ? resource.relationships : [],
    key_fields: resource.key_fields || {},
    metadata: resource.metadata || {},
  }
}

function normalizeData(value) {
  if (Array.isArray(value)) {
    const looksLikeResources = value.every(
      (item) => item && typeof item === 'object' && typeof item.name === 'string' && typeof item.kind === 'string'
    )
    if (looksLikeResources) {
      return value.map(normalizeResource)
    }
    return value.map(normalizeData)
  }

  if (!value || typeof value !== 'object') {
    return value
  }

  const normalized = {}
  for (const [key, child] of Object.entries(value)) {
    normalized[key] = normalizeData(child)
  }
  return normalized
}

// Read must-gather data from the DOM or a preloaded site summary script
function getMustGatherData() {
  if (globalThis.__MGA_DATA__) {
    return normalizeData(globalThis.__MGA_DATA__)
  }

  const dataElement = document.getElementById('must-gather-data')
  if (dataElement) {
    try {
      return normalizeData(JSON.parse(dataElement.textContent))
    } catch (e) {
      console.error('Failed to parse must-gather data:', e)
    }
  }

  // Return mock triage-structured data for development
  return {
    overview: {
      dashboard: {
        cluster_name: "OpenShift Cluster",
        cluster_version: "4.14.0",
        platform_type: "AWS",
        cluster_status: "warning",
        total_nodes: 3,
        healthy_nodes: 2,
        total_pods: 150,
        healthy_pods: 145,
        total_operators: 30,
        degraded_operators: 1
      },
      cluster_health: {
        items: [],
        degraded_count: 1
      }
    },
    core: {
      nodes: {
        items: [
          {
            name: 'ip-10-0-0-1.control.plane',
            status: 'Ready',
            roles: ['control-plane', 'master'],
            version: 'v1.27.0',
            os: 'Red Hat Enterprise Linux CoreOS',
            kernel: '5.14.0',
            errors: [],
            warnings: []
          },
          {
            name: 'ip-10-0-0-2.control.plane',
            status: 'NotReady',
            roles: ['control-plane', 'master'],
            version: 'v1.27.0',
            os: 'Red Hat Enterprise Linux CoreOS',
            kernel: '5.14.0',
            errors: ['Node is not Ready'],
            warnings: []
          }
        ],
        not_ready_count: 1
      },
      workloads: {
        deployments: { items: [], unavailable_count: 0 },
        statefulsets: { items: [], unavailable_count: 0 },
        daemonsets: { items: [], misscheduled_count: 0 },
        jobs: { items: [], failed_count: 0 },
        cronjobs: { items: [], suspended_count: 0 },
        replicasets: { items: [], unavailable_count: 0 },
        pods: { items: [], crashloop_count: 0, pending_count: 0 }
      },
      namespaces: {
        items: []
      }
    },
    infrastructure: {
      networking: {
        routes: { items: [], unadmitted_count: 0 },
        services: { items: [], no_endpoints_count: 0 },
        endpoints: { items: [], not_ready_count: 0 },
        ingress_controllers: { items: [], degraded_count: 0 }
      },
      storage: {
        pvcs: { items: [], pending_count: 0 },
        pvs: { items: [], unbound_count: 0 },
        storage_classes: { items: [] },
        volume_attachments: { items: [], failed_count: 0 }
      }
    },
    platform: {
      fusion_detected: false,
      odf_detected: false,
      service_mesh_detected: false,
      acm_detected: false,
      virtualization_detected: false,
      cpd_detected: false,
      virtualization: {
        hyperconvergeds: { items: [] },
        kubevirts: { items: [] },
        virtual_machines: { items: [] },
        virtual_machine_instances: { items: [] },
        virtual_machine_pools: { items: [] },
        virtual_machine_exports: { items: [] },
        virtual_machine_clones: { items: [] },
        virtual_machine_snapshots: { items: [] },
        virtual_machine_snapshot_contents: { items: [] },
        virtual_machine_restores: { items: [] },
        data_volumes: { items: [] },
        data_sources: { items: [] },
        data_import_crons: { items: [] },
        instance_types: { items: [] },
        preferences: { items: [] },
      }
    }
  }
}

const mustGatherData = getMustGatherData()

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App data={mustGatherData} />
  </React.StrictMode>,
)
