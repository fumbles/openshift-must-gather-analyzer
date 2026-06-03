import React, { useEffect, useMemo, useRef, useState } from 'react'

const FAVORITES_STORAGE_KEY = 'mga-favorite-projects'

export function ProjectSelector({
  namespaces = [],
  selectedNamespace = 'all',
  onNamespaceChange,
  label = 'Project:',
  allLabel = 'All projects',
  className = '',
}) {
  const [isOpen, setIsOpen] = useState(false)
  const [searchTerm, setSearchTerm] = useState('')
  const [favoriteNamespaces, setFavoriteNamespaces] = useState(() => {
    if (typeof window === 'undefined') return []
    try {
      const raw = window.localStorage.getItem(FAVORITES_STORAGE_KEY)
      const parsed = raw ? JSON.parse(raw) : []
      return Array.isArray(parsed) ? parsed : []
    } catch {
      return []
    }
  })
  const containerRef = useRef(null)
  const inputRef = useRef(null)

  const options = useMemo(() => {
    const baseOptions = [{ value: 'all', label: allLabel }]
    return [
      ...baseOptions,
      ...namespaces.map((namespace) => ({ value: namespace, label: namespace })),
    ]
  }, [allLabel, namespaces])

  const filteredOptions = useMemo(() => {
    if (!searchTerm) return options
    const search = searchTerm.toLowerCase()
    return options.filter((option) => option.label.toLowerCase().includes(search))
  }, [options, searchTerm])

  const favoriteOptions = useMemo(
    () =>
      filteredOptions.filter(
        (option) => option.value !== 'all' && favoriteNamespaces.includes(option.value)
      ),
    [filteredOptions, favoriteNamespaces]
  )

  const regularOptions = useMemo(
    () =>
      filteredOptions.filter(
        (option) => option.value === 'all' || !favoriteNamespaces.includes(option.value)
      ),
    [filteredOptions, favoriteNamespaces]
  )

  const selectedOption =
    options.find((option) => option.value === selectedNamespace) || options[0]

  useEffect(() => {
    if (typeof window === 'undefined') return
    window.localStorage.setItem(FAVORITES_STORAGE_KEY, JSON.stringify(favoriteNamespaces))
  }, [favoriteNamespaces])

  useEffect(() => {
    if (!isOpen) {
      setSearchTerm('')
      return
    }

    const timer = window.setTimeout(() => {
      inputRef.current?.focus()
    }, 0)

    return () => window.clearTimeout(timer)
  }, [isOpen])

  useEffect(() => {
    if (!isOpen) return undefined

    function handlePointerDown(event) {
      if (!containerRef.current?.contains(event.target)) {
        setIsOpen(false)
      }
    }

    function handleKeyDown(event) {
      if (event.key === 'Escape') {
        setIsOpen(false)
      }
    }

    document.addEventListener('mousedown', handlePointerDown)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('mousedown', handlePointerDown)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [isOpen])

  function toggleFavorite(namespace) {
    setFavoriteNamespaces((current) =>
      current.includes(namespace)
        ? current.filter((value) => value !== namespace)
        : [...current, namespace].sort((a, b) => a.localeCompare(b))
    )
  }

  function renderOption(option) {
    const isSelected = option.value === selectedNamespace
    const isFavorite = favoriteNamespaces.includes(option.value)
    const isAll = option.value === 'all'

    return (
      <div
        key={option.value}
        className={`flex w-full items-center justify-between rounded-lg text-sm transition-colors ${
          isSelected ? 'bg-blue-500/15 text-white ring-1 ring-blue-500/35' : 'text-slate-200 hover:bg-slate-800'
        }`}
      >
        <button
          type="button"
          onClick={() => {
            onNamespaceChange(option.value)
            setIsOpen(false)
          }}
          className="flex min-w-0 flex-1 items-center gap-2 px-3 py-1.5 text-left"
        >
          {isSelected ? (
            <span className="text-sm text-sky-300">✓</span>
          ) : (
            <span className="w-4" />
          )}
          <span className="truncate">{option.label}</span>
        </button>
        {!isAll && (
          <button
            type="button"
            onClick={() => toggleFavorite(option.value)}
            className={`ml-2 mr-1 rounded-md px-1.5 py-1 text-sm transition-colors ${
              isFavorite
                ? 'text-amber-300 hover:bg-amber-500/10 hover:text-amber-200'
                : 'text-slate-500 hover:bg-slate-800 hover:text-slate-300'
            }`}
            title={isFavorite ? 'Remove favorite' : 'Add favorite'}
            aria-label={isFavorite ? `Remove ${option.label} from favorites` : `Add ${option.label} to favorites`}
          >
            {isFavorite ? '★' : '☆'}
          </button>
        )}
      </div>
    )
  }

  return (
    <div ref={containerRef} className={`relative flex items-center gap-3 ${className}`.trim()}>
      <span className="text-sm font-medium text-slate-400">{label}</span>
      <button
        type="button"
        onClick={() => setIsOpen((open) => !open)}
        className="input flex min-w-[16rem] items-center justify-between gap-4 pr-3 text-left"
        aria-expanded={isOpen}
        aria-haspopup="listbox"
      >
        <span className="truncate">{selectedOption?.label || allLabel}</span>
        <span className="text-slate-500">▾</span>
      </button>

      {isOpen && (
        <div className="absolute left-0 top-full z-40 mt-2 w-[min(26rem,calc(100vw-2rem))] overflow-hidden rounded-2xl border border-slate-700 bg-slate-900 shadow-2xl">
          <div className="border-b border-slate-800 bg-slate-900/95 p-3 backdrop-blur">
            <div className="mb-2 px-1 text-[11px] font-semibold uppercase tracking-wider text-slate-500">
              Select project
            </div>
            <div className="relative">
              <div className="pointer-events-none absolute inset-y-0 left-3 flex items-center text-slate-400">
                🔍
              </div>
              <input
                ref={inputRef}
                type="text"
                value={searchTerm}
                onChange={(event) => setSearchTerm(event.target.value)}
                placeholder="Select project..."
                className="input w-full pl-10 pr-10"
              />
              {searchTerm && (
                <button
                  type="button"
                  onClick={() => setSearchTerm('')}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white"
                  title="Clear search"
                >
                  ✕
                </button>
              )}
            </div>
          </div>

          <div className="border-b border-slate-800 px-4 py-2 text-xs font-semibold uppercase tracking-wider text-slate-500">
            Projects
          </div>

          <div className="max-h-80 overflow-y-auto p-2">
            {filteredOptions.length ? (
              <div className="space-y-1.5">
                {favoriteOptions.length > 0 && (
                  <div className="space-y-1">
                    <div className="px-2 py-1 text-[11px] font-semibold uppercase tracking-wider text-slate-500">
                      Favorites
                    </div>
                    <div className="space-y-0.5">
                      {favoriteOptions.map(renderOption)}
                    </div>
                  </div>
                )}
                {regularOptions.length > 0 && (
                  <div className="space-y-1">
                    {favoriteOptions.length > 0 && (
                      <div className="px-2 pt-2 text-[11px] font-semibold uppercase tracking-wider text-slate-500">
                        All projects
                      </div>
                    )}
                    <div className="space-y-0.5">
                      {regularOptions.map(renderOption)}
                    </div>
                  </div>
                )}
              </div>
            ) : (
              <div className="px-3 py-4 text-sm text-slate-400">No matching projects</div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
