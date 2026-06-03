import React, { useState, useMemo } from 'react'
import { Card } from './Card'
import { ResourceList } from './ResourceList'
import { ProjectSelector } from './ProjectSelector'
import { StatusBadge } from './StatusBadge'
import { Tabs } from './Tabs'
import { YAMLViewer } from './YAMLViewer'
import { HealthAnalysis } from './HealthAnalysis'
import { ContainerLogs } from './ContainerLogs'
import { useResourceDetail, useResourceLogs, useResourceRaw } from '../hooks/useResourceDetail'

function WorkloadTabs({ tabs, activeTab, onTabChange, theme = 'dark' }) {
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

function getLabel(resource, key) {
  return resource?.labels?.[key] || null
}

function getKeyField(resource, key) {
  return resource?.key_fields?.[key]
}

function getAnnotation(resource, key) {
  return resource?.annotations?.[key] || null
}

function getNumberField(resource, key) {
  const value = getKeyField(resource, key)
  if (value === undefined || value === null || value === '') return undefined
  const parsed = Number(value)
  return Number.isNaN(parsed) ? undefined : parsed
}

function getPodPhaseLabel(resource) {
  const phase = getKeyField(resource, 'phase')
  if (phase === 'Succeeded') return 'Completed'
  return phase || 'Unknown'
}

function hasHashLikeSegment(value) {
  if (typeof value !== 'string') return false
  return /[a-f0-9]{20,}/i.test(value) || /[A-Za-z0-9_-]{36,}/.test(value)
}

function getNameWrapClass(value) {
  return hasHashLikeSegment(value) ? 'break-all' : 'break-words'
}

function getCollapsedNameStyle() {
  return {
    display: '-webkit-box',
    WebkitLineClamp: 2,
    WebkitBoxOrient: 'vertical',
    overflow: 'hidden',
  }
}

function getCollapsedNameProps(value, expanded = false) {
  return expanded
    ? { title: value }
    : {
        style: getCollapsedNameStyle(),
        title: value,
      }
}

function hasOwnerReference(resource, kind, name) {
  return (resource?.owner_references || []).some(
    (ref) => ref.kind === kind && ref.name === name
  )
}

function hasOwnerReferenceUid(resource, uid) {
  if (!uid) return false
  return (resource?.owner_references || []).some((ref) => ref.uid === uid)
}

function getAllWorkloadResources(workloads) {
  return Object.values(workloads || {}).flatMap((collection) => collection?.items || [])
}

function resolveResourceReference(reference, workloads, fallbackNamespace) {
  const namespace = reference.namespace || fallbackNamespace || null
  return getAllWorkloadResources(workloads).find(
    (candidate) =>
      candidate.kind === reference.kind &&
      candidate.name === reference.name &&
      (namespace === null || candidate.namespace === namespace)
  ) || null
}

function getRelationshipResources(resource, workloads, relationshipType) {
  return dedupeResources(
    (resource?.relationships || [])
      .filter((relationship) => relationship.relationship === relationshipType)
      .map((relationship) =>
        resolveResourceReference(relationship, workloads, resource.namespace)
      )
      .filter(Boolean)
  )
}

function getControllerUid(resource) {
  return (
    getLabel(resource, 'batch.kubernetes.io/controller-uid') ||
    getLabel(resource, 'controller-uid') ||
    resource?.uid ||
    null
  )
}

function matchesNamePrefix(resourceName, expectedPrefix) {
  return (
    typeof resourceName === 'string' &&
    typeof expectedPrefix === 'string' &&
    resourceName.startsWith(`${expectedPrefix}-`)
  )
}

function getPodsForJob(job, workloads) {
  const pods = workloads?.pods?.items || []
  const jobControllerUid = getControllerUid(job)
  return pods.filter((pod) => {
    if (pod.namespace !== job.namespace) return false
    return (
      getLabel(pod, 'job-name') === job.name ||
      getLabel(pod, 'batch.kubernetes.io/job-name') === job.name ||
      getLabel(pod, 'controller-uid') === jobControllerUid ||
      getLabel(pod, 'batch.kubernetes.io/controller-uid') === jobControllerUid ||
      hasOwnerReference(pod, 'Job', job.name) ||
      hasOwnerReferenceUid(pod, job.uid) ||
      matchesNamePrefix(pod.name, job.name)
    )
  })
}

function parseSelector(selectorValue) {
  if (!selectorValue || typeof selectorValue !== 'string') return {}
  return selectorValue
    .split(',')
    .map((entry) => entry.trim())
    .filter(Boolean)
    .reduce((acc, entry) => {
      const separator = entry.indexOf('=')
      if (separator <= 0) return acc
      const key = entry.slice(0, separator).trim()
      const value = entry.slice(separator + 1).trim()
      if (key && value) {
        acc[key] = value
      }
      return acc
    }, {})
}

function podMatchesSelector(pod, selector) {
  const entries = Object.entries(selector)
  if (!entries.length) return false
  return entries.every(([key, value]) => getLabel(pod, key) === value)
}

function getPodsForService(service, workloads) {
  const selector = parseSelector(getKeyField(service, 'selector'))
  if (!Object.keys(selector).length) return []
  const pods = workloads?.pods?.items || []
  return pods.filter((pod) => {
    if (pod.namespace !== service.namespace) return false
    return podMatchesSelector(pod, selector)
  })
}

function getServiceForRoute(route, workloads) {
  const serviceName = getKeyField(route, 'service_name')
  if (!serviceName) return null
  const services = workloads?.services?.items || []
  return (
    services.find(
      (service) =>
        service.namespace === route.namespace &&
        service.name === serviceName
    ) || null
  )
}

function getRoutesForService(service, workloads) {
  const routes = workloads?.routes?.items || []
  return routes.filter(
    (route) =>
      route.namespace === service.namespace &&
      getKeyField(route, 'service_name') === service.name
  )
}

function getEndpointsForService(service, workloads) {
  const endpoints = workloads?.endpoints?.items || []
  return endpoints.filter(
    (endpoint) =>
      endpoint.namespace === service.namespace &&
      endpoint.name === service.name
  )
}

function getRouteRouterNames(route) {
  if (typeof route?.raw !== 'string') return []
  const matches = [...route.raw.matchAll(/routerName:\s*([^\s]+)/g)]
  return [...new Set(matches.map((match) => match[1]).filter(Boolean))]
}

function getIngressControllersForRoute(route, workloads) {
  const ingressControllers = workloads?.ingresscontrollers?.items || []
  const routerNames = new Set(getRouteRouterNames(route))
  const host = getKeyField(route, 'host') || ''

  return ingressControllers.filter((controller) => {
    if (routerNames.has(controller.name)) return true
    const domain = getKeyField(controller, 'domain')
    return Boolean(domain && host && host.endsWith(domain))
  })
}

function getJobsForCronJob(cronjob, workloads) {
  const jobs = workloads?.jobs?.items || []
  return jobs
    .filter((job) => {
      if (job.namespace !== cronjob.namespace) return false
      return (
        hasOwnerReference(job, 'CronJob', cronjob.name) ||
        hasOwnerReferenceUid(job, cronjob.uid) ||
        getLabel(job, 'cronjob.kubernetes.io/name') === cronjob.name ||
        getLabel(job, 'cronjob-name') === cronjob.name ||
        getAnnotation(job, 'cronjob.kubernetes.io/instantiate') === 'manual' ||
        job.name.startsWith(`${cronjob.name}-`)
      )
    })
    .sort((a, b) =>
      String(b.creation_timestamp || '').localeCompare(
        String(a.creation_timestamp || '')
      )
    )
}

function getReplicaSetsForDeployment(deployment, workloads) {
  const replicaSets = workloads?.replicasets?.items || []
  return replicaSets.filter((replicaSet) => {
    if (replicaSet.namespace !== deployment.namespace) return false
    return (
      hasOwnerReference(replicaSet, 'Deployment', deployment.name) ||
      hasOwnerReferenceUid(replicaSet, deployment.uid)
    )
  })
}

function getPodsForController(resource, workloads) {
  const pods = workloads?.pods?.items || []

  if (resource.kind === 'Job') {
    return getPodsForJob(resource, workloads)
  }

  if (resource.kind === 'ReplicaSet') {
    return pods.filter((pod) => {
      if (pod.namespace !== resource.namespace) return false
      return (
        hasOwnerReference(pod, 'ReplicaSet', resource.name) ||
        hasOwnerReferenceUid(pod, resource.uid)
      )
    })
  }

  if (resource.kind === 'Deployment') {
    const replicaSets = getReplicaSetsForDeployment(resource, workloads)
    const replicaSetNames = new Set(replicaSets.map((replicaSet) => replicaSet.name))
    const replicaSetUids = new Set(replicaSets.map((replicaSet) => replicaSet.uid))
    return pods.filter((pod) => {
      if (pod.namespace !== resource.namespace) return false
      return (
        hasOwnerReference(pod, 'Deployment', resource.name) ||
        hasOwnerReferenceUid(pod, resource.uid) ||
        (pod.owner_references || []).some(
          (ref) =>
            ref.kind === 'ReplicaSet' &&
            (replicaSetNames.has(ref.name) || (ref.uid && replicaSetUids.has(ref.uid)))
        )
      )
    })
  }

  if (resource.kind === 'StatefulSet' || resource.kind === 'DaemonSet') {
    return pods.filter((pod) => {
      if (pod.namespace !== resource.namespace) return false
      return (
        hasOwnerReference(pod, resource.kind, resource.name) ||
        hasOwnerReferenceUid(pod, resource.uid)
      )
    })
  }

  return []
}

function getUpstreamOwners(resource, workloads) {
  const ownerRefs = resource?.owner_references || []
  if (!ownerRefs.length) return []

  const collections = getAllWorkloadResources(workloads)

  return ownerRefs
    .map((ownerRef) =>
      collections.find(
        (candidate) =>
          candidate.kind === ownerRef.kind &&
          candidate.namespace === resource.namespace &&
          (candidate.uid === ownerRef.uid || candidate.name === ownerRef.name)
      )
    )
    .filter(Boolean)
}

function getReferencedByResources(resource, workloads) {
  return dedupeResources(
    getAllWorkloadResources(workloads).filter((candidate) =>
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

function getResourceWithAncestors(resource, workloads, seen = new Set()) {
  if (!resource) return []
  const key = resource.uid || `${resource.kind}:${resource.namespace || ''}:${resource.name}`
  if (seen.has(key)) return []
  seen.add(key)

  const owners = getUpstreamOwners(resource, workloads)
  return [resource, ...owners.flatMap((owner) => getResourceWithAncestors(owner, workloads, seen))]
}

function getControllerResourcesForPods(pods, workloads) {
  return dedupeResources(
    pods.flatMap((pod) => getResourceWithAncestors(pod, workloads)).filter((resource) => resource.kind !== 'Pod')
  )
}

function getRelatedResources(resource, workloads) {
  const dependencyResources = getRelationshipResources(resource, workloads, 'References')
  const referencedByResources = getReferencedByResources(resource, workloads)

  switch (resource.kind) {
    case 'Deployment':
      return dedupeResources([
        ...getUpstreamOwners(resource, workloads),
        ...getReplicaSetsForDeployment(resource, workloads),
        ...getPodsForController(resource, workloads),
        ...dependencyResources,
      ])
    case 'StatefulSet':
    case 'DaemonSet':
      return dedupeResources([
        ...getUpstreamOwners(resource, workloads),
        ...getPodsForController(resource, workloads),
        ...dependencyResources,
      ])
    case 'Job':
      return dedupeResources([
        ...getUpstreamOwners(resource, workloads),
        ...getPodsForController(resource, workloads),
        ...dependencyResources,
      ])
    case 'CronJob':
      return dedupeResources([...getJobsForCronJob(resource, workloads), ...dependencyResources])
    case 'ReplicaSet':
      return dedupeResources([
        ...getUpstreamOwners(resource, workloads),
        ...getPodsForController(resource, workloads),
        ...dependencyResources,
      ])
    case 'Pod':
      return dedupeResources([...getUpstreamOwners(resource, workloads), ...dependencyResources])
    case 'Service':
      return dedupeResources([
        ...getControllerResourcesForPods(getPodsForService(resource, workloads), workloads),
        ...getPodsForService(resource, workloads),
        ...getEndpointsForService(resource, workloads),
        ...getRoutesForService(resource, workloads),
      ])
    case 'Route': {
      const service = getServiceForRoute(resource, workloads)
      const endpoints = service ? getEndpointsForService(service, workloads) : []
      const pods = service ? getPodsForService(service, workloads) : []
      return dedupeResources([
        ...(service ? [service] : []),
        ...endpoints,
        ...getControllerResourcesForPods(pods, workloads),
        ...pods,
        ...getIngressControllersForRoute(resource, workloads),
      ])
    }
    case 'Endpoints': {
      const services = workloads?.services?.items || []
      const service =
        services.find(
          (candidate) =>
            candidate.namespace === resource.namespace &&
            candidate.name === resource.name
        ) || null
      const pods = service ? getPodsForService(service, workloads) : []
      return dedupeResources([
        ...(service ? [service] : []),
        ...getControllerResourcesForPods(pods, workloads),
        ...pods,
      ])
    }
    case 'ConfigMap':
    case 'Secret':
      return referencedByResources
    default:
      return []
  }
}

function getRelatedResourceSections(resource, workloads) {
  const upstreamOwners = dedupeResources(getUpstreamOwners(resource, workloads))
  const dependencyResources = dedupeResources(
    getRelationshipResources(resource, workloads, 'References')
  )
  const referencedByResources = dedupeResources(getReferencedByResources(resource, workloads))

  switch (resource.kind) {
    case 'Deployment':
      return [
        { title: 'Upstream Owners', resources: upstreamOwners },
        { title: 'ReplicaSets', resources: dedupeResources(getReplicaSetsForDeployment(resource, workloads)) },
        { title: 'Pods', resources: dedupeResources(getPodsForController(resource, workloads)) },
        {
          title: 'Uses ConfigMaps',
          resources: dependencyResources.filter((item) => item.kind === 'ConfigMap'),
        },
        {
          title: 'Uses Secrets',
          resources: dependencyResources.filter((item) => item.kind === 'Secret'),
        },
      ].filter((section) => section.resources.length > 0)
    case 'StatefulSet':
    case 'DaemonSet':
      return [
        { title: 'Upstream Owners', resources: upstreamOwners },
        { title: 'Pods', resources: dedupeResources(getPodsForController(resource, workloads)) },
        {
          title: 'Uses ConfigMaps',
          resources: dependencyResources.filter((item) => item.kind === 'ConfigMap'),
        },
        {
          title: 'Uses Secrets',
          resources: dependencyResources.filter((item) => item.kind === 'Secret'),
        },
      ].filter((section) => section.resources.length > 0)
    case 'Job':
      return [
        { title: 'Upstream Owners', resources: upstreamOwners },
        { title: 'Pods', resources: dedupeResources(getPodsForController(resource, workloads)) },
        {
          title: 'Uses ConfigMaps',
          resources: dependencyResources.filter((item) => item.kind === 'ConfigMap'),
        },
        {
          title: 'Uses Secrets',
          resources: dependencyResources.filter((item) => item.kind === 'Secret'),
        },
      ].filter((section) => section.resources.length > 0)
    case 'CronJob':
      return [
        { title: 'Recent Jobs', resources: dedupeResources(getJobsForCronJob(resource, workloads)) },
        {
          title: 'Uses ConfigMaps',
          resources: dependencyResources.filter((item) => item.kind === 'ConfigMap'),
        },
        {
          title: 'Uses Secrets',
          resources: dependencyResources.filter((item) => item.kind === 'Secret'),
        },
      ].filter((section) => section.resources.length > 0)
    case 'ReplicaSet':
      return [
        { title: 'Upstream Owners', resources: upstreamOwners },
        { title: 'Pods', resources: dedupeResources(getPodsForController(resource, workloads)) },
        {
          title: 'Uses ConfigMaps',
          resources: dependencyResources.filter((item) => item.kind === 'ConfigMap'),
        },
        {
          title: 'Uses Secrets',
          resources: dependencyResources.filter((item) => item.kind === 'Secret'),
        },
      ].filter((section) => section.resources.length > 0)
    case 'Pod':
      return [
        { title: 'Upstream Owners', resources: upstreamOwners },
        {
          title: 'Uses ConfigMaps',
          resources: dependencyResources.filter((item) => item.kind === 'ConfigMap'),
        },
        {
          title: 'Uses Secrets',
          resources: dependencyResources.filter((item) => item.kind === 'Secret'),
        },
      ].filter((section) => section.resources.length > 0)
    case 'Service':
      return [
        {
          title: 'Workload Controllers',
          resources: getControllerResourcesForPods(getPodsForService(resource, workloads), workloads),
        },
        { title: 'Pods', resources: dedupeResources(getPodsForService(resource, workloads)) },
        { title: 'Endpoints', resources: dedupeResources(getEndpointsForService(resource, workloads)) },
        { title: 'Routes', resources: dedupeResources(getRoutesForService(resource, workloads)) },
      ].filter((section) => section.resources.length > 0)
    case 'Endpoints': {
      const services = workloads?.services?.items || []
      const service =
        services.find(
          (candidate) =>
            candidate.namespace === resource.namespace &&
            candidate.name === resource.name
        ) || null
      const pods = service ? getPodsForService(service, workloads) : []
      return [
        { title: 'Service', resources: dedupeResources(service ? [service] : []) },
        {
          title: 'Workload Controllers',
          resources: getControllerResourcesForPods(pods, workloads),
        },
        { title: 'Pods', resources: dedupeResources(pods) },
      ].filter((section) => section.resources.length > 0)
    }
    case 'Route': {
      const service = getServiceForRoute(resource, workloads)
      const endpoints = service ? getEndpointsForService(service, workloads) : []
      const pods = service ? getPodsForService(service, workloads) : []
      return [
        { title: 'Service', resources: dedupeResources(service ? [service] : []) },
        { title: 'Endpoints', resources: dedupeResources(endpoints) },
        {
          title: 'Workload Controllers',
          resources: getControllerResourcesForPods(pods, workloads),
        },
        { title: 'Pods', resources: dedupeResources(pods) },
        {
          title: 'IngressControllers',
          resources: dedupeResources(getIngressControllersForRoute(resource, workloads)),
        },
      ].filter((section) => section.resources.length > 0)
    }
    case 'ConfigMap':
    case 'Secret':
      return [{ title: 'Used By', resources: referencedByResources }].filter(
        (section) => section.resources.length > 0
      )
    case 'ClusterOperator': {
      const relatedObjects = (resource.relationships || []).map((relationship) => ({
        ...relationship,
        resolved: resolveResourceReference(relationship, workloads, relationship.namespace),
      }))

      const sections = [
        {
          title: 'Deployments',
          resources: relatedObjects.filter((item) => item.kind === 'Deployment'),
        },
        {
          title: 'ConfigMaps',
          resources: relatedObjects.filter((item) => item.kind === 'ConfigMap'),
        },
        {
          title: 'Secrets',
          resources: relatedObjects.filter((item) => item.kind === 'Secret'),
        },
        {
          title: 'Services',
          resources: relatedObjects.filter((item) => item.kind === 'Service'),
        },
        {
          title: 'Namespaces',
          resources: relatedObjects.filter((item) => item.kind === 'Namespace'),
        },
        {
          title: 'Other Related Objects',
          resources: relatedObjects.filter(
            (item) =>
              !['Deployment', 'ConfigMap', 'Secret', 'Service', 'Namespace'].includes(item.kind)
          ),
        },
      ]

      return sections.filter((section) => section.resources.length > 0)
    }
    default:
      return []
  }
}

function getFailedPods(pods) {
  return pods.filter((pod) => pod.status === 'Error' || (pod.errors?.length || 0) > 0)
}

function dedupeResources(resources) {
  const seen = new Set()
  return resources.filter((resource) => {
    const key = resource?.uid || `${resource?.kind}:${resource?.namespace || ''}:${resource?.name || ''}`
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
}

function getTopologyChildren(resource, workloads) {
  const dependencyResources = getRelationshipResources(resource, workloads, 'References')

  switch (resource.kind) {
    case 'Deployment':
      return dedupeResources([
        ...getReplicaSetsForDeployment(resource, workloads),
        ...getPodsForController(resource, workloads),
        ...dependencyResources,
      ])
    case 'StatefulSet':
    case 'DaemonSet':
    case 'Job':
      return dedupeResources([
        ...getPodsForController(resource, workloads),
        ...dependencyResources,
      ])
    case 'CronJob': {
      const jobs = getJobsForCronJob(resource, workloads)
      const pods = jobs.flatMap((job) => getPodsForJob(job, workloads))
      return dedupeResources([...jobs, ...pods, ...dependencyResources])
    }
    case 'ReplicaSet':
      return dedupeResources([
        ...getPodsForController(resource, workloads),
        ...dependencyResources,
      ])
    case 'Pod':
      return dependencyResources
    case 'Service':
      return dedupeResources([
        ...getRoutesForService(resource, workloads),
        ...getEndpointsForService(resource, workloads),
        ...getPodsForService(resource, workloads),
      ])
    case 'Route': {
      const service = getServiceForRoute(resource, workloads)
      return dedupeResources([
        ...(service ? [service] : []),
        ...getIngressControllersForRoute(resource, workloads),
      ])
    }
    case 'Endpoints': {
      const services = workloads?.services?.items || []
      const matchingService = services.find(
        (service) =>
          service.namespace === resource.namespace &&
          service.name === resource.name
      )
      return dedupeResources(matchingService ? [matchingService] : [])
    }
    default:
      return []
  }
}

function getTopologyResources(workloads, selectedNamespace) {
  const namespaceMatches = (resource) =>
    selectedNamespace === 'all' || resource.namespace === selectedNamespace

  const deployments = (workloads?.deployments?.items || []).filter(namespaceMatches)
  const statefulsets = (workloads?.statefulsets?.items || []).filter(namespaceMatches)
  const daemonsets = (workloads?.daemonsets?.items || []).filter(namespaceMatches)
  const cronjobs = (workloads?.cronjobs?.items || []).filter(namespaceMatches)
  const services = (workloads?.services?.items || []).filter(namespaceMatches)
  const routes = (workloads?.routes?.items || []).filter(namespaceMatches)
  const standaloneEndpoints = (workloads?.endpoints?.items || [])
    .filter(namespaceMatches)
    .filter((endpoint) => {
      const service = (workloads?.services?.items || []).find(
        (candidate) =>
          candidate.namespace === endpoint.namespace &&
          candidate.name === endpoint.name
      )
      return !service
    })
  const standaloneJobs = (workloads?.jobs?.items || [])
    .filter(namespaceMatches)
    .filter((job) => !getUpstreamOwners(job, workloads).some((owner) => owner.kind === 'CronJob'))
  const standalonePods = (workloads?.pods?.items || [])
    .filter(namespaceMatches)
    .filter((pod) => getUpstreamOwners(pod, workloads).length === 0)

  return dedupeResources([
    ...deployments,
    ...statefulsets,
    ...daemonsets,
    ...cronjobs,
    ...services,
    ...routes,
    ...standaloneEndpoints,
    ...standaloneJobs,
    ...standalonePods,
  ])
}

function applyStatusFilter(resources, statusFilter) {
  if (statusFilter === 'all') return resources

  return resources.filter((r) => {
    const hasErrors = r.errors && r.errors.length > 0
    const hasWarnings = r.warnings && r.warnings.length > 0
    const isError = r.status === 'Error' || hasErrors
    const isWarning = r.status === 'Warning' || hasWarnings
    const isHealthy = r.status === 'Healthy' && !hasErrors && !hasWarnings

    switch (statusFilter) {
      case 'error':
        return isError
      case 'warning':
        return isWarning
      case 'healthy':
        return isHealthy
      default:
        return true
    }
  })
}

function applySearchFilter(resources, searchTerm) {
  if (!searchTerm) return resources
  const search = searchTerm.toLowerCase()
  return resources.filter((r) =>
    r.name?.toLowerCase().includes(search) ||
    r.namespace?.toLowerCase().includes(search) ||
    r.kind?.toLowerCase().includes(search)
  )
}

function resourceKindToTab(kind) {
  switch (kind) {
    case 'ClusterOperator':
      return 'operators'
    case 'Pod':
      return 'pods'
    case 'ConfigMap':
      return 'configmaps'
    case 'Secret':
      return 'secrets'
    case 'Service':
      return 'services'
    case 'Endpoints':
      return 'endpoints'
    case 'NetworkPolicy':
      return 'networkpolicies'
    case 'Route':
      return 'routes'
    case 'Job':
      return 'jobs'
    case 'CronJob':
      return 'cronjobs'
    case 'Deployment':
      return 'deployments'
    case 'StatefulSet':
      return 'statefulsets'
    case 'DaemonSet':
      return 'daemonsets'
    case 'ReplicaSet':
      return 'replicasets'
    default:
      return null
  }
}

function getRelatedSectionVariant(title) {
  if (title.includes('Upstream')) return 'owner'
  if (title.includes('Uses ConfigMaps') || title.includes('Uses Secrets')) return 'dependency'
  if (title.includes('Used By')) return 'reverse'
  if (
    title.includes('Service') ||
    title.includes('Services') ||
    title.includes('Route') ||
    title.includes('Routes') ||
    title.includes('Endpoint') ||
    title.includes('IngressController')
  ) {
    return 'network'
  }
  return 'child'
}

function RelatedResourceButton({ resource, onOpen, variant = 'child' }) {
  const variantClasses = {
    owner: 'border-sky-500/30 bg-sky-500/10 text-sky-100 hover:border-sky-400 hover:bg-sky-500/20',
    child: 'border-violet-500/25 bg-violet-500/10 text-violet-100 hover:border-violet-400 hover:bg-violet-500/20',
    dependency: 'border-amber-500/30 bg-amber-500/10 text-amber-100 hover:border-amber-400 hover:bg-amber-500/20',
    reverse: 'border-emerald-500/30 bg-emerald-500/10 text-emerald-100 hover:border-emerald-400 hover:bg-emerald-500/20',
    network: 'border-cyan-500/30 bg-cyan-500/10 text-cyan-100 hover:border-cyan-400 hover:bg-cyan-500/20',
  }

  return (
    <button
      onClick={() => onOpen?.(resource)}
      disabled={!onOpen}
      className={`inline-flex max-w-full items-center gap-2 rounded-lg border px-3 py-2 text-left text-sm transition-colors ${variantClasses[variant] || variantClasses.child}`}
    >
      <span
        className={`min-w-0 ${getNameWrapClass(resource.name)} font-medium`}
        {...getCollapsedNameProps(resource.name)}
      >
        {resource.name}
      </span>
      <span className="text-xs opacity-75">{resource.kind}</span>
      <span className="text-xs opacity-60">{resource.status}</span>
    </button>
  )
}

function RelatedObjectEntry({ item, variant = 'child', onOpenResource }) {
  const resolvedResource = item?.resolved || null
  const displayResource =
    resolvedResource ||
    {
      name: item.name,
      kind: item.kind,
      namespace: item.namespace,
      status: 'Reference',
    }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <RelatedResourceButton
        resource={displayResource}
        onOpen={resolvedResource ? onOpenResource : null}
        variant={variant}
      />
      {!resolvedResource && (
        <span className="rounded-lg border border-slate-700 bg-slate-900/70 px-2 py-1 text-xs text-slate-400">
          Not indexed in this view
        </span>
      )}
    </div>
  )
}

function ResourceLogCard({ resource, onOpen }) {
  const { logs, isLoading, error } = useResourceLogs(resource, true)

  return (
    <div className="rounded-2xl border border-slate-800 bg-slate-950/60">
      <div className="flex items-center justify-between gap-3 border-b border-slate-800 px-4 py-3">
        <div>
          <div
            className={`text-sm font-semibold text-slate-200 ${getNameWrapClass(resource.name)}`}
            {...getCollapsedNameProps(resource.name)}
          >
            {resource.name}
          </div>
          <div className="text-xs text-slate-500">
            {resource.kind}
            {resource.namespace ? ` • ${resource.namespace}` : ''}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <RelatedResourceButton resource={resource} onOpen={onOpen} />
        </div>
      </div>
      <div className="space-y-3 p-4">
        {isLoading ? (
          <p className="text-sm text-slate-400">Loading logs…</p>
        ) : error ? (
          <p className="text-sm text-red-400">{error}</p>
        ) : logs.length ? (
          <ContainerLogs
            logs={logs}
            podName={resource.name}
            namespace={resource.namespace}
            viewerClassName="max-h-72"
          />
        ) : (
          <p className="text-sm text-slate-400">No logs available for this resource.</p>
        )}
      </div>
    </div>
  )
}

function RelatedWorkloadLogs({
  resources,
  onOpen,
  title = 'Related Logs',
  emptyMessage = 'No logs available for this resource.',
}) {
  if (!resources.length) {
    return (
      <Card>
        <p className="text-slate-400">{emptyMessage}</p>
      </Card>
    )
  }

  return (
    <Card>
      <div className="mb-4">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-slate-400">
          {title} ({resources.length})
        </h3>
      </div>
      <div className="space-y-4">
        {resources.map((resource) => (
          <ResourceLogCard key={resource.uid || resource.name} resource={resource} onOpen={onOpen} />
        ))}
      </div>
    </Card>
  )
}

function JobLogsTab({ resource, workloads, onOpenResource }) {
  if (resource.kind === 'Job') {
    const relatedPods = getPodsForJob(resource, workloads)
    const failedPods = getFailedPods(relatedPods)
    const healthyPods = relatedPods.filter(
      (pod) => !(pod.status === 'Error' || (pod.errors?.length || 0) > 0)
    )

    return (
      <RelatedWorkloadLogs
        resources={[...failedPods, ...healthyPods]}
        onOpen={onOpenResource}
        title="Related Pod Logs"
        emptyMessage="No pod logs available for this job."
      />
    )
  }

  if (resource.kind === 'CronJob') {
    const relatedJobs = getJobsForCronJob(resource, workloads)
    const failedJobs = relatedJobs.filter(
      (job) => job.status === 'Error' || (job.errors?.length || 0) > 0
    )
    const remainingJobs = relatedJobs.filter(
      (job) => !(job.status === 'Error' || (job.errors?.length || 0) > 0)
    )
    const orderedPods = [...failedJobs, ...remainingJobs].flatMap((job) =>
      getPodsForJob(job, workloads)
    )
    const failedPods = getFailedPods(orderedPods)
    const healthyPods = orderedPods.filter(
      (pod) => !(pod.status === 'Error' || (pod.errors?.length || 0) > 0)
    )

    return (
      <RelatedWorkloadLogs
        resources={[...failedPods, ...healthyPods]}
        onOpen={onOpenResource}
        title="Related Pod Logs"
        emptyMessage="No related pod logs available for this cronjob."
      />
    )
  }

  return (
    <Card>
      <p className="text-slate-400">No logs available for this resource.</p>
    </Card>
  )
}

function RelatedResourcesTab({ resource, workloads, onOpenResource }) {
  const relatedSections = getRelatedResourceSections(resource, workloads)
  const totalRelated = relatedSections.reduce((sum, section) => sum + section.resources.length, 0)

  if (relatedSections.length) {
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
                {section.resources.map((related) => {
                  const key =
                    related.uid ||
                    `${related.kind}-${related.namespace || ''}-${related.name}`
                  const variant = getRelatedSectionVariant(section.title)

                  if (resource.kind === 'ClusterOperator') {
                    return (
                      <RelatedObjectEntry
                        key={key}
                        item={related}
                        variant={variant}
                        onOpenResource={onOpenResource}
                      />
                    )
                  }

                  return (
                    <RelatedResourceButton
                      key={key}
                      resource={related}
                      onOpen={onOpenResource}
                      variant={variant}
                    />
                  )
                })}
              </div>
            </div>
          ))}
        </div>
      </Card>
    )
  }

  return (
    <Card>
      <p className="text-slate-400">No related resources available.</p>
    </Card>
  )
}

function WorkloadAnalysisExtras({ resource, workloads, onOpenResource }) {
  if (resource.kind === 'Job') {
    const relatedPods = getPodsForJob(resource, workloads)
    const failedPods = getFailedPods(relatedPods)
    const healthyPods = relatedPods.filter(
      (pod) => !(pod.status === 'Error' || (pod.errors?.length || 0) > 0)
    )

    return (
      <div className="space-y-6">
        <Card>
          <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Related Pods ({relatedPods.length})
          </h3>
          {relatedPods.length ? (
            <div className="flex flex-wrap gap-2">
              {relatedPods.map((pod) => (
                <RelatedResourceButton key={pod.uid || pod.name} resource={pod} onOpen={onOpenResource} />
              ))}
            </div>
          ) : (
            <p className="text-sm text-slate-400">No related pods found for this job.</p>
          )}
        </Card>
        <RelatedWorkloadLogs
          resources={[...failedPods, ...healthyPods]}
          onOpen={onOpenResource}
          title="Related Pod Logs"
          emptyMessage="No pod logs available for this job."
        />
      </div>
    )
  }

  if (resource.kind === 'CronJob') {
    const relatedJobs = getJobsForCronJob(resource, workloads)
    const failedJobs = relatedJobs.filter((job) => job.status === 'Error' || (job.errors?.length || 0) > 0)
    const latestFailedJob = failedJobs[0] || null
    const relatedPods = relatedJobs.flatMap((job) => getPodsForJob(job, workloads))
    const failedPods = getFailedPods(relatedPods)
    const healthyPods = relatedPods.filter(
      (pod) => !(pod.status === 'Error' || (pod.errors?.length || 0) > 0)
    )

    return (
      <div className="space-y-6">
        <Card>
          <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Recent Jobs ({relatedJobs.length})
          </h3>
          {relatedJobs.length ? (
            <div className="flex flex-wrap gap-2">
              {relatedJobs.map((job) => (
                <RelatedResourceButton key={job.uid || job.name} resource={job} onOpen={onOpenResource} />
              ))}
            </div>
          ) : (
            <p className="text-sm text-slate-400">No related jobs found for this cronjob.</p>
          )}
        </Card>
        {latestFailedJob && (
          <Card>
            <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-slate-400">
              Latest Failed Job
            </h3>
            <RelatedResourceButton resource={latestFailedJob} onOpen={onOpenResource} />
          </Card>
        )}
        <RelatedWorkloadLogs
          resources={[...failedPods, ...healthyPods]}
          onOpen={onOpenResource}
          title="Related Pod Logs"
          emptyMessage="No related pod logs available for this cronjob."
        />
      </div>
    )
  }

  return null
}

function ResourceDetailsPanel({ resource, workloads, onOpenResource }) {
  const { resource: detailedResource, isLoading, error } = useResourceDetail(resource)
  const displayResource = detailedResource || resource || {
    uid: '',
    name: '',
    kind: '',
    status: 'Healthy',
    namespace: null,
    errors: [],
    warnings: [],
    metadata: {},
    key_fields: {},
    relationships: [],
    owner_references: [],
    labels: {},
    annotations: {},
  }
  const [activeTab, setActiveTab] = React.useState(0)
  const metadataEntries = Object.entries(displayResource.metadata || {})
  const jobRelatedPods =
    displayResource.kind === 'Job' ? getPodsForJob(displayResource, workloads) : []
  const cronjobRelatedJobs =
    displayResource.kind === 'CronJob' ? getJobsForCronJob(displayResource, workloads) : []
  const supportsRelatedTab = ['Deployment', 'StatefulSet', 'DaemonSet', 'Job', 'CronJob', 'ReplicaSet', 'Pod', 'ConfigMap', 'Secret', 'Service', 'Route', 'ClusterOperator'].includes(displayResource.kind)
  const relatedSections =
    supportsRelatedTab ? getRelatedResourceSections(displayResource, workloads) : []
  const workloadLogCount =
    displayResource.kind === 'Job'
      ? jobRelatedPods.length
      : displayResource.kind === 'CronJob'
        ? cronjobRelatedJobs.flatMap((job) => getPodsForJob(job, workloads)).length
        : 0
  const relatedResourceCount =
    relatedSections.reduce((sum, section) => sum + section.resources.length, 0)
  const tabsWithLabels = [
    'YAML',
    ...(supportsRelatedTab ? [`Related (${relatedResourceCount})`] : []),
    ...(['Job', 'CronJob'].includes(displayResource.kind) ? [`Logs (${workloadLogCount})`] : []),
    ...(displayResource.kind === 'Pod' ? ['Logs'] : []),
    'Analysis',
    `Errors (${displayResource.errors?.length || 0})`,
    `Warnings (${displayResource.warnings?.length || 0})`,
    'Metadata',
  ]
  const activeTabLabel = tabsWithLabels[activeTab] || 'YAML'
  const shouldLoadRaw = activeTabLabel === 'YAML'
  const shouldLoadPodLogs =
    displayResource.kind === 'Pod' &&
    (activeTabLabel === 'Logs' || activeTabLabel === 'Analysis')
  const {
    raw: loadedRaw,
    isLoading: isLoadingRaw,
    error: rawError,
  } = useResourceRaw(displayResource, !!resource && shouldLoadRaw)
  const {
    logs: loadedPodLogs,
    isLoading: isLoadingPodLogs,
    error: podLogsError,
  } = useResourceLogs(displayResource, !!resource && shouldLoadPodLogs)

  React.useEffect(() => {
    setActiveTab(0)
  }, [resource?.uid])

  const [collapsedSections, setCollapsedSections] = React.useState({
    errors: false,
    warnings: false,
    metadata: false,
  })

  const toggleSection = (section) => {
    setCollapsedSections(prev => ({
      ...prev,
      [section]: !prev[section]
    }))
  }

  if (!resource) {
    return (
      <Card className="flex min-h-[24rem] items-center justify-center">
        <div className="text-center text-slate-400">
          Select a workload resource to inspect YAML, analysis, and metadata.
        </div>
      </Card>
    )
  }

  const CollapsibleSection = ({ title, children, section, count }) => {
    const isCollapsed = collapsedSections[section]
    return (
      <Card>
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-semibold text-white">{title} {count !== undefined && `(${count})`}</h3>
          <button
            onClick={() => toggleSection(section)}
            className="text-xs text-slate-400 hover:text-white transition-colors"
          >
            {isCollapsed ? '▼ Expand' : '▲ Collapse'}
          </button>
        </div>
        {!isCollapsed && children}
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
        <div className="h-[70vh] min-h-[24rem]">
          <YAMLViewer content={loadedRaw || '# No YAML available'} />
        </div>
      ),
    },
    ...(supportsRelatedTab ? [{
      label: `Related (${relatedResourceCount})`,
      content: (
        <RelatedResourcesTab
          resource={displayResource}
          workloads={workloads}
          onOpenResource={onOpenResource}
        />
      ),
    }] : []),
    ...(['Job', 'CronJob'].includes(displayResource.kind) ? [{
      label: `Logs (${workloadLogCount})`,
      content: (
        <JobLogsTab
          resource={displayResource}
          workloads={workloads}
          onOpenResource={onOpenResource}
        />
      ),
    }] : []),
    ...(displayResource.kind === 'Pod' ? [{
      label: 'Logs',
      content: isLoadingPodLogs ? (
        <Card>
          <p className="text-slate-400">Loading pod logs…</p>
        </Card>
      ) : podLogsError ? (
        <Card>
          <p className="text-red-400">{podLogsError}</p>
        </Card>
      ) : loadedPodLogs.length ? (
        <ContainerLogs
          logs={loadedPodLogs}
          podName={displayResource.name}
          namespace={displayResource.namespace}
          viewer="yaml"
          viewerClassName="h-[60vh] min-h-[20rem]"
        />
      ) : (
        <Card>
          <p className="text-slate-400">No pod logs available for this resource.</p>
        </Card>
      ),
    }] : []),
    {
      label: 'Analysis',
      content: displayResource.health_analysis ? (
        <div className="space-y-6">
          <HealthAnalysis
            analysis={displayResource.health_analysis}
            logs={displayResource.kind === 'Pod' ? loadedPodLogs : []}
            podName={displayResource.kind === 'Pod' ? displayResource.name : null}
            namespace={displayResource.kind === 'Pod' ? displayResource.namespace : null}
          />
          <WorkloadAnalysisExtras
            resource={displayResource}
            workloads={workloads}
            onOpenResource={onOpenResource}
          />
        </div>
      ) : (
        <Card>
          <p className="text-slate-400">No health analysis available for this resource.</p>
        </Card>
      ),
    },
    {
      label: `Errors (${displayResource.errors?.length || 0})`,
      content: (
        <CollapsibleSection title="Errors" section="errors" count={displayResource.errors?.length || 0}>
          {displayResource.errors?.length ? (
            <div className="space-y-2 max-h-[400px] overflow-y-auto">
              {displayResource.errors.map((error, idx) => (
                <div key={idx} className="rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-300">
                  {error}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-slate-400">No errors reported.</p>
          )}
        </CollapsibleSection>
      ),
    },
    {
      label: `Warnings (${displayResource.warnings?.length || 0})`,
      content: (
        <CollapsibleSection title="Warnings" section="warnings" count={displayResource.warnings?.length || 0}>
          {displayResource.warnings?.length ? (
            <div className="space-y-2 max-h-[400px] overflow-y-auto">
              {displayResource.warnings.map((warning, idx) => (
                <div key={idx} className="rounded-xl border border-amber-500/20 bg-amber-500/10 p-3 text-sm text-amber-300">
                  {warning}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-slate-400">No warnings reported.</p>
          )}
        </CollapsibleSection>
      ),
    },
    {
      label: 'Metadata',
      content: (
        <CollapsibleSection title="Metadata" section="metadata" count={metadataEntries.length}>
          {metadataEntries.length ? (
            <div className="space-y-2 max-h-[400px] overflow-y-auto">
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
        </CollapsibleSection>
      ),
    },
  ]

  return (
    <div className="space-y-2">
      <Card className="px-4 py-3">
        <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
          <h2 className={`${getNameWrapClass(displayResource.name)} min-w-0 text-lg font-semibold text-white`}>{displayResource.name}</h2>
          <StatusBadge status={displayResource.status} size="sm">{displayResource.status}</StatusBadge>
          <span className="rounded-lg bg-slate-800 px-2 py-0.5 text-xs text-slate-300">{displayResource.kind || 'Resource'}</span>
          {displayResource.namespace && (
            <span className="break-words text-sm text-slate-400">Namespace: <span className="text-slate-200">{displayResource.namespace}</span></span>
          )}
          {isLoading && (
            <p className="text-sm text-slate-400">Loading resource details…</p>
          )}
          {error && (
            <p className="text-sm text-red-400">{error}</p>
          )}
        </div>
      </Card>
      <Tabs tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} />
    </div>
  )
}

function ResourceFilterBar({ searchTerm, onSearchChange, statusFilter, onStatusFilterChange, resourceCount }) {
  const statusOptions = [
    { value: 'all', label: 'All', color: 'slate' },
    { value: 'error', label: 'Errors', color: 'red' },
    { value: 'warning', label: 'Warnings', color: 'amber' },
    { value: 'healthy', label: 'Healthy', color: 'green' },
  ]

  return (
    <Card className="mb-2 px-3 py-2.5">
      <div className="space-y-2">
        {/* Search input - full width on its own row */}
        <div className="relative">
          <div className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500">
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

        {/* Status filter buttons */}
        <div className="flex flex-wrap gap-1.5">
          {statusOptions.map((option) => (
            <button
              key={option.value}
              onClick={() => onStatusFilterChange(option.value)}
              className={`rounded-lg px-2 py-1 text-xs font-medium whitespace-nowrap transition-colors ${
                statusFilter === option.value
                  ? `bg-${option.color}-500/20 text-${option.color}-400 border border-${option.color}-500/50`
                  : 'text-slate-400 hover:text-white hover:bg-slate-800'
              }`}
            >
              {option.label}
            </button>
          ))}
          <div className="ml-auto text-xs text-slate-500 flex items-center">
            Showing {resourceCount} resource{resourceCount !== 1 ? 's' : ''}
          </div>
        </div>
      </div>
    </Card>
  )
}

function TopologyNode({ resource, onOpen, compact = false, selected = false, theme = 'dark' }) {
  return (
    <button
      onClick={() => onOpen(resource)}
      className={`w-full rounded-2xl border bg-slate-900/80 p-3.5 text-left transition-colors ${
        selected
          ? theme === 'light'
            ? 'resource-selected border-blue-500 ring-1 ring-blue-500/40'
            : 'resource-selected border-red-500 ring-1 ring-red-500/50'
          : 'border-slate-800 hover:border-slate-600 hover:bg-slate-900'
      } ${compact ? 'min-w-[15rem]' : ''
      }`}
    >
      <div className={`${compact ? 'space-y-2.5' : 'space-y-3'}`}>
        <div className="min-w-0">
          <div
            className={`font-semibold text-white ${compact ? 'text-sm' : 'text-base'} ${getNameWrapClass(resource.name)}`}
            {...getCollapsedNameProps(resource.name, selected)}
          >
            {resource.name}
          </div>
          <div className="mt-1 text-xs text-slate-500">
            {resource.kind}
            {resource.namespace ? ` • ${resource.namespace}` : ''}
          </div>
        </div>
        <div className="flex justify-start">
          <StatusBadge status={resource.status} size="sm">
            {resource.status}
          </StatusBadge>
        </div>
      </div>
    </button>
  )
}

function TopologyLane({ title, resources, onOpenResource, theme = 'dark' }) {
  if (!resources.length) return null

  return (
    <div className="space-y-2.5">
      <div className="text-xs font-semibold uppercase tracking-wider text-slate-500">
        {title} ({resources.length})
      </div>
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-2 2xl:grid-cols-3">
        {resources.map((resource) => (
          <TopologyNode
            key={resource.uid || `${resource.kind}-${resource.name}`}
            resource={resource}
            onOpen={onOpenResource}
            compact
            theme={theme}
          />
        ))}
      </div>
    </div>
  )
}

function TopologyResourceList({ resources, selectedResource, onResourceClick, theme = 'dark' }) {
  if (!resources.length) {
    return (
      <Card>
        <p className="text-slate-400">No topology resources found for the current filters.</p>
      </Card>
    )
  }

  return (
    <div className="space-y-1.5">
      {resources.map((resource) => {
        const isSelected =
          selectedResource?.name === resource.name &&
          selectedResource?.namespace === resource.namespace &&
          selectedResource?.kind === resource.kind

        return (
          <TopologyNode
            key={resource.uid || `${resource.kind}-${resource.name}`}
            resource={resource}
            onOpen={onResourceClick}
            compact
            selected={isSelected}
            theme={theme}
          />
        )
      })}
    </div>
  )
}

function TopologyBoard({ resources, workloads, onOpenResource, theme = 'dark' }) {
  if (!resources.length) {
    return (
      <Card>
        <p className="text-slate-400">No topology resources found for the current filters.</p>
      </Card>
    )
  }

  return (
    <div className="space-y-4">
      {resources.map((resource) => {
        const upstreamOwners = getUpstreamOwners(resource, workloads)
        const children = getTopologyChildren(resource, workloads)
        const childGroups = {
          Job: children.filter((child) => child.kind === 'Job'),
          ReplicaSet: children.filter((child) => child.kind === 'ReplicaSet'),
          Pod: children.filter((child) => child.kind === 'Pod'),
          Service: children.filter((child) => child.kind === 'Service'),
          Route: children.filter((child) => child.kind === 'Route'),
          Endpoints: children.filter((child) => child.kind === 'Endpoints'),
          IngressController: children.filter((child) => child.kind === 'IngressController'),
          ConfigMap: children.filter((child) => child.kind === 'ConfigMap'),
          Secret: children.filter((child) => child.kind === 'Secret'),
        }

        return (
          <Card key={resource.uid || `${resource.kind}-${resource.name}`}>
            <div className="space-y-6">
              {upstreamOwners.length > 0 && (
                <TopologyLane
                  title="Upstream"
                  resources={upstreamOwners}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
              )}

              <div className="space-y-3">
                <div className="text-xs font-semibold uppercase tracking-wider text-slate-500">
                  Selected
                </div>
                <div className="max-w-2xl">
                  <TopologyNode resource={resource} onOpen={onOpenResource} theme={theme} />
                </div>
              </div>

              <div className="grid gap-6">
                <TopologyLane
                  title="ReplicaSets"
                  resources={childGroups.ReplicaSet}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="Jobs"
                  resources={childGroups.Job}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="Pods"
                  resources={childGroups.Pod}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="Services"
                  resources={childGroups.Service}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="Routes"
                  resources={childGroups.Route}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="Endpoints"
                  resources={childGroups.Endpoints}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="IngressControllers"
                  resources={childGroups.IngressController}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="Uses ConfigMaps"
                  resources={childGroups.ConfigMap}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
                <TopologyLane
                  title="Uses Secrets"
                  resources={childGroups.Secret}
                  onOpenResource={onOpenResource}
                  theme={theme}
                />
              </div>
            </div>
          </Card>
        )
      })}
    </div>
  )
}

function WorkloadResourceList({ resources, type, selectedResource, onResourceClick, theme = 'dark' }) {
  if (!resources || resources.length === 0) {
    return (
      <div className="rounded-lg border border-slate-800 bg-slate-900 p-8 text-center">
        <p className="text-slate-400">No {type} found</p>
      </div>
    )
  }

  return (
    <div className="space-y-1.5">
      {resources.map((resource, index) => {
        const isSelected =
          selectedResource?.name === resource.name &&
          selectedResource?.namespace === resource.namespace &&
          selectedResource?.kind === resource.kind
        const hasIssues =
          resource.status === 'Error' ||
          resource.status === 'Warning' ||
          (resource.errors?.length || 0) > 0 ||
          (resource.warnings?.length || 0) > 0
        return (
        <Card
          key={index}
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
            <div className="min-w-0 space-y-1.5">
              <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                <h3
                  className={`min-w-0 text-sm font-semibold leading-snug text-white ${getNameWrapClass(resource.name)}`}
                  style={isSelected ? undefined : getCollapsedNameStyle()}
                  title={resource.name}
                >
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

              {/* Type-specific details */}
              <div className={hasIssues ? 'contents' : 'hidden'}>
              {type === 'deployments' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {(getNumberField(resource, 'ready_replicas') !== undefined || getNumberField(resource, 'replicas') !== undefined) && (
                    <span className="text-slate-400">
                      Replicas: <span className="text-white">{getNumberField(resource, 'ready_replicas') || 0}/{getNumberField(resource, 'replicas') || 0}</span>
                    </span>
                  )}
                  {getNumberField(resource, 'available_replicas') !== undefined && (
                    <span className="text-slate-400">
                      Available: <span className="text-white">{getNumberField(resource, 'available_replicas')}</span>
                    </span>
                  )}
                </div>
              )}

              {type === 'statefulsets' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Replicas: <span className="text-white">{getNumberField(resource, 'ready_replicas') || 0}/{getNumberField(resource, 'replicas') || 0}</span>
                  </span>
                </div>
              )}

              {type === 'services' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Type: <span className="text-white">{getKeyField(resource, 'type') || 'ClusterIP'}</span>
                  </span>
                  {getKeyField(resource, 'cluster_ip') && (
                    <span className="text-slate-400">
                      ClusterIP: <span className="text-white">{getKeyField(resource, 'cluster_ip')}</span>
                    </span>
                  )}
                  {getKeyField(resource, 'ports') && (
                    <span className="text-slate-400">
                      Ports: <span className="text-white">{getKeyField(resource, 'ports')}</span>
                    </span>
                  )}
                </div>
              )}

              {type === 'routes' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {getKeyField(resource, 'host') && (
                    <span className="text-slate-400">
                      Host: <span className="text-white">{getKeyField(resource, 'host')}</span>
                    </span>
                  )}
                  {getKeyField(resource, 'service_name') && (
                    <span className="text-slate-400">
                      Service: <span className="text-white">{getKeyField(resource, 'service_name')}</span>
                    </span>
                  )}
                  {getKeyField(resource, 'tls_termination') && (
                    <span className="text-slate-400">
                      TLS: <span className="text-white">{getKeyField(resource, 'tls_termination')}</span>
                    </span>
                  )}
                </div>
              )}

              {type === 'networkpolicies' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {getKeyField(resource, 'policy_types') && (
                    <span className="text-slate-400">
                      Types: <span className="text-white">{getKeyField(resource, 'policy_types')}</span>
                    </span>
                  )}
                  <span className="text-slate-400">
                    Ingress: <span className="text-white">{getNumberField(resource, 'ingress_rules') || 0}</span>
                  </span>
                  <span className="text-slate-400">
                    Egress: <span className="text-white">{getNumberField(resource, 'egress_rules') || 0}</span>
                  </span>
                </div>
              )}

              {type === 'ingresscontrollers' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  {getKeyField(resource, 'domain') && (
                    <span className="text-slate-400">
                      Domain: <span className="text-white">{getKeyField(resource, 'domain')}</span>
                    </span>
                  )}
                  {getKeyField(resource, 'available') && (
                    <span className="text-slate-400">
                      Available: <span className="text-white">{getKeyField(resource, 'available')}</span>
                    </span>
                  )}
                  {getKeyField(resource, 'degraded') && (
                    <span className="text-slate-400">
                      Degraded: <span className="text-white">{getKeyField(resource, 'degraded')}</span>
                    </span>
                  )}
                </div>
              )}

              {type === 'daemonsets' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Desired: <span className="text-white">{getNumberField(resource, 'desired_number_scheduled') || 0}</span>
                  </span>
                  <span className="text-slate-400">
                    Current: <span className="text-white">{getNumberField(resource, 'current_number_scheduled') || 0}</span>
                  </span>
                  <span className="text-slate-400">
                    Ready: <span className="text-white">{getNumberField(resource, 'number_ready') || 0}</span>
                  </span>
                </div>
              )}

              {type === 'jobs' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Completions: <span className="text-white">{getNumberField(resource, 'succeeded') || 0}/{getNumberField(resource, 'completions') || 0}</span>
                  </span>
                  {(getNumberField(resource, 'failed') || 0) > 0 && (
                    <span className="text-red-400">
                      Failed: {getNumberField(resource, 'failed')}
                    </span>
                  )}
                </div>
              )}

              {type === 'cronjobs' && (
                <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Schedule: <span className="text-white">{getKeyField(resource, 'schedule') || 'N/A'}</span>
                  </span>
                  {getKeyField(resource, 'suspend') === 'true' && (
                    <span className="text-amber-400">Suspended</span>
                  )}
                </div>
              )}

              {type === 'pods' && (
                <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Phase: <span className="text-white">{getPodPhaseLabel(resource)}</span>
                  </span>
                  {(getNumberField(resource, 'restart_count') || 0) > 0 && (
                    <span className="text-amber-400">
                      Restarts: {getNumberField(resource, 'restart_count')}
                    </span>
                  )}
                </div>
              )}

              {type === 'configmaps' && (
                <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Data Keys: <span className="text-white">{getNumberField(resource, 'data_count') || 0}</span>
                  </span>
                  {(getNumberField(resource, 'binary_data_count') || 0) > 0 && (
                    <span className="text-slate-400">
                      Binary Keys: <span className="text-white">{getNumberField(resource, 'binary_data_count')}</span>
                    </span>
                  )}
                  {getKeyField(resource, 'immutable') === 'true' && (
                    <span className="text-emerald-400">Immutable</span>
                  )}
                </div>
              )}

              {type === 'secrets' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Type: <span className="text-white">{getKeyField(resource, 'type') || 'Opaque'}</span>
                  </span>
                  <span className="text-slate-400">
                    Data Keys: <span className="text-white">{getNumberField(resource, 'data_count') || 0}</span>
                  </span>
                  {getKeyField(resource, 'immutable') === 'true' && (
                    <span className="text-emerald-400">Immutable</span>
                  )}
                </div>
              )}

              {type === 'operators' && (
                <div className="flex flex-wrap gap-x-3 gap-y-1 text-sm">
                  <span className="text-slate-400">
                    Scope: <span className="text-white">Cluster</span>
                  </span>
                  {getKeyField(resource, 'available') && (
                    <span className="text-slate-400">
                      Available: <span className="text-white">{getKeyField(resource, 'available')}</span>
                    </span>
                  )}
                  {getKeyField(resource, 'degraded') && (
                    <span className="text-slate-400">
                      Degraded: <span className="text-white">{getKeyField(resource, 'degraded')}</span>
                    </span>
                  )}
                </div>
              )}
              </div>

              {/* Errors and warnings */}
              {resource.errors && resource.errors.length > 0 && (
                <div className="mt-1.5">
                  {resource.errors.map((error, idx) => (
                    <div key={idx} className="text-sm text-red-400">⚠️ {error}</div>
                  ))}
                </div>
              )}

              {resource.warnings && resource.warnings.length > 0 && (
                <div className="mt-1.5">
                  {resource.warnings.map((warning, idx) => (
                    <div key={idx} className="text-sm text-amber-400">⚡ {warning}</div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </Card>
        )
      })}
    </div>
  )
}

export function Workloads({ data, defaultTab = 'topology', visibleTabs = null, title = 'Workloads', onShowHelp = null, theme = 'dark', initialNamespace = 'all', showTabs = true }) {
  const workloads = data?.core?.workloads || {}
  const networking = data?.infrastructure?.networking || {}
  const operatorItems = data?.overview?.cluster_health?.operators || []
  const relationshipContext = useMemo(
    () => ({
      ...workloads,
      routes: networking.routes || { items: [] },
      services: networking.services || { items: [] },
      endpoints: networking.endpoints || { items: [] },
      networkpolicies: networking.network_policies || { items: [] },
      ingresscontrollers: networking.ingress_controllers || { items: [] },
      operators: { items: operatorItems },
    }),
    [workloads, networking, operatorItems]
  )
  const [activeTab, setActiveTab] = useState(defaultTab)
  const [selectedResource, setSelectedResource] = useState(null)
  const [selectedNamespace, setSelectedNamespace] = useState(initialNamespace)
  const [resourceSearch, setResourceSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [chromeCollapsed, setChromeCollapsed] = useState(false)
  const showNamespaceControls = activeTab !== 'operators'
  const handlePaneScroll = React.useCallback((event) => {
    const shouldCollapse = event.currentTarget.scrollTop > 24
    setChromeCollapsed((previous) => (previous === shouldCollapse ? previous : shouldCollapse))
  }, [])

  const desktopGridClass = !showNamespaceControls
    ? 'lg:grid-cols-1 xl:grid-cols-[24rem_minmax(0,1fr)]'
    : 'lg:grid-cols-1 xl:grid-cols-[24rem_minmax(0,1.35fr)]'
  const isTopologyView = activeTab === 'topology'

  // Extract unique namespaces from all workload resources
  const allNamespaces = useMemo(() => {
    const nsSet = new Set()
    Object.values(relationshipContext).forEach(workloadType => {
      if (workloadType?.items) {
        workloadType.items.forEach(item => {
          if (item.namespace) {
            nsSet.add(item.namespace)
          }
        })
      }
    })
    return Array.from(nsSet).sort()
  }, [relationshipContext])

  // Helper function to filter resources by namespace and count issues
  const getFilteredCount = useMemo(() => {
    return (workloadType) => {
      if (workloadType === 'operators') {
        return operatorItems.length
      }
      const items = relationshipContext[workloadType]?.items || []
      if (selectedNamespace === 'all') {
        return items.length
      }
      return items.filter(r => r.namespace === selectedNamespace).length
    }
  }, [relationshipContext, operatorItems, selectedNamespace])

  const getFilteredIssues = useMemo(() => {
    return (workloadType, issueChecker) => {
      if (workloadType === 'operators') {
        return operatorItems.filter(issueChecker).length
      }
      const items = relationshipContext[workloadType]?.items || []
      const filtered = selectedNamespace === 'all'
        ? items
        : items.filter(r => r.namespace === selectedNamespace)
      return filtered.filter(issueChecker).length
    }
  }, [relationshipContext, operatorItems, selectedNamespace])

  const topologyResources = useMemo(() => {
    const resources = getTopologyResources(workloads, selectedNamespace)
    const searched = applySearchFilter(resources, resourceSearch)
    return applyStatusFilter(searched, statusFilter)
  }, [workloads, selectedNamespace, resourceSearch, statusFilter])

  const isSameResource = (a, b) =>
    !!a &&
    !!b &&
    a.name === b.name &&
    a.namespace === b.namespace &&
    a.kind === b.kind

  // Filter resources by namespace, search term, and status
  const filteredResources = useMemo(() => {
    if (activeTab === 'topology') {
      return []
    }

    let resources = activeTab === 'operators'
      ? [...operatorItems]
      : relationshipContext[activeTab]?.items || []

    // Filter by namespace
    if (activeTab !== 'operators' && selectedNamespace !== 'all') {
      resources = resources.filter(r => r.namespace === selectedNamespace)
    }

    resources = applySearchFilter(resources, resourceSearch)
    resources = applyStatusFilter(resources, statusFilter)

    // Sort: errors first, then warnings, then healthy
    return resources.sort((a, b) => {
      const aHasErrors = a.errors && a.errors.length > 0
      const bHasErrors = b.errors && b.errors.length > 0
      const aHasWarnings = a.warnings && a.warnings.length > 0
      const bHasWarnings = b.warnings && b.warnings.length > 0

      if (aHasErrors && !bHasErrors) return -1
      if (!aHasErrors && bHasErrors) return 1
      if (aHasWarnings && !bHasWarnings) return -1
      if (!aHasWarnings && bHasWarnings) return 1
      return 0
    })
  }, [relationshipContext, operatorItems, activeTab, selectedNamespace, resourceSearch, statusFilter])

  const currentSelectedResource = useMemo(() => {
    if (isTopologyView) {
      if (selectedResource && topologyResources.some((resource) => isSameResource(resource, selectedResource))) {
        return selectedResource
      }
      return topologyResources[0] || null
    }

    if (selectedResource && filteredResources.some((resource) => isSameResource(resource, selectedResource))) {
      return selectedResource
    }

    return null
  }, [filteredResources, isTopologyView, selectedResource, topologyResources])

  const selectedTopologyResource = isTopologyView ? currentSelectedResource : null

  React.useEffect(() => {
    if (selectedResource && !currentSelectedResource) {
      setSelectedResource(null)
    }
  }, [currentSelectedResource, selectedResource])

  React.useEffect(() => {
    setSelectedNamespace(initialNamespace)
    setSelectedResource(null)
  }, [initialNamespace])

  React.useEffect(() => {
    setActiveTab(defaultTab)
    setSelectedResource(null)
    setResourceSearch('')
    setStatusFilter('all')
    setChromeCollapsed(false)
  }, [defaultTab])

  const tabs = useMemo(() => [
    {
      id: 'topology',
      label: 'Topology',
      count: getTopologyResources(workloads, selectedNamespace).length,
      issues: getTopologyResources(relationshipContext, selectedNamespace).filter((r) => {
        return r.status === 'Error' || r.errors?.length > 0
      }).length
    },
    {
      id: 'operators',
      label: 'Operators',
      count: getFilteredCount('operators'),
      issues: getFilteredIssues('operators', (o) => {
        return o.status === 'Error' || o.errors?.length > 0
      })
    },
    {
      id: 'pods',
      label: 'Pods',
      count: getFilteredCount('pods'),
      issues: getFilteredIssues('pods', (p) => {
        return p.status === 'Error' || p.errors?.length > 0
      })
    },
    {
      id: 'deployments',
      label: 'Deployments',
      count: getFilteredCount('deployments'),
      issues: getFilteredIssues('deployments', (d) => {
        return d.status === 'Error' || d.errors?.length > 0
      })
    },
    {
      id: 'statefulsets',
      label: 'StatefulSets',
      count: getFilteredCount('statefulsets'),
      issues: getFilteredIssues('statefulsets', (s) => {
        return s.status === 'Error' || s.errors?.length > 0
      })
    },
    {
      id: 'services',
      label: 'Services',
      count: getFilteredCount('services'),
      issues: getFilteredIssues('services', (s) => {
        return s.status === 'Error' || s.errors?.length > 0
      })
    },
    {
      id: 'endpoints',
      label: 'Endpoints',
      count: getFilteredCount('endpoints'),
      issues: getFilteredIssues('endpoints', (e) => {
        return e.status === 'Error' || e.errors?.length > 0 || e.warnings?.length > 0
      })
    },
    {
      id: 'networkpolicies',
      label: 'NetworkPolicies',
      count: getFilteredCount('networkpolicies'),
      issues: getFilteredIssues('networkpolicies', (policy) => {
        return policy.status === 'Error' || policy.errors?.length > 0 || policy.warnings?.length > 0
      })
    },
    {
      id: 'routes',
      label: 'Routes',
      count: getFilteredCount('routes'),
      issues: getFilteredIssues('routes', (r) => {
        return r.status === 'Error' || r.errors?.length > 0
      })
    },
    {
      id: 'ingresscontrollers',
      label: 'IngressControllers',
      count: getFilteredCount('ingresscontrollers'),
      issues: getFilteredIssues('ingresscontrollers', (i) => {
        return i.status === 'Error' || i.errors?.length > 0 || i.warnings?.length > 0
      })
    },
    {
      id: 'configmaps',
      label: 'ConfigMaps',
      count: getFilteredCount('configmaps'),
      issues: 0,
    },
    {
      id: 'secrets',
      label: 'Secrets',
      count: getFilteredCount('secrets'),
      issues: 0,
    },
    {
      id: 'daemonsets',
      label: 'DaemonSets',
      count: getFilteredCount('daemonsets'),
      issues: getFilteredIssues('daemonsets', (d) => {
        return d.status === 'Error' || d.errors?.length > 0
      })
    },
    {
      id: 'jobs',
      label: 'Jobs',
      count: getFilteredCount('jobs'),
      issues: getFilteredIssues('jobs', (j) => {
        return j.status === 'Error' || j.errors?.length > 0
      })
    },
    {
      id: 'cronjobs',
      label: 'CronJobs',
      count: getFilteredCount('cronjobs'),
      issues: getFilteredIssues('cronjobs', (c) => {
        return c.status === 'Error' || c.errors?.length > 0
      })
    },
    {
      id: 'replicasets',
      label: 'ReplicaSets',
      count: getFilteredCount('replicasets'),
      issues: getFilteredIssues('replicasets', (r) => {
        return r.status === 'Error' || r.errors?.length > 0
      })
    },
  ], [getFilteredCount, getFilteredIssues, workloads, relationshipContext, selectedNamespace])

  const activeTabData = tabs.find(t => t.id === activeTab)
  const renderedTabs = visibleTabs
    ? tabs.filter((tab) => visibleTabs.includes(tab.id))
    : tabs

  // Handle tab change - clear selection and filters when switching tabs
  const handleTabChange = (newTab) => {
    setActiveTab(newTab)
    setSelectedResource(null)
    setResourceSearch('')
    setStatusFilter('all')
  }

  // Handle resource selection
  const handleResourceClick = (resource) => {
    setSelectedResource(resource)
  }

  const handleOpenRelatedResource = (resource) => {
    const nextTab = resourceKindToTab(resource.kind)
    if (nextTab && nextTab !== activeTab) {
      setActiveTab(nextTab)
    }
    setSelectedResource(resource)
  }

  // Handle namespace change
  const handleNamespaceChange = (namespace) => {
    setSelectedNamespace(namespace)
    setSelectedResource(null)
  }

  return (
    <div className="flex flex-col gap-4 xl:h-full xl:min-h-0">
      <div className="xl:flex-shrink-0">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800 pb-4">
          <div className="flex flex-wrap items-center gap-4">
            {showNamespaceControls && (
              <ProjectSelector
                namespaces={allNamespaces}
                selectedNamespace={selectedNamespace}
                onNamespaceChange={handleNamespaceChange}
              />
            )}
            <div className="text-sm text-slate-400">
              {(activeTab === 'topology' ? topologyResources.length : filteredResources.length)} of {activeTabData?.count || 0} {activeTabData?.label || 'resources'}
              {showNamespaceControls && selectedNamespace !== 'all' && (
                <span className="ml-2 text-slate-300">
                  in <span className="font-medium">{selectedNamespace}</span>
                </span>
              )}
              {activeTabData?.issues > 0 && (
                <span className="ml-2 text-red-400">
                  ({activeTabData.issues} with issues)
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

      <div
        className={`overflow-hidden transition-all duration-200 xl:flex-shrink-0 ${
          chromeCollapsed ? 'xl:max-h-0 xl:opacity-0 xl:-translate-y-2' : 'max-h-40 opacity-100 translate-y-0'
        }`}
      >
        <div className="pt-2">
          <h1 className="text-3xl font-bold text-white">{title}</h1>
        </div>
      </div>

      {showTabs && (
        <div className="xl:flex-shrink-0">
          <WorkloadTabs tabs={renderedTabs} activeTab={activeTab} onTabChange={handleTabChange} theme={theme} />
        </div>
      )}

      {isTopologyView ? (
        <div className={`grid grid-cols-1 gap-6 xl:min-h-0 xl:flex-1 xl:overflow-hidden ${desktopGridClass}`}>
          <div className="pane-scrollbar min-h-0 xl:h-full xl:overflow-y-scroll xl:pr-2" onScroll={handlePaneScroll}>
            <ResourceFilterBar
              searchTerm={resourceSearch}
              onSearchChange={setResourceSearch}
              statusFilter={statusFilter}
              onStatusFilterChange={setStatusFilter}
              resourceCount={topologyResources.length}
            />

            <TopologyResourceList
              resources={topologyResources}
              selectedResource={selectedTopologyResource}
              onResourceClick={handleResourceClick}
              theme={theme}
            />
          </div>

          <div className="pane-scrollbar min-h-0 space-y-4 xl:h-full xl:overflow-y-scroll xl:pr-2" onScroll={handlePaneScroll}>
            <TopologyBoard
              resources={selectedTopologyResource ? [selectedTopologyResource] : []}
              workloads={relationshipContext}
              onOpenResource={handleOpenRelatedResource}
              theme={theme}
            />

            {selectedTopologyResource && (
              <ResourceDetailsPanel
                resource={selectedTopologyResource}
                workloads={relationshipContext}
                onOpenResource={handleOpenRelatedResource}
              />
            )}
          </div>
        </div>
      ) : (
        <div className={`grid grid-cols-1 gap-6 xl:min-h-0 xl:flex-1 xl:overflow-hidden ${desktopGridClass}`}>
          <div className="pane-scrollbar min-h-0 xl:h-full xl:overflow-y-scroll xl:pr-2" onScroll={handlePaneScroll}>
            <ResourceFilterBar
              searchTerm={resourceSearch}
              onSearchChange={setResourceSearch}
              statusFilter={statusFilter}
              onStatusFilterChange={setStatusFilter}
              resourceCount={filteredResources.length}
            />
            <WorkloadResourceList
              resources={filteredResources}
              type={activeTab}
              selectedResource={currentSelectedResource}
              onResourceClick={handleResourceClick}
              theme={theme}
            />
          </div>

          <div className="pane-scrollbar hidden min-h-0 xl:block xl:h-full xl:overflow-y-scroll xl:pr-2" onScroll={handlePaneScroll}>
            <ResourceDetailsPanel
              resource={currentSelectedResource}
              workloads={relationshipContext}
              onOpenResource={handleOpenRelatedResource}
            />
          </div>
        </div>
      )}

      {/* Mobile/tablet detail panel (shown on lg and below when resource selected) */}
      {currentSelectedResource && !isTopologyView && (
        <div className="xl:hidden">
          <ResourceDetailsPanel
            resource={currentSelectedResource}
            workloads={relationshipContext}
            onOpenResource={handleOpenRelatedResource}
          />
        </div>
      )}
    </div>
  )
}
