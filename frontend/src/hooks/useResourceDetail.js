import { useEffect, useMemo, useState } from 'react'

function loadScript(src) {
  return new Promise((resolve, reject) => {
    const existing = document.querySelector(`script[data-mga-src="${src}"]`)
    if (existing) {
      if (existing.dataset.loaded === 'true') {
        resolve()
      } else {
        existing.addEventListener('load', () => resolve(), { once: true })
        existing.addEventListener('error', () => reject(new Error(`Failed to load ${src}`)), { once: true })
      }
      return
    }

    const script = document.createElement('script')
    script.src = src
    script.async = true
    script.dataset.mgaSrc = src
    script.onload = () => {
      script.dataset.loaded = 'true'
      resolve()
    }
    script.onerror = () => reject(new Error(`Failed to load ${src}`))
    document.head.appendChild(script)
  })
}

function getDetailPath(resource) {
  if (!resource?.uid) return null
  return resource.detail_path || `data/resources/${resource.uid}.js`
}

function getRawPath(resource) {
  if (!resource?.uid) return null
  return resource.raw_path || `data/raw/${resource.uid}.js`
}

function getLogsPath(resource) {
  if (!resource?.uid || resource?.kind !== 'Pod') return null
  return resource.logs_path || `data/logs/${resource.uid}.js`
}

function hasInlineDetail(resource) {
  if (!resource) return false
  if (resource.raw) return true
  if (resource.health_analysis) return true
  if (resource.logs?.length) return true
  if (resource.metadata && Object.keys(resource.metadata).length > 0) return true
  return false
}

export function useResourceDetail(resource) {
  const [cache, setCache] = useState({})
  const [loadingPath, setLoadingPath] = useState(null)
  const [error, setError] = useState(null)
  const detailPath = getDetailPath(resource)
  const needsDetailLoad = !!resource?.uid && !hasInlineDetail(resource)

  useEffect(() => {
    if (!needsDetailLoad || !detailPath) {
      return
    }

    if (cache[resource.uid]) {
      return
    }

    let cancelled = false
    setLoadingPath(detailPath)
    setError(null)

    loadScript(detailPath)
      .then(() => {
        if (cancelled) {
          return
        }
        const details = globalThis.__MGA_RESOURCE_DETAILS__?.[resource.uid]
        if (details) {
          setCache((prev) => ({ ...prev, [resource.uid]: details }))
        } else {
          setError(`No detail data found for ${resource.name}`)
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err.message)
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingPath(null)
        }
      })

    return () => {
      cancelled = true
    }
  }, [cache, detailPath, needsDetailLoad, resource?.name, resource?.uid])

  const resolvedResource = useMemo(() => {
    if (!resource) {
      return null
    }
    if (!needsDetailLoad) {
      return resource
    }
    return cache[resource.uid] ? { ...resource, ...cache[resource.uid] } : resource
  }, [cache, needsDetailLoad, resource])

  return {
    resource: resolvedResource,
    isLoading: needsDetailLoad && loadingPath === detailPath && !cache[resource.uid],
    error,
  }
}

export function useResourceLogs(resource, enabled = true) {
  const [cache, setCache] = useState({})
  const [loadingPath, setLoadingPath] = useState(null)
  const [error, setError] = useState(null)
  const logsPath = getLogsPath(resource)

  useEffect(() => {
    if (!enabled || !logsPath || !resource?.uid) {
      return
    }

    if (cache[resource.uid] || (resource.logs && resource.logs.length > 0)) {
      return
    }

    let cancelled = false
    setLoadingPath(logsPath)
    setError(null)

    loadScript(logsPath)
      .then(() => {
        if (cancelled) {
          return
        }
        const logs = globalThis.__MGA_RESOURCE_LOGS__?.[resource.uid]
        if (logs) {
          setCache((prev) => ({ ...prev, [resource.uid]: logs }))
        } else {
          setError(`No log data found for ${resource.name}`)
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err.message)
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingPath(null)
        }
      })

    return () => {
      cancelled = true
    }
  }, [cache, enabled, logsPath, resource?.logs, resource?.name, resource?.uid])

  const logs = useMemo(() => {
    if (!resource) {
      return []
    }
    if (resource.logs && resource.logs.length > 0) {
      return resource.logs
    }
    return cache[resource.uid] || []
  }, [cache, resource])

  return {
    logs,
    isLoading: enabled && !!logsPath && loadingPath === logsPath && logs.length === 0,
    error,
  }
}

export function useResourceRaw(resource, enabled = true) {
  const [cache, setCache] = useState({})
  const [loadingPath, setLoadingPath] = useState(null)
  const [error, setError] = useState(null)
  const rawPath = getRawPath(resource)

  useEffect(() => {
    if (!enabled || !rawPath || !resource?.uid) {
      return
    }

    if (cache[resource.uid] !== undefined || resource.raw) {
      return
    }

    let cancelled = false
    setLoadingPath(rawPath)
    setError(null)

    loadScript(rawPath)
      .then(() => {
        if (cancelled) {
          return
        }
        const raw = globalThis.__MGA_RESOURCE_RAW__?.[resource.uid]
        if (raw !== undefined) {
          setCache((prev) => ({ ...prev, [resource.uid]: raw }))
        } else {
          setError(`No YAML data found for ${resource.name}`)
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err.message)
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingPath(null)
        }
      })

    return () => {
      cancelled = true
    }
  }, [cache, enabled, rawPath, resource?.name, resource?.raw, resource?.uid])

  const raw = useMemo(() => {
    if (!resource) {
      return ''
    }
    if (resource.raw) {
      return resource.raw
    }
    return cache[resource.uid] || ''
  }, [cache, resource])

  return {
    raw,
    isLoading: enabled && !!rawPath && loadingPath === rawPath && !raw,
    error,
  }
}
