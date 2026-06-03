import React, { useEffect, useMemo, useState } from 'react'
import { Card } from './Card'
import { ProjectSelector } from './ProjectSelector'
import { StatusBadge } from './StatusBadge'
import { Tabs } from './Tabs'
import { YAMLViewer } from './YAMLViewer'
import { HealthAnalysis } from './HealthAnalysis'
import { useResourceDetail, useResourceRaw } from '../hooks/useResourceDetail'

function StorageTabs({ tabs, activeTab, onTabChange, theme = 'dark' }) {
  return (
    <div className="flex flex-wrap gap-x-2 gap-y-1 border-b border-slate-800 pb-2">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap ${
            activeTab === tab.id
              ? theme === 'light'
                ? 'border-b-2 border-blue-600 text-slate-900'
                : 'border-b-2 border-red-500 text-white'
              : theme === 'light'
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          <span>{tab.label}</span>
          <span className="text-xs text-slate-500">({tab.count})</span>
          {tab.issues > 0 && (
            <span className={`rounded-full px-2 py-0.5 text-xs ${
              theme === 'light'
                ? 'bg-red-100 text-red-600'
                : 'bg-red-500/20 text-red-400'
            }`}>
              {tab.issues}
            </span>
          )}
        </button>
      ))}
    </div>
  )
}

function ResourceFilterBar({ searchTerm, onSearchChange, statusFilter, onStatusFilterChange, resourceCount }) {
  const statusOptions = [
    { value: 'all', label: 'All' },
    { value: 'error', label: 'Errors' },
    { value: 'warning', label: 'Warnings' },
    { value: 'healthy', label: 'Healthy' },
  ]

  return (
    <Card className="mb-2 px-3 py-2.5">
      <div className="space-y-2">
        <div className="relative">
          <div className="pointer-events-none absolute inset-y-0 left-3 flex items-center text-slate-400">
            🔍
          </div>
          <input
            type="text"
            value={searchTerm}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search resources by name..."
            className="w-full rounded-lg border border-slate-700 bg-slate-900 py-1.5 pl-10 pr-10 text-sm text-white placeholder-slate-500 focus:border-red-500 focus:outline-none focus:ring-1 focus:ring-red-500"
          />
          {searchTerm && (
            <button
              onClick={() => onSearchChange('')}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white"
              title="Clear search"
            >
              ✕
            </button>
          )}
        </div>

        <div className="flex flex-wrap gap-1.5">
          {statusOptions.map((option) => (
            <button
              key={option.value}
              onClick={() => onStatusFilterChange(option.value)}
              className={`rounded-lg px-2 py-1 text-xs font-medium whitespace-nowrap transition-colors ${
                statusFilter === option.value
                  ? option.value === 'error'
                    ? 'bg-red-500/20 text-red-400 border border-red-500/50'
                    : option.value === 'warning'
                      ? 'bg-amber-500/20 text-amber-400 border border-amber-500/50'
                      : option.value === 'healthy'
                        ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/50'
                        : 'bg-slate-700 text-white border border-slate-600'
                  : 'text-slate-400 hover:text-white hover:bg-slate-800'
              }`}
            >
              {option.label}
            </button>
          ))}
          <div className="ml-auto text-xs text-slate-400 self-center">
            Showing {resourceCount} resource{resourceCount === 1 ? '' : 's'}
          </div>
        </div>
      </div>
    </Card>
  )
}

function isSameResource(a, b) {
  return !!a && !!b && a.name === b.name && a.namespace === b.namespace && a.kind === b.kind
}

function dedupeResources(resources) {
  const seen = new Set()
  return resources.filter((resource) => {
    if (!resource) return false
    const key = resource.uid || `${resource.kind}:${resource.namespace || ''}:${resource.name}`
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
}

function getStorageCollectionItems(storage, key) {
  return storage?.[key]?.items || []
}

function getKeyField(resource, key) {
  return resource?.key_fields?.[key] || null
}

function getAllWorkloadResources(data) {
  return Object.values(data?.core?.workloads || {}).flatMap((collection) => collection?.items || [])
}

function getReferencedByResources(resource, data) {
  return dedupeResources(
    getAllWorkloadResources(data).filter((candidate) =>
      (candidate.relationships || []).some(
        (relationship) =>
          relationship.relationship === 'References' &&
          relationship.kind === resource.kind &&
          relationship.name === resource.name &&
          (relationship.namespace || candidate.namespace || null) === (resource.namespace || null)
      )
    )
  )
}

function getUpstreamOwners(resource, data) {
  const ownerRefs = resource?.owner_references || []
  if (!ownerRefs.length) return []

  const collections = getAllWorkloadResources(data)

  return ownerRefs
    .map((ownerRef) =>
      collections.find(
        (candidate) =>
          candidate.kind === ownerRef.kind &&
          candidate.namespace === resource.namespace &&
          ((ownerRef.uid && candidate.uid === ownerRef.uid) || candidate.name === ownerRef.name)
      )
    )
    .filter(Boolean)
}

function getResourceWithAncestors(resource, data, seen = new Set()) {
  if (!resource) return []
  const key = resource.uid || `${resource.kind}:${resource.namespace || ''}:${resource.name}`
  if (seen.has(key)) return []
  seen.add(key)

  const owners = getUpstreamOwners(resource, data)
  return [resource, ...owners.flatMap((owner) => getResourceWithAncestors(owner, data, seen))]
}

function getControllerResourcesForPods(pods, data) {
  return dedupeResources(
    pods.flatMap((pod) => getResourceWithAncestors(pod, data)).filter((resource) => resource.kind !== 'Pod')
  )
}

function getStorageRelatedSections(resource, data) {
  const storage = data?.infrastructure?.storage || {}
  const nodes = data?.core?.nodes?.items || []
  const pvcs = getStorageCollectionItems(storage, 'pvcs')
  const pvs = getStorageCollectionItems(storage, 'pvs')
  const storageClasses = getStorageCollectionItems(storage, 'storage_classes')
  const volumeAttachments = getStorageCollectionItems(storage, 'volume_attachments')

  const sections = []

  switch (resource?.kind) {
    case 'PersistentVolumeClaim': {
      const pv = pvs.find(
        (candidate) =>
          candidate.name === getKeyField(resource, 'volume_name') ||
          getKeyField(candidate, 'claim_ref') === `${resource.namespace}/${resource.name}`
      )
      const storageClass = storageClasses.find(
        (candidate) => candidate.name === getKeyField(resource, 'storage_class')
      )
      const attachments = pv
        ? volumeAttachments.filter((candidate) => getKeyField(candidate, 'pv_name') === pv.name)
        : []
      const consumers = getReferencedByResources(resource, data)
      const consumerPods = dedupeResources(consumers.filter((candidate) => candidate.kind === 'Pod'))
      const consumerControllers = dedupeResources([
        ...consumers.filter((candidate) => candidate.kind !== 'Pod'),
        ...getControllerResourcesForPods(consumerPods, data),
      ])
      sections.push(
        { title: 'Persistent Volumes', resources: dedupeResources(pv ? [pv] : []) },
        { title: 'Storage Classes', resources: dedupeResources(storageClass ? [storageClass] : []) },
        { title: 'Volume Attachments', resources: dedupeResources(attachments) },
        { title: 'Workload Controllers', resources: consumerControllers },
        { title: 'Pods', resources: consumerPods },
      )
      break
    }
    case 'PersistentVolume': {
      const claimRef = getKeyField(resource, 'claim_ref')
      const pvc = claimRef
        ? pvcs.find((candidate) => `${candidate.namespace}/${candidate.name}` === claimRef)
        : null
      const storageClass = storageClasses.find(
        (candidate) => candidate.name === getKeyField(resource, 'storage_class')
      )
      const attachments = volumeAttachments.filter(
        (candidate) => getKeyField(candidate, 'pv_name') === resource.name
      )
      const consumers = pvc ? getReferencedByResources(pvc, data) : []
      const consumerPods = dedupeResources(consumers.filter((candidate) => candidate.kind === 'Pod'))
      const consumerControllers = dedupeResources([
        ...consumers.filter((candidate) => candidate.kind !== 'Pod'),
        ...getControllerResourcesForPods(consumerPods, data),
      ])
      sections.push(
        { title: 'Persistent Volume Claims', resources: dedupeResources(pvc ? [pvc] : []) },
        { title: 'Storage Classes', resources: dedupeResources(storageClass ? [storageClass] : []) },
        { title: 'Volume Attachments', resources: dedupeResources(attachments) },
        { title: 'Workload Controllers', resources: consumerControllers },
        { title: 'Pods', resources: consumerPods },
      )
      break
    }
    case 'StorageClass': {
      const relatedPvcs = pvcs.filter(
        (candidate) => getKeyField(candidate, 'storage_class') === resource.name
      )
      const relatedPvs = pvs.filter(
        (candidate) => getKeyField(candidate, 'storage_class') === resource.name
      )
      const relatedPods = dedupeResources(
        relatedPvcs.flatMap((pvc) => getReferencedByResources(pvc, data)).filter((candidate) => candidate.kind === 'Pod')
      )
      const relatedControllers = dedupeResources([
        ...relatedPvcs.flatMap((pvc) =>
          getReferencedByResources(pvc, data).filter((candidate) => candidate.kind !== 'Pod')
        ),
        ...getControllerResourcesForPods(relatedPods, data),
      ])
      sections.push(
        { title: 'Persistent Volume Claims', resources: dedupeResources(relatedPvcs) },
        { title: 'Persistent Volumes', resources: dedupeResources(relatedPvs) },
        { title: 'Workload Controllers', resources: relatedControllers },
        { title: 'Pods', resources: relatedPods },
      )
      break
    }
    case 'VolumeAttachment': {
      const pv = pvs.find((candidate) => candidate.name === getKeyField(resource, 'pv_name'))
      const pvc = pv
        ? pvcs.find(
            (candidate) =>
              `${candidate.namespace}/${candidate.name}` === getKeyField(pv, 'claim_ref')
          )
        : null
      const node = nodes.find((candidate) => candidate.name === getKeyField(resource, 'node_name'))
      const consumers = pvc ? getReferencedByResources(pvc, data) : []
      const consumerPods = dedupeResources(consumers.filter((candidate) => candidate.kind === 'Pod'))
      const consumerControllers = dedupeResources([
        ...consumers.filter((candidate) => candidate.kind !== 'Pod'),
        ...getControllerResourcesForPods(consumerPods, data),
      ])
      sections.push(
        { title: 'Persistent Volumes', resources: dedupeResources(pv ? [pv] : []) },
        { title: 'Persistent Volume Claims', resources: dedupeResources(pvc ? [pvc] : []) },
        { title: 'Nodes', resources: dedupeResources(node ? [node] : []) },
        { title: 'Workload Controllers', resources: consumerControllers },
        { title: 'Pods', resources: consumerPods },
      )
      break
    }
    default:
      break
  }

  return sections.filter((section) => section.resources.length > 0)
}

function getRelatedSectionVariant(title) {
  if (title.includes('Node')) return 'owner'
  if (title.includes('Storage Class')) return 'dependency'
  if (title.includes('Volume Attachment')) return 'network'
  return 'child'
}

function RelatedResourceButton({ resource, onOpen, variant = 'child' }) {
  const variantClasses = {
    owner: 'border-sky-500/30 bg-sky-500/10 text-sky-100 hover:border-sky-400 hover:bg-sky-500/20',
    child: 'border-violet-500/25 bg-violet-500/10 text-violet-100 hover:border-violet-400 hover:bg-violet-500/20',
    dependency: 'border-amber-500/30 bg-amber-500/10 text-amber-100 hover:border-amber-400 hover:bg-amber-500/20',
    network: 'border-cyan-500/30 bg-cyan-500/10 text-cyan-100 hover:border-cyan-400 hover:bg-cyan-500/20',
  }

  return (
    <button
      onClick={() => onOpen?.(resource)}
      disabled={!onOpen}
      className={`inline-flex max-w-full items-center gap-2 rounded-lg border px-3 py-2 text-left text-sm transition-colors ${variantClasses[variant] || variantClasses.child}`}
    >
      <span className="min-w-0 break-words font-medium">{resource.name}</span>
      <span className="text-xs opacity-75">{resource.kind}</span>
      <span className="text-xs opacity-60">{resource.status}</span>
    </button>
  )
}

function RelatedResourcesTab({ resource, data, onOpenResource }) {
  const relatedSections = getStorageRelatedSections(resource, data)
  const totalRelated = relatedSections.reduce((sum, section) => sum + section.resources.length, 0)

  if (!relatedSections.length) {
    return (
      <Card>
        <p className="text-slate-400">No related resources available.</p>
      </Card>
    )
  }

  return (
    <Card>
      <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-slate-400">
        Related Resources ({totalRelated})
      </h3>
      <div className="space-y-5">
        {relatedSections.map((section) => (
          <div key={section.title} className="space-y-3">
            <div className="text-xs font-semibold uppercase tracking-wider text-slate-500">
              {section.title} ({section.resources.length})
            </div>
            <div className="flex flex-wrap gap-2">
              {section.resources.map((related) => (
                <RelatedResourceButton
                  key={related.uid || `${related.kind}-${related.namespace || ''}-${related.name}`}
                  resource={related}
                  onOpen={onOpenResource}
                  variant={getRelatedSectionVariant(section.title)}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
    </Card>
  )
}

function applySearchFilter(resources, searchTerm) {
  if (!searchTerm) return resources
  const search = searchTerm.toLowerCase()
  return resources.filter((resource) => {
    const namespace = resource.namespace || ''
    return resource.name.toLowerCase().includes(search) || namespace.toLowerCase().includes(search)
  })
}

function applyStatusFilter(resources, statusFilter) {
  if (statusFilter === 'all') return resources
  return resources.filter((resource) => {
    const errors = resource.errors?.length || 0
    const warnings = resource.warnings?.length || 0
    if (statusFilter === 'error') return errors > 0 || resource.status === 'Error'
    if (statusFilter === 'warning') return (warnings > 0 && errors === 0) || resource.status === 'Warning'
    if (statusFilter === 'healthy') return errors === 0 && warnings === 0 && resource.status !== 'Error' && resource.status !== 'Warning'
    return true
  })
}

function StorageResourceList({ resources, type, selectedResource, onResourceClick, theme = 'dark' }) {
  if (!resources.length) {
    return (
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-8 text-center">
        <p className="text-slate-400">No {type} found</p>
      </div>
    )
  }

  return (
    <div className="space-y-1.5">
      {resources.map((resource) => {
        const isSelected = isSameResource(selectedResource, resource)
        const hasIssues =
          resource.status === 'Error' ||
          resource.status === 'Warning' ||
          (resource.errors?.length || 0) > 0 ||
          (resource.warnings?.length || 0) > 0
        return (
          <Card
            key={resource.uid || `${resource.kind}-${resource.namespace || 'cluster'}-${resource.name}`}
            className={`cursor-pointer px-3 py-2 hover:border-slate-700 transition-colors ${
              isSelected
                ? theme === 'light'
                  ? 'resource-selected border-blue-500 ring-1 ring-blue-500/40'
                  : 'resource-selected border-red-500 ring-1 ring-red-500/50'
                : ''
            }`}
            onClick={() => onResourceClick(resource)}
          >
            <div className="space-y-1.5">
              <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                <h3 className="min-w-0 break-words text-sm font-semibold leading-snug text-white" title={resource.name}>
                  {resource.name}
                </h3>
                <StatusBadge status={resource.status} size="sm">
                  {resource.status}
                </StatusBadge>
                {resource.namespace && (
                <span className="text-xs text-slate-400">
                  Namespace: <span className="text-slate-300">{resource.namespace}</span>
                </span>
                )}
              </div>

              <div className={hasIssues ? 'contents' : 'hidden'}>
              {type === 'pvcs' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {resource.storage_class && (
                    <span className="text-slate-400">
                      Storage Class: <span className="text-white">{resource.storage_class}</span>
                    </span>
                  )}
                  {resource.capacity && (
                    <span className="text-slate-400">
                      Capacity: <span className="text-white">{resource.capacity}</span>
                    </span>
                  )}
                  {resource.phase && (
                    <span className="text-slate-400">
                      Phase: <span className="text-white">{resource.phase}</span>
                    </span>
                  )}
                </div>
              )}

              {type === 'pvs' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {resource.storage_class && (
                    <span className="text-slate-400">
                      Storage Class: <span className="text-white">{resource.storage_class}</span>
                    </span>
                  )}
                  {resource.capacity && (
                    <span className="text-slate-400">
                      Capacity: <span className="text-white">{resource.capacity}</span>
                    </span>
                  )}
                  {resource.phase && (
                    <span className="text-slate-400">
                      Phase: <span className="text-white">{resource.phase}</span>
                    </span>
                  )}
                </div>
              )}

              {type === 'storage_classes' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {resource.provisioner && (
                    <span className="text-slate-400">
                      Provisioner: <span className="text-white">{resource.provisioner}</span>
                    </span>
                  )}
                  {resource.volume_binding_mode && (
                    <span className="text-slate-400">
                      Binding: <span className="text-white">{resource.volume_binding_mode}</span>
                    </span>
                  )}
                  {resource.is_default && (
                    <span className="text-blue-400">Default</span>
                  )}
                </div>
              )}

              {type === 'volume_attachments' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {resource.attacher && (
                    <span className="text-slate-400">
                      Attacher: <span className="text-white">{resource.attacher}</span>
                    </span>
                  )}
                  {resource.node_name && (
                    <span className="text-slate-400">
                      Node: <span className="text-white">{resource.node_name}</span>
                    </span>
                  )}
                  {resource.pv_name && (
                    <span className="text-slate-400">
                      PV: <span className="text-white">{resource.pv_name}</span>
                    </span>
                  )}
                </div>
              )}
              </div>

              {resource.errors?.map((error, index) => (
                <div key={`error-${index}`} className="text-sm text-red-400">⚠️ {error}</div>
              ))}
              {resource.warnings?.map((warning, index) => (
                <div key={`warning-${index}`} className="text-sm text-amber-400">⚡ {warning}</div>
              ))}
            </div>
          </Card>
        )
      })}
    </div>
  )
}

function StorageResourceDetailsPanel({ resource, data, onOpenResource }) {
  const { resource: detailedResource, isLoading, error } = useResourceDetail(resource)
  const displayResource = detailedResource || resource || {
    name: '',
    kind: '',
    status: 'Healthy',
    errors: [],
    warnings: [],
    metadata: {},
  }
  const [activeTab, setActiveTab] = useState(0)
  const metadataEntries = Object.entries(displayResource.metadata || {})
  const relatedSections = getStorageRelatedSections(displayResource, data)
  const relatedCount = relatedSections.reduce((sum, section) => sum + section.resources.length, 0)
  const tabLabels = [
    'YAML',
    ...(relatedSections.length ? [`Related (${relatedCount})`] : []),
    'Analysis',
    `Errors (${displayResource.errors?.length || 0})`,
    `Warnings (${displayResource.warnings?.length || 0})`,
    'Metadata',
  ]
  const relatedTabIndex = tabLabels.findIndex((label) => label.startsWith('Related'))
  const activeTabLabel = tabLabels[activeTab] || 'YAML'
  const { raw, isLoading: isLoadingRaw, error: rawError } = useResourceRaw(
    displayResource,
    !!resource && activeTabLabel === 'YAML'
  )

  useEffect(() => {
    setActiveTab(0)
  }, [resource?.uid])

  if (!resource) {
    return (
      <Card className="sticky top-6">
        <div className="flex min-h-[24rem] items-center justify-center text-center text-slate-400">
          Select a storage resource to inspect YAML, analysis, and metadata.
        </div>
      </Card>
    )
  }

  const tabs = [
    {
      label: 'YAML',
      content: isLoadingRaw ? (
        <Card>
          <p className="text-slate-400">Loading YAML…</p>
        </Card>
      ) : rawError ? (
        <Card>
          <p className="text-red-400">{rawError}</p>
        </Card>
      ) : (
        <YAMLViewer content={raw || '# No YAML available'} />
      ),
    },
    ...(relatedSections.length
      ? [{
          label: `Related (${relatedCount})`,
          content: (
            <RelatedResourcesTab
              resource={displayResource}
              data={data}
              onOpenResource={onOpenResource}
            />
          ),
        }]
      : []),
    {
      label: 'Analysis',
      content: displayResource.health_analysis ? (
        <HealthAnalysis analysis={displayResource.health_analysis} />
      ) : (
        <Card>
          <p className="text-slate-400">No health analysis available for this resource.</p>
        </Card>
      ),
    },
    {
      label: `Errors (${displayResource.errors?.length || 0})`,
      content: (
        <Card>
          {displayResource.errors?.length ? (
            <div className="space-y-2">
              {displayResource.errors.map((entry, idx) => (
                <div key={idx} className="rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-300">
                  {entry}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-slate-400">No errors reported.</p>
          )}
        </Card>
      ),
    },
    {
      label: `Warnings (${displayResource.warnings?.length || 0})`,
      content: (
        <Card>
          {displayResource.warnings?.length ? (
            <div className="space-y-2">
              {displayResource.warnings.map((entry, idx) => (
                <div key={idx} className="rounded-xl border border-amber-500/20 bg-amber-500/10 p-3 text-sm text-amber-300">
                  {entry}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-slate-400">No warnings reported.</p>
          )}
        </Card>
      ),
    },
    {
      label: 'Metadata',
      content: (
        <Card>
          {metadataEntries.length ? (
            <div className="space-y-2">
              {metadataEntries.map(([key, value]) => (
                <div key={key} className="grid items-start gap-2 rounded-xl border border-slate-800 bg-slate-950/60 p-3 text-sm md:grid-cols-[minmax(0,12rem)_1fr] md:gap-3">
                  <div className="break-words font-medium text-slate-400 [overflow-wrap:anywhere]">{key}</div>
                  <div className="break-words text-slate-200 [overflow-wrap:anywhere]">{String(value)}</div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-slate-400">No metadata available.</p>
          )}
        </Card>
      ),
    },
  ]

  return (
    <div className="sticky top-6 space-y-2">
      <Card className="px-4 py-3">
        <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
          <h2 className="text-lg font-semibold text-white">{displayResource.name}</h2>
          <StatusBadge status={displayResource.status} size="sm">{displayResource.status}</StatusBadge>
          <span className="rounded-lg bg-slate-800 px-2 py-0.5 text-xs text-slate-300">{displayResource.kind}</span>
          {displayResource.namespace && (
            <span className="text-sm text-slate-400">Namespace: <span className="text-slate-200">{displayResource.namespace}</span></span>
          )}
        </div>
        {isLoading && <p className="mt-2 text-sm text-slate-400">Loading resource details…</p>}
        {error && <p className="mt-2 text-sm text-red-400">{error}</p>}
      </Card>
      <Tabs tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} />
    </div>
  )
}

export function Storage({ data, defaultTab = 'pvcs', theme = 'dark', onShowHelp = null }) {
  const storage = data?.infrastructure?.storage || {}
  const [activeTab, setActiveTab] = useState(defaultTab)
  const [selectedResource, setSelectedResource] = useState(null)
  const [selectedNamespace, setSelectedNamespace] = useState('all')
  const [resourceSearch, setResourceSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')

  const tabs = [
    {
      id: 'pvcs',
      label: 'PVCs',
      count: storage.pvcs?.items?.length || 0,
      issues: storage.pvcs?.pending_count || 0,
    },
    {
      id: 'pvs',
      label: 'PVs',
      count: storage.pvs?.items?.length || 0,
      issues: storage.pvs?.unbound_count || 0,
    },
    {
      id: 'storage_classes',
      label: 'Storage Classes',
      count: storage.storage_classes?.items?.length || 0,
      issues: 0,
    },
    {
      id: 'volume_attachments',
      label: 'Volume Attachments',
      count: storage.volume_attachments?.items?.length || 0,
      issues: storage.volume_attachments?.failed_count || 0,
    },
  ]

  const activeTabData = tabs.find((tab) => tab.id === activeTab)

  const allNamespaces = useMemo(() => {
    const nsSet = new Set()
    Object.values(storage).forEach((collection) => {
      collection?.items?.forEach((item) => {
        if (item.namespace) nsSet.add(item.namespace)
      })
    })
    return Array.from(nsSet).sort()
  }, [storage])

  const namespaceApplicable = useMemo(() => {
    const items = storage[activeTab]?.items || []
    return items.some((item) => !!item.namespace)
  }, [activeTab, storage])

  const filteredResources = useMemo(() => {
    let resources = storage[activeTab]?.items || []
    if (namespaceApplicable && selectedNamespace !== 'all') {
      resources = resources.filter((resource) => resource.namespace === selectedNamespace)
    }
    resources = applySearchFilter(resources, resourceSearch)
    resources = applyStatusFilter(resources, statusFilter)
    return resources
  }, [activeTab, namespaceApplicable, resourceSearch, selectedNamespace, statusFilter, storage])

  const currentSelectedResource = useMemo(() => {
    if (selectedResource) {
      return selectedResource
    }
    return filteredResources[0] || null
  }, [filteredResources, selectedResource])

  useEffect(() => {
    if (selectedResource && !currentSelectedResource) {
      setSelectedResource(null)
    }
  }, [currentSelectedResource, selectedResource])

  useEffect(() => {
    setActiveTab(defaultTab)
    setSelectedResource(null)
    setResourceSearch('')
    setStatusFilter('all')
  }, [defaultTab])

  const handleTabChange = (nextTab) => {
    setActiveTab(nextTab)
    setSelectedResource(null)
    setResourceSearch('')
    setStatusFilter('all')
  }

  return (
    <div className="flex flex-col gap-4 xl:h-full xl:min-h-0">
      <div className="xl:flex-shrink-0">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800 pb-4">
          <div className="flex flex-wrap items-center gap-4">
            {namespaceApplicable && (
              <ProjectSelector
                namespaces={allNamespaces}
                selectedNamespace={selectedNamespace}
                onNamespaceChange={(value) => {
                  setSelectedNamespace(value)
                  setSelectedResource(null)
                }}
              />
            )}
            <div className="text-sm text-slate-400">
              {filteredResources.length} of {activeTabData?.count || 0} {activeTabData?.label || 'resources'}
              {namespaceApplicable && selectedNamespace !== 'all' && (
                <span className="ml-2 text-slate-300">
                  in <span className="font-medium">{selectedNamespace}</span>
                </span>
              )}
              {activeTabData?.issues > 0 && (
                <span className="ml-2 text-red-400">({activeTabData.issues} with issues)</span>
              )}
            </div>
          </div>

          {onShowHelp && (
            <button
              onClick={onShowHelp}
              className="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-slate-300 hover:bg-slate-700"
              title="Keyboard shortcuts"
            >
              <span className="mr-1">?</span>
              Shortcuts
            </button>
          )}
        </div>
      </div>

      <div className="xl:flex-shrink-0">
        <div className="pt-2">
          <h1 className="text-3xl font-bold text-white">Storage</h1>
        </div>
      </div>

      <div className="xl:flex-shrink-0">
        <StorageTabs tabs={tabs} activeTab={activeTab} onTabChange={handleTabChange} theme={theme} />
      </div>

      <div className="grid grid-cols-1 gap-6 xl:min-h-0 xl:flex-1 xl:overflow-hidden xl:grid-cols-[24rem_minmax(0,1.35fr)]">
        <div className="pane-scrollbar min-h-0 xl:h-full xl:overflow-y-scroll xl:pr-2">
          <ResourceFilterBar
            searchTerm={resourceSearch}
            onSearchChange={setResourceSearch}
            statusFilter={statusFilter}
            onStatusFilterChange={setStatusFilter}
            resourceCount={filteredResources.length}
          />
          <StorageResourceList
            resources={filteredResources}
            type={activeTab}
            selectedResource={currentSelectedResource}
            onResourceClick={setSelectedResource}
            theme={theme}
          />
        </div>

        <div className="pane-scrollbar hidden min-h-0 xl:block xl:h-full xl:overflow-y-scroll xl:pr-2">
          <StorageResourceDetailsPanel
            resource={currentSelectedResource}
            data={data}
            onOpenResource={setSelectedResource}
          />
        </div>
      </div>

      {currentSelectedResource && (
        <div className="xl:hidden">
          <StorageResourceDetailsPanel
            resource={currentSelectedResource}
            data={data}
            onOpenResource={setSelectedResource}
          />
        </div>
      )}
    </div>
  )
}
