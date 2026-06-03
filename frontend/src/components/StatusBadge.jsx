import React from 'react'

export function StatusBadge({ status, children, size = 'md' }) {
  const statusClasses = {
    healthy: 'badge-healthy',
    warning: 'badge-warning',
    error: 'badge-error',
    unknown: 'border-slate-700 bg-slate-800/50 text-slate-400'
  }
  const sizeClasses = {
    sm: 'px-2.5 py-1 text-[11px] gap-1.5',
    md: 'px-4 py-2 text-sm gap-2',
  }

  const statusClass = statusClasses[status?.toLowerCase()] || statusClasses.unknown
  const sizeClass = sizeClasses[size] || sizeClasses.md

  return (
    <div className={`badge ${statusClass} ${sizeClass}`}>
      <span className={`${size === 'sm' ? 'h-1.5 w-1.5' : 'h-2 w-2'} rounded-full bg-current`} />
      {children || status}
    </div>
  )
}
