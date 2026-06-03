import { useState, useMemo } from 'react'

export function useFilters(resources) {
  const [filters, setFilters] = useState({
    status: 'all', // 'all', 'healthy', 'warning', 'error'
    hasErrors: false,
    hasWarnings: false,
  })

  const filteredResources = useMemo(() => {
    return resources.filter(resource => {
      const errors = resource.errors || []
      const warnings = resource.warnings || []

      // Filter by status
      if (filters.status !== 'all') {
        const resourceStatus = errors.length > 0 ? 'error' :
                              warnings.length > 0 ? 'warning' :
                              'healthy'
        if (resourceStatus !== filters.status) {
          return false
        }
      }

      // Filter by errors
      if (filters.hasErrors && errors.length === 0) {
        return false
      }

      // Filter by warnings
      if (filters.hasWarnings && warnings.length === 0) {
        return false
      }

      return true
    })
  }, [resources, filters])

  return { filters, setFilters, filteredResources }
}
