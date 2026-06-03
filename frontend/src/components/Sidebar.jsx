import React, { useState } from 'react'
import { FilterControls } from './FilterControls'

function SectionIcon({ kind, className = 'h-4 w-4' }) {
  const common = { className, fill: 'none', stroke: 'currentColor', strokeWidth: 1.8, viewBox: '0 0 24 24' }

  switch (kind) {
    case 'home':
      return (
        <svg {...common}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M3 11.5 12 4l9 7.5M6.5 10.5V20h11V10.5" />
        </svg>
      )
    case 'workloads':
      return (
        <svg {...common}>
          <rect x="4" y="5" width="7" height="6" rx="1.5" />
          <rect x="13" y="5" width="7" height="6" rx="1.5" />
          <rect x="4" y="13" width="7" height="6" rx="1.5" />
          <rect x="13" y="13" width="7" height="6" rx="1.5" />
        </svg>
      )
    case 'networking':
      return (
        <svg {...common}>
          <circle cx="6" cy="12" r="2.5" />
          <circle cx="18" cy="7" r="2.5" />
          <circle cx="18" cy="17" r="2.5" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M8.5 11 15 8.2M8.5 13 15 15.8" />
        </svg>
      )
    case 'storage':
      return (
        <svg {...common}>
          <ellipse cx="12" cy="6.5" rx="6.5" ry="2.5" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M5.5 6.5v11c0 1.4 2.9 2.5 6.5 2.5s6.5-1.1 6.5-2.5v-11" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M5.5 12c0 1.4 2.9 2.5 6.5 2.5s6.5-1.1 6.5-2.5" />
        </svg>
      )
    case 'compute':
      return (
        <svg {...common}>
          <rect x="4" y="5" width="16" height="11" rx="2" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M9 19h6M12 16v3" />
        </svg>
      )
    case 'platform':
      return (
        <svg {...common}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M12 3 4.5 7.2v9.6L12 21l7.5-4.2V7.2L12 3Z" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M12 3v18M4.5 7.2 12 12l7.5-4.8" />
        </svg>
      )
    case 'security':
      return (
        <svg {...common}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M12 3 5 6v5c0 4.6 2.7 7.9 7 10 4.3-2.1 7-5.4 7-10V6l-7-3Z" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M9.5 12.2 11.3 14l3.4-3.6" />
        </svg>
      )
    case 'administration':
      return (
        <svg {...common}>
          <circle cx="12" cy="12" r="3" />
          <path strokeLinecap="round" strokeLinejoin="round" d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1" />
        </svg>
      )
    case 'observe':
      return (
        <svg {...common}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M2.5 12s3.6-6 9.5-6 9.5 6 9.5 6-3.6 6-9.5 6-9.5-6-9.5-6Z" />
          <circle cx="12" cy="12" r="3" />
        </svg>
      )
    default:
      return null
  }
}

export function Sidebar({ data, activeSection, onSectionClick, filters, onFilterChange, filterStats, theme = 'dark' }) {
  const [expandedSections, setExpandedSections] = useState(['home'])

  const toggleSection = (sectionId) => {
    setExpandedSections(prev =>
      prev.includes(sectionId)
        ? prev.filter(id => id !== sectionId)
        : [...prev, sectionId]
    )
  }

  // Helper functions to calculate issue counts
  const calculateWorkloadIssues = (workloads) => {
    if (!workloads) return 0
    return (
      (workloads.deployments?.unavailable_count || 0) +
      (workloads.statefulsets?.unavailable_count || 0) +
      (workloads.daemonsets?.misscheduled_count || 0) +
      (workloads.jobs?.failed_count || 0) +
      (workloads.pods?.crashloop_count || 0) +
      (workloads.pods?.pending_count || 0)
    )
  }

  const calculateNetworkingIssues = (networking) => {
    if (!networking) return 0
    return (
      (networking.routes?.unadmitted_count || 0) +
      (networking.endpoints?.not_ready_count || 0) +
      (networking.ingress_controllers?.degraded_count || 0) +
      (networking.network_policies?.warning_count || 0)
    )
  }

  const calculateStorageIssues = (storage) => {
    if (!storage) return 0
    return (
      (storage.pvcs?.pending_count || 0) +
      (storage.pvs?.unbound_count || 0) +
      (storage.volume_attachments?.failed_count || 0)
    )
  }

  const calculateSecurityIssues = (security) => {
    if (!security) return 0
    return security.security_context_constraints?.warning_count || 0
  }

  const calculateComputeIssues = (compute, core) => {
    return (
      (core?.nodes?.not_ready_count || 0) +
      (compute?.machines?.not_running_count || 0) +
      (compute?.machine_health_checks?.unhealthy_count || 0) +
      (compute?.machine_config_pools?.degraded_count || 0) +
      (compute?.machine_config_pools?.updating_count || 0) +
      (compute?.machine_configurations?.degraded_count || 0) +
      (compute?.machine_configurations?.progressing_count || 0)
    )
  }

  const getPlatformSections = (platform) => {
    if (!platform) return []
    const sections = []
    if (platform.fusion_detected) {
      sections.push({ id: 'fusion-main', title: 'IBM Spectrum Fusion', badge: null })
      sections.push({ id: 'fusion-storage-scale', title: 'Storage Scale', badge: null })
    }
    if (platform.odf_detected) sections.push({ id: 'odf', title: 'OpenShift Data Foundation', badge: null })
    if (platform.service_mesh_detected) sections.push({ id: 'service-mesh', title: 'Service Mesh', badge: null })
    if (platform.acm_detected) sections.push({ id: 'acm', title: 'Advanced Cluster Management', badge: null })
    if (platform.virtualization_detected) sections.push({ id: 'virtualization', title: 'Virtualization', badge: null })
    if (platform.cpd_detected) sections.push({ id: 'cpd', title: 'Cloud Pak for Data', badge: null })
    return sections
  }

  const sections = [
    {
      id: 'home',
      title: 'Home',
      icon: 'home',
      badge: null,
      subsections: [
        { id: 'dashboard', title: 'Overview', badge: null },
        { id: 'cluster-health', title: 'Cluster Health', badge: data?.overview?.cluster_health?.degraded_count || 0 },
        { id: 'cluster-operators', title: 'Cluster Operators', badge: data?.overview?.cluster_health?.degraded_count || 0 },
      ]
    },
    {
      id: 'observe-nav',
      title: 'Observe',
      icon: 'observe',
      badge: data?.core?.events?.warning_count || 0,
      subsections: [
        { id: 'events', title: 'Events', badge: data?.core?.events?.warning_count || 0 },
      ]
    },
    {
      id: 'compute-nav',
      title: 'Compute',
      icon: 'compute',
      badge: calculateComputeIssues(data?.compute, data?.core),
      subsections: [
        { id: 'nodes', title: 'Nodes', badge: data?.core?.nodes?.not_ready_count || 0 },
        { id: 'compute-machines', title: 'Machines', badge: data?.compute?.machines?.not_running_count || 0 },
        { id: 'compute-machineautoscalers', title: 'MachineAutoscalers', badge: null },
        { id: 'compute-machinehealthchecks', title: 'MachineHealthChecks', badge: data?.compute?.machine_health_checks?.unhealthy_count || 0 },
        { id: '__divider_compute_primary', title: 'divider', divider: true },
        { id: 'compute-controlplanemachinesets', title: 'ControlPlaneMachineSets', badge: null },
        { id: 'compute-machinesets', title: 'MachineSets', badge: null },
        { id: '__divider_compute_secondary', title: 'divider', divider: true },
        { id: 'compute-machineconfigs', title: 'MachineConfigs', badge: null },
        { id: 'compute-machineconfigpools', title: 'MachineConfigPools', badge: data?.compute?.machine_config_pools?.degraded_count || data?.compute?.machine_config_pools?.updating_count || 0 },
        { id: 'compute-machineconfigurations', title: 'MachineConfigurations', badge: data?.compute?.machine_configurations?.degraded_count || data?.compute?.machine_configurations?.progressing_count || 0 },
      ]
    },
    {
      id: 'workloads-nav',
      title: 'Workloads',
      icon: 'workloads',
      badge: calculateWorkloadIssues(data?.core?.workloads),
      subsections: [
        { id: 'workloads-topology', title: 'Topology', badge: calculateWorkloadIssues(data?.core?.workloads) },
        { id: 'workloads-pods', title: 'Pods', badge: (data?.core?.workloads?.pods?.crashloop_count || 0) + (data?.core?.workloads?.pods?.pending_count || 0) },
        { id: 'workloads-deployments', title: 'Deployments', badge: data?.core?.workloads?.deployments?.unavailable_count || 0 },
        { id: 'workloads-statefulsets', title: 'StatefulSets', badge: data?.core?.workloads?.statefulsets?.unavailable_count || 0 },
        { id: 'workloads-secrets', title: 'Secrets', badge: null },
        { id: 'workloads-configmaps', title: 'ConfigMaps', badge: null },
        { id: '__divider_workloads_primary', title: 'divider', divider: true },
        { id: 'workloads-cronjobs', title: 'CronJobs', badge: null },
        { id: 'workloads-jobs', title: 'Jobs', badge: data?.core?.workloads?.jobs?.failed_count || 0 },
        { id: 'workloads-daemonsets', title: 'DaemonSets', badge: data?.core?.workloads?.daemonsets?.misscheduled_count || 0 },
        { id: 'workloads-replicasets', title: 'ReplicaSets', badge: null },
      ]
    },
    {
      id: 'networking-nav',
      title: 'Networking',
      icon: 'networking',
      badge: calculateNetworkingIssues(data?.infrastructure?.networking),
      subsections: [
        { id: 'networking-services', title: 'Services', badge: data?.infrastructure?.networking?.services?.no_endpoints_count || 0 },
        { id: 'networking-routes', title: 'Routes', badge: data?.infrastructure?.networking?.routes?.unadmitted_count || 0 },
        { id: 'networking-endpoints', title: 'Endpoints', badge: data?.infrastructure?.networking?.endpoints?.not_ready_count || 0 },
        { id: 'networking-networkpolicies', title: 'NetworkPolicies', badge: data?.infrastructure?.networking?.network_policies?.warning_count || 0 },
        { id: 'networking-ingresscontrollers', title: 'IngressControllers', badge: data?.infrastructure?.networking?.ingress_controllers?.degraded_count || 0 },
      ]
    },
    {
      id: 'security-nav',
      title: 'Security',
      icon: 'security',
      badge: calculateSecurityIssues(data?.security),
      subsections: [
        { id: 'security-clusterroles', title: 'Cluster Roles', badge: null },
        { id: 'security-clusterrolebindings', title: 'Cluster Role Bindings', badge: null },
        { id: 'security-securitycontextconstraints', title: 'Security Context Constraints', badge: data?.security?.security_context_constraints?.warning_count || 0 },
      ]
    },
    {
      id: 'storage-nav',
      title: 'Storage',
      icon: 'storage',
      badge: calculateStorageIssues(data?.infrastructure?.storage),
      subsections: [
        { id: 'storage-pvcs', title: 'PersistentVolumeClaims', badge: data?.infrastructure?.storage?.pvcs?.pending_count || 0 },
        { id: 'storage-pvs', title: 'PersistentVolumes', badge: data?.infrastructure?.storage?.pvs?.unbound_count || 0 },
        { id: 'storage-storage_classes', title: 'StorageClasses', badge: null },
        { id: 'storage-volume_attachments', title: 'VolumeAttachments', badge: data?.infrastructure?.storage?.volume_attachments?.failed_count || 0 },
      ]
    },
    {
      id: 'administration-nav',
      title: 'Administration',
      icon: 'administration',
      badge: null,
      subsections: [
        { id: 'administration-cluster-settings', title: 'Cluster Settings', badge: null },
        { id: 'administration-namespaces', title: 'Namespaces', badge: null },
        { id: 'administration-resourcequotas', title: 'ResourceQuotas', badge: null },
        { id: 'administration-limitranges', title: 'LimitRanges', badge: null },
        { id: 'administration-customresourcedefinitions', title: 'CustomResourceDefinitions', badge: null },
        { id: 'administration-dynamicplugins', title: 'Dynamic Plugins', badge: null },
      ]
    },
    {
      id: 'platform',
      title: 'Platform',
      icon: 'platform',
      badge: null,
      subsections: getPlatformSections(data?.platform)
    }
  ]

  return (
    <aside className="h-full min-h-0 w-64 overflow-y-auto border-r border-slate-800 bg-slate-950">
      <div className="p-4">
        {/* Filters */}
        {filterStats && (
          <div className="mb-6">
            <FilterControls
              filters={filters}
              onFilterChange={onFilterChange}
              stats={filterStats}
            />
          </div>
        )}

        {/* Triage sections */}
        <nav className="space-y-2">
          {sections.map((section) => (
            <div key={section.id} className="space-y-1">
              {/* Section header */}
              <button
                onClick={() => toggleSection(section.id)}
                className={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm font-semibold transition-colors ${
                  theme === 'light'
                    ? 'text-slate-700 hover:bg-slate-100'
                    : 'text-slate-300 hover:bg-slate-800'
                }`}
              >
                <span className="flex items-center gap-2">
                  <SectionIcon kind={section.icon} />
                  <span>{section.title}</span>
                </span>
                <span className="flex items-center gap-2">
                  {section.badge !== null && section.badge > 0 && (
                    <span className={`rounded-full px-2 py-0.5 text-xs font-semibold ${
                      theme === 'light'
                        ? 'bg-red-100 text-red-600'
                        : 'bg-red-500/20 text-red-400'
                    }`}>
                      {section.badge}
                    </span>
                  )}
                  <span className={`transition-transform ${expandedSections.includes(section.id) ? 'rotate-90' : ''}`}>
                    ▶
                  </span>
                </span>
              </button>

              {/* Subsections */}
              {expandedSections.includes(section.id) && (
                <div className="ml-3 space-y-1.5">
                  {section.subsections.map((subsection) =>
                    subsection.divider ? (
                      <div
                        key={subsection.id}
                        className={`mx-5 my-3 border-t ${theme === 'light' ? 'border-slate-200' : 'border-slate-800'}`}
                      />
                    ) : (
                      <button
                        key={subsection.id}
                        onClick={() => onSectionClick && onSectionClick({ id: subsection.id })}
                        className={`flex w-full items-center justify-between rounded-lg px-4 py-2 text-left text-sm transition-colors ${
                          activeSection === subsection.id
                            ? theme === 'light'
                              ? 'border border-blue-200 bg-blue-50 text-blue-700'
                              : 'bg-red-500/10 text-red-400'
                            : theme === 'light'
                              ? 'text-slate-700 hover:bg-slate-100'
                              : 'text-slate-300 hover:bg-slate-800'
                        }`}
                      >
                        <span className="pl-3">{subsection.title}</span>
                        {subsection.badge !== null && subsection.badge > 0 && (
                          <span className={`rounded-full px-2 py-0.5 text-xs ${
                            theme === 'light'
                              ? 'bg-red-100 text-red-600'
                              : 'bg-red-500/20 text-red-400'
                          }`}>
                            {subsection.badge}
                          </span>
                        )}
                      </button>
                    )
                  )}
                </div>
              )}
            </div>
          ))}
        </nav>
      </div>
    </aside>
  )
}
