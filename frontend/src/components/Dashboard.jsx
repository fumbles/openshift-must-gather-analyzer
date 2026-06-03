import React from 'react'
import { Card } from './Card'

function HealthCard({ title, total, healthy, issues, version, platform, icon }) {
  if (version !== undefined) {
    return (
      <Card className="text-center">
        <div className="text-3xl mb-2">{icon}</div>
        <div className="text-sm text-slate-400 mb-1">{title}</div>
        <div className="text-xl font-semibold text-white">{version}</div>
        {platform && <div className="text-xs text-slate-500 mt-1">{platform}</div>}
      </Card>
    )
  }

  const healthyCount = healthy !== undefined ? healthy : (total - (issues || 0))
  const percentage = total > 0 ? Math.round((healthyCount / total) * 100) : 0
  const isHealthy = percentage === 100
  const hasIssues = issues > 0 || percentage < 100

  return (
    <Card className="text-center">
      <div className="text-3xl mb-2">{icon}</div>
      <div className="text-sm text-slate-400 mb-1">{title}</div>
      <div className="text-2xl font-semibold text-white mb-1">
        {healthyCount} / {total}
      </div>
      <div className={`text-xs ${isHealthy ? 'text-green-400' : hasIssues ? 'text-red-400' : 'text-amber-400'}`}>
        {percentage}% Healthy
      </div>
    </Card>
  )
}

function PlatformBadge({ name }) {
  return (
    <span className="inline-flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-slate-300">
      <span className="h-2 w-2 rounded-full bg-green-400"></span>
      {name}
    </span>
  )
}

function IssuesSummary({ data }) {
  const issues = []

  // Collect issues from different sections
  if (data?.core?.nodes?.not_ready_count > 0) {
    issues.push({
      section: 'Nodes',
      count: data.core.nodes.not_ready_count,
      message: `${data.core.nodes.not_ready_count} node(s) not ready`,
      severity: 'error'
    })
  }

  if (data?.core?.workloads?.pods?.crashloop_count > 0) {
    issues.push({
      section: 'Pods',
      count: data.core.workloads.pods.crashloop_count,
      message: `${data.core.workloads.pods.crashloop_count} pod(s) in CrashLoopBackOff`,
      severity: 'error'
    })
  }

  if (data?.core?.workloads?.pods?.pending_count > 0) {
    issues.push({
      section: 'Pods',
      count: data.core.workloads.pods.pending_count,
      message: `${data.core.workloads.pods.pending_count} pod(s) pending`,
      severity: 'warning'
    })
  }

  if (data?.core?.workloads?.deployments?.unavailable_count > 0) {
    issues.push({
      section: 'Deployments',
      count: data.core.workloads.deployments.unavailable_count,
      message: `${data.core.workloads.deployments.unavailable_count} deployment(s) unavailable`,
      severity: 'error'
    })
  }

  if (data?.infrastructure?.storage?.pvcs?.pending_count > 0) {
    issues.push({
      section: 'Storage',
      count: data.infrastructure.storage.pvcs.pending_count,
      message: `${data.infrastructure.storage.pvcs.pending_count} PVC(s) pending`,
      severity: 'warning'
    })
  }

  if (data?.overview?.cluster_health?.degraded_count > 0) {
    issues.push({
      section: 'Cluster Operators',
      count: data.overview.cluster_health.degraded_count,
      message: `${data.overview.cluster_health.degraded_count} operator(s) degraded`,
      severity: 'error'
    })
  }

  if (issues.length === 0) {
    return (
      <div className="flex items-center justify-center py-8 text-green-400">
        <span className="text-2xl mr-2">✓</span>
        <span>No critical issues detected</span>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {issues.map((issue, idx) => (
        <div
          key={idx}
          className={`flex items-center justify-between rounded-lg border p-3 ${
            issue.severity === 'error'
              ? 'border-red-500/20 bg-red-500/5'
              : 'border-amber-500/20 bg-amber-500/5'
          }`}
        >
          <div className="flex items-center gap-3">
            <span className={`text-xl ${issue.severity === 'error' ? 'text-red-400' : 'text-amber-400'}`}>
              {issue.severity === 'error' ? '⚠️' : '⚡'}
            </span>
            <div>
              <div className="text-sm font-medium text-white">{issue.section}</div>
              <div className="text-xs text-slate-400">{issue.message}</div>
            </div>
          </div>
          <span className={`rounded-full px-2 py-1 text-xs font-semibold ${
            issue.severity === 'error'
              ? 'bg-red-500/20 text-red-400'
              : 'bg-amber-500/20 text-amber-400'
          }`}>
            {issue.count}
          </span>
        </div>
      ))}
    </div>
  )
}

export function Dashboard({ data }) {
  const dashboard = data?.overview?.dashboard || {}
  const platform = data?.platform || {}

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-white">Cluster Dashboard</h1>

      {/* Health Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <HealthCard
          title="Nodes"
          total={dashboard.total_nodes || 0}
          healthy={dashboard.healthy_nodes || 0}
          icon="🖥️"
        />
        <HealthCard
          title="Pods"
          total={dashboard.total_pods || 0}
          healthy={dashboard.healthy_pods || 0}
          icon="📦"
        />
        <HealthCard
          title="Operators"
          total={dashboard.total_operators || 0}
          issues={dashboard.degraded_operators || 0}
          icon="⚙️"
        />
        <HealthCard
          title="Cluster Version"
          version={dashboard.cluster_version || 'Unknown'}
          platform={dashboard.platform_type || 'OpenShift'}
          icon="🏷️"
        />
      </div>

      {/* Platform Detection */}
      {Object.values(platform).some(v => v) && (
        <Card title="Detected Platforms">
          <div className="flex flex-wrap gap-2">
            {platform.fusion_detected && <PlatformBadge name="IBM Spectrum Fusion" />}
            {platform.odf_detected && <PlatformBadge name="OpenShift Data Foundation" />}
            {platform.service_mesh_detected && <PlatformBadge name="Service Mesh" />}
            {platform.acm_detected && <PlatformBadge name="Advanced Cluster Management" />}
            {platform.virtualization_detected && <PlatformBadge name="Virtualization" />}
            {platform.cpd_detected && <PlatformBadge name="Cloud Pak for Data" />}
          </div>
        </Card>
      )}

      {/* Quick Issues Summary */}
      <Card title="Issues Requiring Attention">
        <IssuesSummary data={data} />
      </Card>
    </div>
  )
}
