import { useEffect, useCallback } from 'react'

/**
 * Keyboard shortcuts configuration
 */
const SHORTCUTS = {
  // Navigation
  NEXT_RESOURCE: ['j', 'ArrowDown'],
  PREV_RESOURCE: ['k', 'ArrowUp'],
  FIRST_RESOURCE: ['g', 'g'], // Press 'g' twice
  LAST_RESOURCE: ['G'],

  // Sections (g + key)
  GO_TO_NODES: ['g', 'n'],
  GO_TO_PODS: ['g', 'p'],
  GO_TO_MACHINES: ['g', 'm'],
  GO_TO_OPERATORS: ['g', 'o'],

  // Actions
  SEARCH: ['/'],
  HELP: ['?'],
  CLOSE: ['Escape'],
  COPY: ['c'],

  // Filters
  FILTER_ERRORS: ['e'],
  FILTER_WARNINGS: ['w'],
  FILTER_HEALTHY: ['h'],
  CLEAR_FILTERS: ['x'],
}

/**
 * Hook for managing keyboard shortcuts
 */
export function useKeyboardShortcuts({
  onNextResource,
  onPrevResource,
  onFirstResource,
  onLastResource,
  onGoToSection,
  onSearch,
  onHelp,
  onClose,
  onCopy,
  onFilterErrors,
  onFilterWarnings,
  onFilterHealthy,
  onClearFilters,
  enabled = true,
}) {
  const handleKeyPress = useCallback((e) => {
    // Don't trigger shortcuts when typing in inputs
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
      // Allow Escape to blur inputs
      if (e.key === 'Escape') {
        e.target.blur()
        onClose?.()
      }
      return
    }

    // Prevent default for our shortcuts
    const key = e.key

    // Single key shortcuts
    switch (key) {
      case 'j':
      case 'ArrowDown':
        e.preventDefault()
        onNextResource?.()
        break
      case 'k':
      case 'ArrowUp':
        e.preventDefault()
        onPrevResource?.()
        break
      case 'G':
        e.preventDefault()
        onLastResource?.()
        break
      case '/':
        e.preventDefault()
        onSearch?.()
        break
      case '?':
        e.preventDefault()
        onHelp?.()
        break
      case 'Escape':
        e.preventDefault()
        onClose?.()
        break
      case 'c':
        if (!e.ctrlKey && !e.metaKey) {
          e.preventDefault()
          onCopy?.()
        }
        break
      case 'e':
        e.preventDefault()
        onFilterErrors?.()
        break
      case 'w':
        e.preventDefault()
        onFilterWarnings?.()
        break
      case 'h':
        e.preventDefault()
        onFilterHealthy?.()
        break
      case 'x':
        e.preventDefault()
        onClearFilters?.()
        break
    }

    // Handle 'g' prefix shortcuts
    if (key === 'g') {
      // Set a flag to wait for next key
      const handleSecondKey = (e2) => {
        e2.preventDefault()
        switch (e2.key) {
          case 'g':
            onFirstResource?.()
            break
          case 'n':
            onGoToSection?.('nodes')
            break
          case 'p':
            onGoToSection?.('pods')
            break
          case 'm':
            onGoToSection?.('machines')
            break
          case 'o':
            onGoToSection?.('operators')
            break
        }
        window.removeEventListener('keydown', handleSecondKey)
      }

      window.addEventListener('keydown', handleSecondKey, { once: true })

      // Clear the listener after 1 second if no second key is pressed
      setTimeout(() => {
        window.removeEventListener('keydown', handleSecondKey)
      }, 1000)
    }
  }, [
    onNextResource,
    onPrevResource,
    onFirstResource,
    onLastResource,
    onGoToSection,
    onSearch,
    onHelp,
    onClose,
    onCopy,
    onFilterErrors,
    onFilterWarnings,
    onFilterHealthy,
    onClearFilters,
  ])

  useEffect(() => {
    if (!enabled) return

    window.addEventListener('keydown', handleKeyPress)
    return () => window.removeEventListener('keydown', handleKeyPress)
  }, [enabled, handleKeyPress])

  return { shortcuts: SHORTCUTS }
}

/**
 * Get human-readable shortcut descriptions
 */
export function getShortcutDescriptions() {
  return [
    { category: 'Navigation', shortcuts: [
      { keys: ['j', '↓'], description: 'Next resource' },
      { keys: ['k', '↑'], description: 'Previous resource' },
      { keys: ['g', 'g'], description: 'First resource' },
      { keys: ['G'], description: 'Last resource' },
    ]},
    { category: 'Sections', shortcuts: [
      { keys: ['g', 'n'], description: 'Go to Nodes' },
      { keys: ['g', 'p'], description: 'Go to Pods' },
      { keys: ['g', 'm'], description: 'Go to Machines' },
      { keys: ['g', 'o'], description: 'Go to Operators' },
    ]},
    { category: 'Actions', shortcuts: [
      { keys: ['/'], description: 'Focus search' },
      { keys: ['?'], description: 'Show this help' },
      { keys: ['Esc'], description: 'Close/Cancel' },
      { keys: ['c'], description: 'Copy resource' },
    ]},
    { category: 'Filters', shortcuts: [
      { keys: ['e'], description: 'Toggle errors filter' },
      { keys: ['w'], description: 'Toggle warnings filter' },
      { keys: ['h'], description: 'Toggle healthy filter' },
      { keys: ['x'], description: 'Clear all filters' },
    ]},
  ]
}
