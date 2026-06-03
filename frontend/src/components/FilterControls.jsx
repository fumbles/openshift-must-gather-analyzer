import React from 'react'

export function FilterControls({ filters, onFilterChange, stats }) {
  return (
    <div className="space-y-4">
      <div>
        <label className="mb-2 block text-xs font-semibold uppercase tracking-wider text-slate-400">
          Status
        </label>
        <div className="space-y-1">
          <FilterButton
            active={filters.status === 'all'}
            onClick={() => onFilterChange({ ...filters, status: 'all' })}
            count={stats.total}
          >
            All
          </FilterButton>
          <FilterButton
            active={filters.status === 'healthy'}
            onClick={() => onFilterChange({ ...filters, status: 'healthy' })}
            count={stats.healthy}
            variant="healthy"
          >
            Healthy
          </FilterButton>
          <FilterButton
            active={filters.status === 'warning'}
            onClick={() => onFilterChange({ ...filters, status: 'warning' })}
            count={stats.warnings}
            variant="warning"
          >
            Warnings
          </FilterButton>
          <FilterButton
            active={filters.status === 'error'}
            onClick={() => onFilterChange({ ...filters, status: 'error' })}
            count={stats.errors}
            variant="error"
          >
            Errors
          </FilterButton>
        </div>
      </div>

      <div>
        <label className="mb-2 block text-xs font-semibold uppercase tracking-wider text-slate-400">
          Quick Filters
        </label>
        <div className="space-y-1">
          <label className="flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 text-sm text-slate-300 hover:bg-slate-800">
            <input
              type="checkbox"
              checked={filters.hasErrors}
              onChange={(e) => onFilterChange({ ...filters, hasErrors: e.target.checked })}
              className="rounded border-slate-600 bg-slate-800 text-red-500 focus:ring-red-500"
            />
            Has Errors
          </label>
          <label className="flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 text-sm text-slate-300 hover:bg-slate-800">
            <input
              type="checkbox"
              checked={filters.hasWarnings}
              onChange={(e) => onFilterChange({ ...filters, hasWarnings: e.target.checked })}
              className="rounded border-slate-600 bg-slate-800 text-amber-500 focus:ring-amber-500"
            />
            Has Warnings
          </label>
        </div>
      </div>
    </div>
  )
}

function FilterButton({ active, onClick, count, variant = 'default', children }) {
  const variantClasses = {
    default: active ? 'bg-slate-800 text-white' : 'text-slate-400 hover:bg-slate-800/50',
    healthy: active ? 'bg-emerald-500/10 text-emerald-400' : 'text-slate-400 hover:bg-slate-800/50',
    warning: active ? 'bg-amber-500/10 text-amber-400' : 'text-slate-400 hover:bg-slate-800/50',
    error: active ? 'bg-red-500/10 text-red-400' : 'text-slate-400 hover:bg-slate-800/50',
  }

  return (
    <button
      onClick={onClick}
      className={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm transition-colors ${variantClasses[variant]}`}
    >
      <span>{children}</span>
      <span className="rounded-full bg-slate-800 px-2 py-0.5 text-xs">{count}</span>
    </button>
  )
}
