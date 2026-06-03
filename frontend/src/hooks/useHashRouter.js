import { useState, useEffect, useCallback } from 'react'

/**
 * Custom hook for hash-based routing
 * Format: #section/resourceId
 * Examples:
 *   #nodes
 *   #nodes/ip-10-0-0-1.control.plane
 *   #pods/cluster-autoscaler-default-f548ffc66-bck7p
 */
export function useHashRouter() {
  const [route, setRoute] = useState(() => parseHash(window.location.hash))

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(parseHash(window.location.hash))
    }

    window.addEventListener('hashchange', handleHashChange)
    return () => window.removeEventListener('hashchange', handleHashChange)
  }, [])

  const navigate = useCallback((section, resourceId = null) => {
    const hash = resourceId ? `#${section}/${resourceId}` : `#${section}`
    window.location.hash = hash
  }, [])

  const navigateBack = useCallback(() => {
    window.history.back()
  }, [])

  return {
    section: route.section,
    resourceId: route.resourceId,
    navigate,
    navigateBack,
  }
}

function parseHash(hash) {
  // Remove leading #
  const path = hash.replace(/^#/, '')

  if (!path) {
    return { section: null, resourceId: null }
  }

  const parts = path.split('/')
  return {
    section: parts[0] || null,
    resourceId: parts[1] || null,
  }
}
