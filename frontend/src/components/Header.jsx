import React, { useEffect } from 'react'

function OpenShiftMark() {
  return (
    <div className="h-9 w-9 shrink-0">
      <svg viewBox="0 0 64 64" className="h-full w-full" aria-hidden="true">
        <circle cx="32" cy="32" r="23" fill="none" stroke="#e11d2e" strokeWidth="11" strokeDasharray="108 42" strokeLinecap="butt" transform="rotate(-18 32 32)" />
        <rect x="41" y="12" width="18" height="5.5" rx="2.75" fill="#b91c3b" transform="rotate(-21 50 14.75)" />
        <rect x="7" y="35" width="18" height="5.5" rx="2.75" fill="#b91c3b" transform="rotate(-21 16 37.75)" />
        <rect x="13" y="44" width="16" height="5" rx="2.5" fill="#b91c3b" transform="rotate(-24 21 46.5)" />
      </svg>
    </div>
  )
}

export function Header({ title = "Must-Gather Explorer", searchQuery, onSearchChange, searchInputRef, theme = 'dark', onToggleTheme }) {
  // Focus search on "/" key press (handled by useKeyboardShortcuts now)
  // But we still handle Escape here for clearing
  useEffect(() => {
    const handleKeyPress = (e) => {
      // Clear search on Escape when search is focused
      if (e.key === 'Escape' && document.activeElement === searchInputRef?.current) {
        onSearchChange('')
        searchInputRef.current?.blur()
      }
    }

    window.addEventListener('keydown', handleKeyPress)
    return () => window.removeEventListener('keydown', handleKeyPress)
  }, [onSearchChange, searchInputRef])

  return (
    <header className={`sticky top-0 z-40 backdrop-blur-xl ${
      theme === 'light'
        ? 'border-b border-slate-200 bg-white/95'
        : 'border-b border-slate-800/80 bg-slate-950/80'
    }`}>
      <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
        <div className="flex items-center gap-3">
          <OpenShiftMark />
          <div className="leading-none">
            <div className={`text-xl font-semibold tracking-tight ${theme === 'light' ? 'text-slate-950' : 'text-white'}`}>{title}</div>
            <div className={`mt-1 text-[0.72rem] font-medium uppercase tracking-[0.16em] ${theme === 'light' ? 'text-slate-500' : 'text-slate-400'}`}>OpenShift Analysis</div>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <a
            href="https://github.com/fumbles/openshift-must-gather-analyzer/issues/new/choose"
            target="_blank"
            rel="noreferrer"
            className={`rounded-lg border px-3 py-2 text-sm transition-colors ${
              theme === 'light'
                ? 'border-slate-300 bg-white text-slate-700 hover:bg-slate-100'
                : 'border-slate-700 bg-slate-900 text-slate-300 hover:bg-slate-800'
            }`}
            title="Report a bug"
          >
            Report a bug
          </a>
          {onToggleTheme && (
            <button
              onClick={onToggleTheme}
              className={`rounded-lg border px-3 py-2 text-sm transition-colors ${
                theme === 'light'
                  ? 'border-slate-300 bg-slate-100 text-slate-700 hover:bg-slate-200'
                  : 'border-slate-700 bg-slate-800 text-slate-300 hover:bg-slate-700'
              }`}
              title={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
            >
              {theme === 'light' ? '🌙 Dark' : '☀️ Light'}
            </button>
          )}
          <div className="relative w-96">
          <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
            <svg className={`h-5 w-5 ${theme === 'light' ? 'text-slate-500' : 'text-slate-400'}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search resources... (Press / to focus)"
            className="input w-full pl-10"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
          />
            {searchQuery && (
              <button
                onClick={() => onSearchChange('')}
                className={`absolute inset-y-0 right-0 flex items-center pr-3 ${theme === 'light' ? 'text-slate-500 hover:text-slate-700' : 'text-slate-400 hover:text-slate-300'}`}
              >
                <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            )}
          </div>
        </div>
      </div>
    </header>
  )
}
