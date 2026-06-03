import React, { useEffect, useMemo, useState } from 'react'
import {
  Header,
  Hero,
  Sidebar,
  Dashboard,
  Workloads,
  Networking,
  Storage,
  ResourceList,
  ProjectSelector,
  Card,
  Tabs,
  YAMLViewer,
  HealthAnalysis,
  StatusBadge
} from './components'
import { KeyboardShortcutsHelp } from './components/KeyboardShortcutsHelp'
import { useHashRouter } from './hooks/useHashRouter'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { useResourceDetail, useResourceRaw } from './hooks/useResourceDetail'

function formatRelativeAge(timestamp) {
  if (!timestamp) return 'unknown'
  const parsed = new Date(timestamp)
  const time = parsed.getTime()
  if (Number.isNaN(time)) return timestamp
  const diffMs = Math.max(0, Date.now() - time)
  const minutes = Math.floor(diffMs / 60000)
  if (minutes < 1) return 'now'
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h`
  const days = Math.floor(hours / 24)
  return `${days}d`
}

function formatCollectedTimestamp(timestamp) {
  if (!timestamp) return null
  const parsed = new Date(timestamp.replace(' ', 'T'))
  if (Number.isNaN(parsed.getTime())) return `Collected ${timestamp}`
  const year = parsed.getFullYear()
  const month = String(parsed.getMonth() + 1).padStart(2, '0')
  const day = String(parsed.getDate()).padStart(2, '0')
  const hours = String(parsed.getHours()).padStart(2, '0')
  const minutes = String(parsed.getMinutes()).padStart(2, '0')
  return `Collected ${year}-${month}-${day} ${hours}:${minutes}`
}

function getEventField(resource, key) {
  return resource?.key_fields?.[key] || ''
}

function getEventObjectLabel(resource) {
  const kind = getEventField(resource, 'involved_kind')
  const name = getEventField(resource, 'involved_name')
  if (kind && name) return `${kind}/${name}`
  if (name) return name
  return resource?.name || 'Unknown object'
}

function getEventTypeLabel(resource) {
  const explicitType = getEventField(resource, 'type')
  if (explicitType) return explicitType
  return resource?.status === 'Warning' ? 'Warning' : 'Normal'
}

function EventRow({ resource, isSelected, onSelect }) {
  const eventType = getEventTypeLabel(resource)
  const reason = getEventField(resource, 'reason') || 'Unknown'
  const message = getEventField(resource, 'message') || resource.warnings?.[0] || resource.errors?.[0] || ''
  const age = formatRelativeAge(getEventField(resource, 'timestamp') || resource.creation_timestamp)
  const objectLabel = getEventObjectLabel(resource)
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => onSelect(resource)}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault()
          onSelect(resource)
        }
      }}
      className={`grid w-full grid-cols-[minmax(5.5rem,6.75rem)_2.25rem_3.25rem_4.75rem_minmax(7rem,9rem)_minmax(32rem,1fr)] gap-3 border-b px-4 py-3 text-left transition-colors ${
        isSelected
          ? 'resource-selected bg-red-500/10 border-red-500/20'
          : 'border-slate-800 hover:bg-slate-900/40'
      }`}
    >
      <div className="select-text truncate text-sm text-slate-200">{resource.namespace || 'cluster'}</div>
      <div className="select-text text-sm text-slate-400">{age}</div>
      <div className={`select-text text-sm font-medium ${eventType === 'Warning' ? 'text-amber-400' : 'text-emerald-400'}`}>{eventType}</div>
      <div className="select-text truncate text-sm text-slate-200">{reason}</div>
      <div className="min-w-0 text-sm text-slate-300" title={objectLabel}>
        <span className="line-clamp-2 cursor-help select-text break-words decoration-dotted underline-offset-2 hover:underline">
          {objectLabel}
        </span>
      </div>
      <div className="whitespace-normal break-words text-sm leading-6 text-slate-400" title={message}>
        <span className="line-clamp-5 select-text [overflow-wrap:anywhere]">{message}</span>
      </div>
    </div>
  )
}

function Events({ data, onShowHelp = null, initialNamespace = 'all' }) {
  const events = data?.core?.events?.items || []
  const [selectedNamespace, setSelectedNamespace] = useState(initialNamespace)
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [selectedResource, setSelectedResource] = useState(null)

  const namespaces = useMemo(() => {
    const values = new Set()
    events.forEach((event) => {
      if (event.namespace) values.add(event.namespace)
    })
    return Array.from(values).sort()
  }, [events])

  const filteredEvents = useMemo(() => {
    let nextEvents = events

    if (selectedNamespace !== 'all') {
      nextEvents = nextEvents.filter((event) => event.namespace === selectedNamespace)
    }

    if (searchTerm) {
      const search = searchTerm.toLowerCase()
      nextEvents = nextEvents.filter((event) => {
        const namespace = event.namespace || ''
        const reason = getEventField(event, 'reason')
        const involved = getEventObjectLabel(event)
        const message = getEventField(event, 'message')
        return (
          namespace.toLowerCase().includes(search) ||
          reason.toLowerCase().includes(search) ||
          involved.toLowerCase().includes(search) ||
          message.toLowerCase().includes(search)
        )
      })
    }

    if (statusFilter === 'warning') {
      nextEvents = nextEvents.filter((event) => getEventField(event, 'type') === 'Warning')
    } else if (statusFilter === 'healthy') {
      nextEvents = nextEvents.filter((event) => getEventField(event, 'type') !== 'Warning')
    }

    return [...nextEvents].sort((a, b) => {
      const aTime = new Date(getEventField(a, 'timestamp') || a.creation_timestamp || 0).getTime() || 0
      const bTime = new Date(getEventField(b, 'timestamp') || b.creation_timestamp || 0).getTime() || 0
      return bTime - aTime
    })
  }, [events, searchTerm, selectedNamespace, statusFilter])

  const currentSelectedResource = useMemo(() => {
    if (
      selectedResource &&
      filteredEvents.some(
        (event) =>
          event.uid === selectedResource.uid &&
          event.name === selectedResource.name &&
          event.namespace === selectedResource.namespace
      )
    ) {
      return selectedResource
    }
    return null
  }, [filteredEvents, selectedResource])

  useEffect(() => {
    if (selectedResource && !currentSelectedResource) {
      setSelectedResource(null)
    }
  }, [currentSelectedResource, selectedResource])

  useEffect(() => {
    setSelectedNamespace(initialNamespace)
    setSelectedResource(null)
  }, [initialNamespace])

  const statusOptions = [
    { value: 'all', label: 'All' },
    { value: 'warning', label: 'Warnings' },
    { value: 'healthy', label: 'Normal' },
  ]

  return (
    <div className="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
      <div className="xl:flex-shrink-0">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800 pb-4">
          <div className="flex flex-wrap items-center gap-4">
            <ProjectSelector
              namespaces={namespaces}
              selectedNamespace={selectedNamespace}
              onNamespaceChange={(value) => {
                setSelectedNamespace(value)
                setSelectedResource(null)
              }}
            />
            <div className="text-sm text-slate-400">
              {filteredEvents.length} of {events.length} events
              {selectedNamespace !== 'all' && (
                <span className="ml-2 text-slate-300">
                  in <span className="font-medium">{selectedNamespace}</span>
                </span>
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
          <h1 className="text-3xl font-bold text-white">Events</h1>
        </div>
      </div>

      <div
        className={`grid grid-cols-1 gap-6 xl:min-h-0 xl:flex-1 xl:overflow-hidden ${
          currentSelectedResource
            ? 'xl:grid-cols-[minmax(0,1.15fr)_minmax(28rem,0.85fr)]'
            : 'xl:grid-cols-[minmax(0,1fr)]'
        }`}
      >
        <div className="pane-scrollbar min-h-0 xl:h-full xl:overflow-y-scroll xl:pr-2">
          <Card className="mb-3.5">
            <div className="space-y-3">
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 left-3 flex items-center text-slate-400">🔍</div>
                <input
                  type="text"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  placeholder="Search events by namespace, reason, object, or message..."
                  className="w-full rounded-lg border border-slate-700 bg-slate-900 pl-10 pr-10 py-2 text-sm text-white placeholder-slate-500 focus:border-red-500 focus:outline-none focus:ring-1 focus:ring-red-500"
                />
                {searchTerm && (
                  <button
                    onClick={() => setSearchTerm('')}
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
                    onClick={() => setStatusFilter(option.value)}
                    className={`px-2.5 py-1.5 rounded-lg text-sm font-medium transition-colors whitespace-nowrap ${
                      statusFilter === option.value
                        ? option.value === 'warning'
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
                  Showing {filteredEvents.length} event{filteredEvents.length === 1 ? '' : 's'}
                </div>
              </div>
            </div>
          </Card>

          <Card className="overflow-hidden">
            <div className="grid grid-cols-[minmax(5.5rem,6.75rem)_2.25rem_3.25rem_4.75rem_minmax(7rem,9rem)_minmax(32rem,1fr)] gap-3 border-b border-slate-800 px-4 py-3 text-xs font-semibold uppercase tracking-wide text-slate-400">
              <div>Namespace</div>
              <div>Age</div>
              <div>Type</div>
              <div>Reason</div>
              <div>Object</div>
              <div>Message</div>
            </div>
            {filteredEvents.length ? (
              filteredEvents.map((event) => (
                <EventRow
                  key={`${event.uid}:${event.namespace || ''}:${event.name}`}
                  resource={event}
                  isSelected={!!currentSelectedResource && event.uid === currentSelectedResource.uid && event.namespace === currentSelectedResource.namespace}
                  onSelect={setSelectedResource}
                />
              ))
            ) : (
              <div className="px-4 py-8 text-sm text-slate-400">No events found for the current filters.</div>
            )}
          </Card>
        </div>

        {currentSelectedResource && (
          <div className="pane-scrollbar hidden min-h-0 xl:block xl:h-full xl:overflow-y-scroll xl:pr-2">
            <div className="mb-3 flex justify-end">
              <button
                type="button"
                onClick={() => setSelectedResource(null)}
                className="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-slate-300 hover:bg-slate-700"
              >
                Hide details
              </button>
            </div>
            <ResourceDetailsPanel resource={currentSelectedResource} />
          </div>
        )}
      </div>

      {currentSelectedResource && (
        <div className="xl:hidden">
          <ResourceDetailsPanel resource={currentSelectedResource} />
        </div>
      )}
    </div>
  )
}

function ResourceDetailsPanel({ resource }) {
  const { resource: detailedResource, isLoading, error } = useResourceDetail(resource)
  const displayResource = detailedResource || resource || {
    name: '',
    kind: '',
    status: 'Healthy',
    errors: [],
    warnings: [],
    metadata: {},
  }
  const displayName = displayResource.name === 'cluster'
    ? `${displayResource.kind}: cluster`
    : displayResource.name
  const [activeTab, setActiveTab] = useState(0)
  const metadataEntries = Object.entries(displayResource.metadata || {})
  const tabLabels = [
    'YAML',
    'Analysis',
    `Errors (${displayResource.errors?.length || 0})`,
    `Warnings (${displayResource.warnings?.length || 0})`,
    'Metadata',
  ]
  const activeTabLabel = tabLabels[activeTab] || 'YAML'
  const eventMessage =
    displayResource.kind === 'Event'
      ? getEventField(displayResource, 'message') ||
        displayResource.warnings?.[0] ||
        displayResource.errors?.[0] ||
        ''
      : ''
  const eventType = displayResource.kind === 'Event' ? getEventTypeLabel(displayResource) : ''
  const eventReason = displayResource.kind === 'Event' ? getEventField(displayResource, 'reason') || 'Event' : ''
  const { raw, isLoading: isLoadingRaw, error: rawError } = useResourceRaw(
    displayResource,
    !!resource && activeTabLabel === 'YAML'
  )

  React.useEffect(() => {
    setActiveTab(0)
  }, [resource?.uid])

  if (!resource) {
    return (
      <Card className="sticky top-6">
        <div className="flex min-h-[24rem] items-center justify-center text-center text-slate-400">
          Select a resource to inspect YAML, analysis, and metadata.
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
              {displayResource.errors.map((error, idx) => (
                <div key={idx} className="rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-300">
                  {error}
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
              {displayResource.warnings.map((warning, idx) => (
                <div key={idx} className="rounded-xl border border-amber-500/20 bg-amber-500/10 p-3 text-sm text-amber-300">
                  {warning}
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
          <h2 className="text-lg font-semibold text-white">{displayName}</h2>
          <StatusBadge status={displayResource.status} size="sm">{displayResource.status}</StatusBadge>
          <span className="rounded-lg bg-slate-800 px-2 py-0.5 text-xs text-slate-300">{displayResource.kind}</span>
          {displayResource.namespace && (
            <span className="text-sm text-slate-400">Namespace: <span className="text-slate-200">{displayResource.namespace}</span></span>
          )}
        </div>
        {eventMessage && (
          <div
            className={`mt-4 rounded-xl border p-4 ${
              eventType === 'Warning'
                ? 'border-amber-500/30 bg-amber-500/10'
                : 'border-emerald-500/25 bg-emerald-500/10'
            }`}
          >
            <div className="mb-2 flex flex-wrap items-center gap-2">
              <span className="text-xs font-semibold uppercase tracking-wide text-slate-300">Message</span>
              <span
                className={`rounded-full px-2 py-0.5 text-[11px] font-semibold uppercase tracking-wide ${
                  eventType === 'Warning'
                    ? 'bg-amber-500/20 text-amber-300'
                    : 'bg-emerald-500/20 text-emerald-300'
                }`}
              >
                {eventReason}
              </span>
            </div>
            <p className="text-sm leading-6 text-slate-100">{eventMessage}</p>
          </div>
        )}
        {isLoading && (
          <p className="mt-2 text-sm text-slate-400">Loading resource details…</p>
        )}
        {error && (
          <p className="mt-2 text-sm text-red-400">{error}</p>
        )}
      </Card>
      <Tabs tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} />
    </div>
  )
}

function ResourceSplitView({ title, resources = [], emptyMessage = 'No resources found' }) {
  const [selectedResource, setSelectedResource] = useState(null)
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [chromeCollapsed, setChromeCollapsed] = useState(false)
  const handlePaneScroll = React.useCallback((event) => {
    const shouldCollapse = event.currentTarget.scrollTop > 24
    setChromeCollapsed((previous) => (previous === shouldCollapse ? previous : shouldCollapse))
  }, [])

  const filteredResources = useMemo(() => {
    let nextResources = resources

    if (searchTerm) {
      const search = searchTerm.toLowerCase()
      nextResources = nextResources.filter((resource) => {
        const namespace = resource.namespace || ''
        return (
          resource.name?.toLowerCase().includes(search) ||
          namespace.toLowerCase().includes(search) ||
          resource.kind?.toLowerCase().includes(search)
        )
      })
    }

    if (statusFilter !== 'all') {
      nextResources = nextResources.filter((resource) => {
        const errors = resource.errors?.length || 0
        const warnings = resource.warnings?.length || 0
        if (statusFilter === 'error') return errors > 0 || resource.status === 'Error'
        if (statusFilter === 'warning') {
          return (warnings > 0 && errors === 0) || resource.status === 'Warning'
        }
        if (statusFilter === 'healthy') {
          return errors === 0 && warnings === 0 && resource.status !== 'Error' && resource.status !== 'Warning'
        }
        return true
      })
    }

    return nextResources
  }, [resources, searchTerm, statusFilter])

  const currentSelectedResource = useMemo(() => {
    if (
      selectedResource &&
      filteredResources.some(
        (resource) =>
          resource.name === selectedResource.name &&
          resource.namespace === selectedResource.namespace &&
          resource.kind === selectedResource.kind
      )
    ) {
      return selectedResource
    }

    return filteredResources[0] || null
  }, [filteredResources, selectedResource])

  React.useEffect(() => {
    if (selectedResource && !currentSelectedResource) {
      setSelectedResource(null)
    }
  }, [currentSelectedResource, selectedResource])

  const statusOptions = [
    { value: 'all', label: 'All' },
    { value: 'error', label: 'Errors' },
    { value: 'warning', label: 'Warnings' },
    { value: 'healthy', label: 'Healthy' },
  ]

  return (
    <div className="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
      <div
        className={`overflow-hidden transition-all duration-200 xl:flex-shrink-0 ${
          chromeCollapsed ? 'xl:max-h-0 xl:opacity-0 xl:-translate-y-2' : 'max-h-24 opacity-100 translate-y-0'
        }`}
      >
        <div className="flex items-center justify-between gap-4">
          <h1 className="text-3xl font-bold text-white">{title}</h1>
          <div className="text-sm text-slate-400">{filteredResources.length} of {resources.length} resource{resources.length === 1 ? '' : 's'}</div>
        </div>
      </div>

      {resources.length === 0 ? (
        <Card>
          <p className="text-slate-400">{emptyMessage}</p>
        </Card>
      ) : (
        <div className="grid min-h-0 flex-1 grid-rows-[minmax(0,1fr)] gap-4 overflow-hidden xl:h-[calc(100vh-17rem)] xl:max-h-[calc(100vh-17rem)] xl:grid-cols-[minmax(0,28rem)_minmax(0,1fr)]">
          <div className="pane-scrollbar h-full min-h-0 overflow-y-scroll pr-2" onScroll={handlePaneScroll}>
            <Card className="mb-2 px-3 py-2.5">
              <div className="space-y-2">
                <div className="relative">
                  <div className="pointer-events-none absolute inset-y-0 left-3 flex items-center text-slate-400">
                    🔍
                  </div>
                  <input
                    type="text"
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    placeholder="Search resources by name..."
                    className="w-full rounded-lg border border-slate-700 bg-slate-900 py-1.5 pl-10 pr-10 text-sm text-white placeholder-slate-500 focus:border-red-500 focus:outline-none focus:ring-1 focus:ring-red-500"
                  />
                  {searchTerm && (
                    <button
                      onClick={() => setSearchTerm('')}
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
                      onClick={() => setStatusFilter(option.value)}
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
                    Showing {filteredResources.length} resource{filteredResources.length === 1 ? '' : 's'}
                  </div>
                </div>
              </div>
            </Card>
            <ResourceList
              resources={filteredResources}
              selectedIndex={Math.max(
                0,
                filteredResources.findIndex(
                  (resource) =>
                    currentSelectedResource &&
                    resource.name === currentSelectedResource.name &&
                    resource.namespace === currentSelectedResource.namespace &&
                    resource.kind === currentSelectedResource.kind
                )
              )}
              onResourceClick={(resource) => setSelectedResource(resource)}
            />
          </div>
          <div className="pane-scrollbar h-full min-h-0 overflow-y-scroll pr-2" onScroll={handlePaneScroll}>
            <ResourceDetailsPanel resource={currentSelectedResource} />
          </div>
        </div>
      )}
    </div>
  )
}

function ClusterHealth({ data }) {
  const operators = data?.overview?.cluster_health?.operators || []
  return (
    <ResourceSplitView
      title="Cluster Health"
      resources={operators}
      emptyMessage="No cluster operators found"
    />
  )
}

function Nodes({ data }) {
  return (
    <ResourceSplitView
      title="Nodes"
      resources={data?.core?.nodes?.items || []}
      emptyMessage="No nodes found"
    />
  )
}

function Namespaces({ data }) {
  return (
    <ResourceSplitView
      title="Namespaces"
      resources={data?.core?.namespaces?.items || []}
      emptyMessage="No namespaces found"
    />
  )
}

function Compute({ title, resources = [], emptyMessage = 'No compute resources found' }) {
  return (
    <ResourceSplitView
      title={title}
      resources={resources}
      emptyMessage={emptyMessage}
    />
  )
}

function ClusterOperators({ data, onShowHelp = null, theme = 'dark' }) {
  return (
    <Workloads
      data={data}
      defaultTab="operators"
      visibleTabs={['operators']}
      title="Cluster Operators"
      onShowHelp={onShowHelp}
      theme={theme}
    />
  )
}

function PlatformView({ platform, data }) {
  if (platform === 'virtualization') {
    const virtualization = data?.platform?.virtualization || {}
    const resources = Object.values(virtualization).flatMap((collection) =>
      Array.isArray(collection?.items) ? collection.items : []
    )

    return (
      <ResourceSplitView
        title="Virtualization"
        resources={resources}
        emptyMessage="No virtualization resources found"
      />
    )
  }

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-white capitalize">{platform.replace('-', ' ')}</h1>
      <Card>
        <p className="text-slate-400">Platform-specific view for {platform} (coming soon)</p>
      </Card>
    </div>
  )
}

function App({ data }) {
  const [showHelp, setShowHelp] = useState(false)
  const [theme, setTheme] = useState(() => {
    if (typeof window === 'undefined') return 'dark'
    return window.localStorage.getItem('mga-theme') || 'dark'
  })
  const { section: routeSection, navigate } = useHashRouter()
  const activeSection = routeSection || 'dashboard'
  const workloadVisibleTabs = [
    'topology',
    'pods',
    'deployments',
    'statefulsets',
    'daemonsets',
    'jobs',
    'cronjobs',
    'replicasets',
    'configmaps',
    'secrets',
  ]
  const networkingVisibleTabs = ['services', 'routes', 'endpoints', 'networkpolicies', 'ingresscontrollers']
  const workloadSectionMap = {
    'workloads-topology': 'topology',
    'workloads-pods': 'pods',
    'workloads-deployments': 'deployments',
    'workloads-statefulsets': 'statefulsets',
    'workloads-daemonsets': 'daemonsets',
    'workloads-jobs': 'jobs',
    'workloads-cronjobs': 'cronjobs',
    'workloads-replicasets': 'replicasets',
    'workloads-configmaps': 'configmaps',
    'workloads-secrets': 'secrets',
  }
  const networkingSectionMap = {
    'networking-services': 'services',
    'networking-routes': 'routes',
    'networking-endpoints': 'endpoints',
    'networking-networkpolicies': 'networkpolicies',
    'networking-ingresscontrollers': 'ingresscontrollers',
  }
  const securitySectionMap = {
    'security-clusterroles': {
      title: 'Cluster Roles',
      resources: data?.security?.cluster_roles?.items || [],
      emptyMessage: 'No cluster roles found',
    },
    'security-clusterrolebindings': {
      title: 'Cluster Role Bindings',
      resources: data?.security?.cluster_role_bindings?.items || [],
      emptyMessage: 'No cluster role bindings found',
    },
    'security-securitycontextconstraints': {
      title: 'Security Context Constraints',
      resources: data?.security?.security_context_constraints?.items || [],
      emptyMessage: 'No security context constraints found',
    },
  }
  const administrationSectionMap = {
    'administration-cluster-settings': {
      title: 'Cluster Settings',
      resources: data?.administration?.cluster_settings?.items || [],
      emptyMessage: 'No cluster settings found',
    },
    'administration-namespaces': {
      title: 'Namespaces',
      resources: data?.administration?.namespaces?.items || data?.core?.namespaces?.items || [],
      emptyMessage: 'No namespaces found',
    },
    'administration-resourcequotas': {
      title: 'ResourceQuotas',
      resources: data?.administration?.resource_quotas?.items || [],
      emptyMessage: 'No resource quotas found',
    },
    'administration-limitranges': {
      title: 'LimitRanges',
      resources: data?.administration?.limit_ranges?.items || [],
      emptyMessage: 'No limit ranges found',
    },
    'administration-customresourcedefinitions': {
      title: 'CustomResourceDefinitions',
      resources: data?.administration?.custom_resource_definitions?.items || [],
      emptyMessage: 'No custom resource definitions found',
    },
    'administration-dynamicplugins': {
      title: 'Dynamic Plugins',
      resources: data?.administration?.dynamic_plugins?.items || [],
      emptyMessage: 'No dynamic plugins found',
    },
  }
  const storageSectionMap = {
    'storage-pvcs': 'pvcs',
    'storage-pvs': 'pvs',
    'storage-storage_classes': 'storage_classes',
    'storage-volume_attachments': 'volume_attachments',
  }
  const computeSectionMap = {
    'compute-machines': {
      title: 'Machines',
      resources: data?.compute?.machines?.items || [],
      emptyMessage: 'No machines found',
    },
    'compute-machineautoscalers': {
      title: 'MachineAutoscalers',
      resources: data?.compute?.machine_autoscalers?.items || [],
      emptyMessage: 'No machine autoscalers found',
    },
    'compute-machinehealthchecks': {
      title: 'MachineHealthChecks',
      resources: data?.compute?.machine_health_checks?.items || [],
      emptyMessage: 'No machine health checks found',
    },
    'compute-controlplanemachinesets': {
      title: 'ControlPlaneMachineSets',
      resources: data?.compute?.control_plane_machine_sets?.items || [],
      emptyMessage: 'No control plane machine sets found',
    },
    'compute-machinesets': {
      title: 'MachineSets',
      resources: data?.compute?.machine_sets?.items || [],
      emptyMessage: 'No machine sets found',
    },
    'compute-machineconfigs': {
      title: 'MachineConfigs',
      resources: data?.compute?.machine_configs?.items || [],
      emptyMessage: 'No machine configs found',
    },
    'compute-machineconfigpools': {
      title: 'MachineConfigPools',
      resources: data?.compute?.machine_config_pools?.items || [],
      emptyMessage: 'No machine config pools found',
    },
    'compute-machineconfigurations': {
      title: 'MachineConfigurations',
      resources: data?.compute?.machine_configurations?.items || [],
      emptyMessage: 'No machine configurations found',
    },
  }
  const fusionWorkloadNamespace = 'ibm-spectrum-fusion-ns'
  const fusionSystemNamespace = 'ibm-spectrum-scale'
  const fullHeightSection =
    activeSection === 'cluster-health' ||
    activeSection === 'cluster-operators' ||
    activeSection === 'nodes' ||
    activeSection === 'namespaces' ||
    activeSection.startsWith('compute-') ||
    activeSection === 'workloads' ||
    activeSection.startsWith('workloads-') ||
    activeSection === 'events' ||
    activeSection.startsWith('security-') ||
    activeSection.startsWith('administration-') ||
    activeSection === 'networking' ||
    activeSection.startsWith('networking-') ||
    activeSection === 'storage' ||
    activeSection.startsWith('storage-') ||
    activeSection.startsWith('fusion-') ||
    activeSection === 'virtualization'

  React.useEffect(() => {
    document.documentElement.classList.toggle('theme-light', theme === 'light')
    window.localStorage.setItem('mga-theme', theme)
  }, [theme])

  useKeyboardShortcuts({
    onHelp: () => setShowHelp(!showHelp),
    onClose: () => setShowHelp(false),
    onGoToSection: (sectionId) => {
      navigate(sectionId)
    },
    enabled: !showHelp,
  })

  const handleSectionClick = (section) => {
    navigate(section.id)
  }

  const heroStats = useMemo(() => ([
    {
      label: 'Nodes',
      value: `${data?.overview?.dashboard?.healthy_nodes || 0}/${data?.overview?.dashboard?.total_nodes || 0}`,
      icon: '🖥️',
    },
    {
      label: 'Pods',
      value: `${data?.overview?.dashboard?.healthy_pods || 0}/${data?.overview?.dashboard?.total_pods || 0}`,
      icon: '📦',
    },
    {
      label: 'Operators',
      value: `${(data?.overview?.dashboard?.total_operators || 0) - (data?.overview?.dashboard?.degraded_operators || 0)}/${data?.overview?.dashboard?.total_operators || 0}`,
      icon: '⚙️',
    },
  ]), [data])

  const renderContent = () => {
    switch (activeSection) {
      case 'dashboard':
        return <Dashboard data={data} />
      case 'cluster-health':
        return <ClusterHealth data={data} />
      case 'cluster-operators':
        return <ClusterOperators data={data} onShowHelp={() => setShowHelp(true)} theme={theme} />
      case 'nodes':
        return <Nodes data={data} />
      case 'compute-machines':
      case 'compute-machineautoscalers':
      case 'compute-machinehealthchecks':
      case 'compute-controlplanemachinesets':
      case 'compute-machinesets':
      case 'compute-machineconfigs':
      case 'compute-machineconfigpools':
      case 'compute-machineconfigurations': {
        const config = computeSectionMap[activeSection]
        return (
          <Compute
            title={config.title}
            resources={config.resources}
            emptyMessage={config.emptyMessage}
          />
        )
      }
      case 'workloads':
        return (
          <Workloads
            data={data}
            onShowHelp={() => setShowHelp(true)}
            visibleTabs={workloadVisibleTabs}
            theme={theme}
            showTabs={false}
          />
        )
      case 'workloads-topology':
      case 'workloads-pods':
      case 'workloads-deployments':
      case 'workloads-statefulsets':
      case 'workloads-daemonsets':
      case 'workloads-jobs':
      case 'workloads-cronjobs':
      case 'workloads-replicasets':
      case 'workloads-configmaps':
      case 'workloads-secrets':
        return (
          <Workloads
            data={data}
            title="Workloads"
            defaultTab={workloadSectionMap[activeSection]}
            visibleTabs={workloadVisibleTabs}
            onShowHelp={() => setShowHelp(true)}
            theme={theme}
            showTabs={false}
          />
        )
      case 'namespaces':
        return <Namespaces data={data} />
      case 'administration-cluster-settings':
      case 'administration-namespaces':
      case 'administration-resourcequotas':
      case 'administration-limitranges':
      case 'administration-customresourcedefinitions':
      case 'administration-dynamicplugins': {
        const config = administrationSectionMap[activeSection]
        return (
          <Compute
            title={config.title}
            resources={config.resources}
            emptyMessage={config.emptyMessage}
          />
        )
      }
      case 'events':
        return <Events data={data} onShowHelp={() => setShowHelp(true)} />
      case 'security-clusterroles':
      case 'security-clusterrolebindings':
      case 'security-securitycontextconstraints': {
        const config = securitySectionMap[activeSection]
        return (
          <Compute
            title={config.title}
            resources={config.resources}
            emptyMessage={config.emptyMessage}
          />
        )
      }
      case 'networking':
        return (
          <Workloads
            data={data}
            title="Networking"
            defaultTab="services"
            visibleTabs={networkingVisibleTabs}
            onShowHelp={() => setShowHelp(true)}
            theme={theme}
            showTabs={false}
          />
        )
      case 'networking-services':
      case 'networking-routes':
      case 'networking-endpoints':
      case 'networking-networkpolicies':
      case 'networking-ingresscontrollers':
        return (
          <Workloads
            data={data}
            title="Networking"
            defaultTab={networkingSectionMap[activeSection]}
            visibleTabs={networkingVisibleTabs}
            onShowHelp={() => setShowHelp(true)}
            theme={theme}
            showTabs={false}
          />
        )
      case 'fusion-main':
        return (
          <Workloads
            data={data}
            title="IBM Spectrum Fusion"
            defaultTab="deployments"
            visibleTabs={workloadVisibleTabs}
            initialNamespace={fusionWorkloadNamespace}
            onShowHelp={() => setShowHelp(true)}
            theme={theme}
          />
        )
      case 'fusion-storage-scale':
        return (
          <Events
            data={data}
            initialNamespace={fusionSystemNamespace}
            onShowHelp={() => setShowHelp(true)}
          />
        )
      case 'storage':
        return <Storage data={data} theme={theme} onShowHelp={() => setShowHelp(true)} />
      case 'storage-pvcs':
      case 'storage-pvs':
      case 'storage-storage_classes':
      case 'storage-volume_attachments':
        return (
          <Storage
            data={data}
            defaultTab={storageSectionMap[activeSection]}
            theme={theme}
            onShowHelp={() => setShowHelp(true)}
          />
        )
      case 'fusion':
      case 'odf':
      case 'service-mesh':
      case 'acm':
      case 'virtualization':
      case 'cpd':
        return <PlatformView platform={activeSection} data={data} />
      default:
        return <Dashboard data={data} />
    }
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-slate-950">
      <Header
        title="Must-Gather Explorer"
        theme={theme}
        onToggleTheme={() => setTheme((previous) => (previous === 'light' ? 'dark' : 'light'))}
      />

      <Hero
        status="unknown"
        statusLabel={formatCollectedTimestamp(data?.overview?.dashboard?.collection_timestamp) || 'Collected unknown'}
        title={data?.overview?.dashboard?.cluster_name || 'OpenShift Cluster'}
        version={data?.overview?.dashboard?.cluster_version || 'Unknown'}
        platform={data?.overview?.dashboard?.platform_type || 'OpenShift'}
        stats={heroStats}
        theme={theme}
      />

      <div className="flex min-h-0 flex-1">
        <Sidebar
          data={data}
          activeSection={activeSection}
          onSectionClick={handleSectionClick}
          theme={theme}
        />

        <main className={`flex-1 min-h-0 p-6 ${fullHeightSection ? 'overflow-hidden' : 'overflow-y-auto'}`}>
          <div className={`mx-auto max-w-7xl ${fullHeightSection ? 'flex h-full min-h-0 flex-col' : ''} xl:flex xl:h-full xl:min-h-0 xl:flex-col`}>
            {!fullHeightSection && (
              <div className="mb-6 flex items-center justify-end">
                <button
                  onClick={() => setShowHelp(true)}
                  className="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-slate-300 hover:bg-slate-700"
                  title="Keyboard shortcuts"
                >
                  <span className="mr-1">?</span>
                  Shortcuts
                </button>
              </div>
            )}

            <div className={fullHeightSection ? 'flex min-h-0 flex-1 flex-col overflow-hidden' : ''}>
              {renderContent()}
            </div>
          </div>
        </main>
      </div>

      <KeyboardShortcutsHelp isOpen={showHelp} onClose={() => setShowHelp(false)} />
    </div>
  )
}

export default App
