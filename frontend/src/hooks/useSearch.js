import { useMemo } from 'react'

export function useSearch(resources, searchQuery) {
  return useMemo(() => {
    if (!searchQuery || searchQuery.trim() === '') {
      return resources
    }

    const query = searchQuery.toLowerCase()

    return resources.filter(resource => {
      // Search in name
      if (resource.name.toLowerCase().includes(query)) {
        return true
      }

      // Search in kind
      if (resource.kind.toLowerCase().includes(query)) {
        return true
      }

      // Search in namespace
      if (resource.namespace && resource.namespace.toLowerCase().includes(query)) {
        return true
      }

      // Search in errors
      if (resource.errors.some(err => err.toLowerCase().includes(query))) {
        return true
      }

      // Search in warnings
      if (resource.warnings.some(warn => warn.toLowerCase().includes(query))) {
        return true
      }

      // Search in raw YAML (for advanced searches)
      if (resource.raw.toLowerCase().includes(query)) {
        return true
      }

      return false
    })
  }, [resources, searchQuery])
}
