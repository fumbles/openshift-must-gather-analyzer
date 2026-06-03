import React, { useState } from 'react'
import { Card } from './Card'
import { StatusBadge } from './StatusBadge'

function NetworkingTabs({ tabs, activeTab, onTabChange }) {
  return (
    <div className="flex gap-2 border-b border-slate-800 overflow-x-auto">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors whitespace-nowrap ${
            activeTab === tab.id
              ? 'border-b-2 border-red-500 text-white'
              : 'text-slate-400 hover:text-slate-300'
          }`}
        >
          <span>{tab.label}</span>
          <span className="text-xs text-slate-500">({tab.count})</span>
          {tab.issues > 0 && (
            <span className="rounded-full bg-red-500/20 px-2 py-0.5 text-xs text-red-400">
              {tab.issues}
            </span>
          )}
        </button>
      ))}
    </div>
  )
}

function NetworkingResourceList({ resources, type }) {
  if (!resources || resources.length === 0) {
    return (
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-8 text-center">
        <p className="text-slate-400">No {type} found</p>
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {resources.map((resource, index) => (
        <Card key={index} className="hover:border-slate-700 transition-colors cursor-pointer">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <div className="flex items-center gap-3 mb-2">
                <h3 className="text-lg font-semibold text-white">{resource.name}</h3>
                <StatusBadge status={resource.status}>
                  {resource.status}
                </StatusBadge>
              </div>

              {resource.namespace && (
                <div className="text-sm text-slate-400 mb-2">
                  Namespace: <span className="text-slate-300">{resource.namespace}</span>
                </div>
              )}

              {/* Type-specific details */}
              {type === 'routes' && (
                <div className="space-y-1 text-sm">
                  {resource.host && (
                    <div className="text-slate-400">
                      Host: <span className="text-slate-300">{resource.host}</span>
                    </div>
                  )}
                  {resource.path && (
                    <div className="text-slate-400">
                      Path: <span className="text-slate-300">{resource.path}</span>
                    </div>
                  )}
                  {resource.service && (
                    <div className="text-slate-400">
                      Service: <span className="text-slate-300">{resource.service}</span>
                    </div>
                  )}
                  {resource.admitted !== undefined && (
                    <div className={resource.admitted ? 'text-green-400' : 'text-red-400'}>
                      {resource.admitted ? '✓ Admitted' : '✗ Not Admitted'}
                    </div>
                  )}
                </div>
              )}

              {type === 'services' && (
                <div className="space-y-1 text-sm">
                  {resource.type && (
                    <div className="text-slate-400">
                      Type: <span className="text-slate-300">{resource.type}</span>
                    </div>
                  )}
                  {resource.cluster_ip && (
                    <div className="text-slate-400">
                      Cluster IP: <span className="text-slate-300">{resource.cluster_ip}</span>
                    </div>
                  )}
                  {resource.ports && resource.ports.length > 0 && (
                    <div className="text-slate-400">
                      Ports: <span className="text-slate-300">{resource.ports.join(', ')}</span>
                    </div>
                  )}
                  {resource.has_endpoints !== undefined && (
                    <div className={resource.has_endpoints ? 'text-green-400' : 'text-amber-400'}>
                      {resource.has_endpoints ? '✓ Has Endpoints' : '⚠ No Endpoints'}
                    </div>
                  )}
                </div>
              )}

              {type === 'endpoints' && (
                <div className="space-y-1 text-sm">
                  {resource.subsets && (
                    <div className="text-slate-400">
                      Subsets: <span className="text-slate-300">{resource.subsets}</span>
                    </div>
                  )}
                  {resource.ready_addresses !== undefined && (
                    <div className="text-slate-400">
                      Ready: <span className="text-white">{resource.ready_addresses}</span>
                    </div>
                  )}
                  {resource.not_ready_addresses !== undefined && resource.not_ready_addresses > 0 && (
                    <div className="text-amber-400">
                      Not Ready: {resource.not_ready_addresses}
                    </div>
                  )}
                </div>
              )}

              {type === 'ingress_controllers' && (
                <div className="space-y-1 text-sm">
                  {resource.domain && (
                    <div className="text-slate-400">
                      Domain: <span className="text-slate-300">{resource.domain}</span>
                    </div>
                  )}
                  {resource.replicas !== undefined && (
                    <div className="text-slate-400">
                      Replicas: <span className="text-white">{resource.replicas}</span>
                    </div>
                  )}
                  {resource.available !== undefined && (
                    <div className={resource.available ? 'text-green-400' : 'text-red-400'}>
                      {resource.available ? '✓ Available' : '✗ Not Available'}
                    </div>
                  )}
                </div>
              )}

              {/* Errors and warnings */}
              {resource.errors && resource.errors.length > 0 && (
                <div className="mt-2">
                  {resource.errors.map((error, idx) => (
                    <div key={idx} className="text-sm text-red-400">⚠️ {error}</div>
                  ))}
                </div>
              )}

              {resource.warnings && resource.warnings.length > 0 && (
                <div className="mt-2">
                  {resource.warnings.map((warning, idx) => (
                    <div key={idx} className="text-sm text-amber-400">⚡ {warning}</div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </Card>
      ))}
    </div>
  )
}

export function Networking({ data }) {
  const networking = data?.infrastructure?.networking || {}
  const [activeTab, setActiveTab] = useState('routes')

  const tabs = [
    {
      id: 'routes',
      label: 'Routes',
      count: networking.routes?.items?.length || 0,
      issues: networking.routes?.unadmitted_count || 0
    },
    {
      id: 'services',
      label: 'Services',
      count: networking.services?.items?.length || 0,
      issues: networking.services?.no_endpoints_count || 0
    },
    {
      id: 'endpoints',
      label: 'Endpoints',
      count: networking.endpoints?.items?.length || 0,
      issues: networking.endpoints?.not_ready_count || 0
    },
    {
      id: 'ingress_controllers',
      label: 'Ingress Controllers',
      count: networking.ingress_controllers?.items?.length || 0,
      issues: networking.ingress_controllers?.degraded_count || 0
    },
  ]

  const activeTabData = tabs.find(t => t.id === activeTab)
  const resources = networking[activeTab]?.items || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold text-white">Networking</h1>
        <div className="text-sm text-slate-400">
          {activeTabData?.count || 0} {activeTabData?.label || 'resources'}
          {activeTabData?.issues > 0 && (
            <span className="ml-2 text-red-400">
              ({activeTabData.issues} with issues)
            </span>
          )}
        </div>
      </div>

      {/* Tabs */}
      <NetworkingTabs tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} />

      {/* Resource list for active tab */}
      <NetworkingResourceList resources={resources} type={activeTab} />
    </div>
  )
}
