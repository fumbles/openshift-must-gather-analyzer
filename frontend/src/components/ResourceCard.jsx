import React from 'react'
import { StatusBadge } from './StatusBadge'

export function ResourceCard({ resource, onClick, isSelected = false }) {
  const errors = resource.errors || []
  const warnings = resource.warnings || []
  const status = errors.length > 0 ? 'error' :
                 warnings.length > 0 ? 'warning' :
                 'healthy'
  const displayName = resource.name === 'cluster'
    ? `${resource.kind}: cluster`
    : resource.name
  const showKind = resource.kind !== 'CustomResourceDefinition' && resource.name !== 'cluster'

  return (
    <div
      onClick={onClick}
      className={`cursor-pointer rounded-xl border bg-slate-900 px-3 py-2 transition-all hover:border-slate-700 ${
        isSelected
          ? 'resource-selected border-red-500 bg-slate-800/50'
          : 'border-slate-800'
      }`}
    >
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
            <h3 className="min-w-0 break-words text-sm font-semibold leading-snug text-white" title={displayName}>{displayName}</h3>
            <StatusBadge status={status} size="sm" />
            {showKind && (
              <span className="rounded-md bg-slate-800 px-2 py-0.5 text-[11px] leading-4 text-slate-400">{resource.kind}</span>
            )}
            {resource.namespace && (
              <span className="text-xs text-slate-400">Namespace: {resource.namespace}</span>
            )}
      </div>

      {errors.length > 0 && (
        <div className="mt-2 rounded-lg bg-red-500/10 p-2.5">
          <p className="text-sm font-semibold text-red-400">
            {errors.length} Error{errors.length !== 1 ? 's' : ''}
          </p>
          <ul className="mt-1.5 space-y-1">
            {errors.slice(0, 2).map((error, idx) => (
              <li key={idx} className="text-sm text-red-300">{error}</li>
            ))}
            {errors.length > 2 && (
              <li className="text-sm text-red-400">
                +{errors.length - 2} more
              </li>
            )}
          </ul>
        </div>
      )}

      {warnings.length > 0 && errors.length === 0 && (
        <div className="mt-2 rounded-lg bg-amber-500/10 p-2.5">
          <p className="text-sm font-semibold text-amber-400">
            {warnings.length} Warning{warnings.length !== 1 ? 's' : ''}
          </p>
          <ul className="mt-1.5 space-y-1">
            {warnings.slice(0, 2).map((warning, idx) => (
              <li key={idx} className="text-sm text-amber-300">{warning}</li>
            ))}
            {warnings.length > 2 && (
              <li className="text-sm text-amber-400">
                +{warnings.length - 2} more
              </li>
            )}
          </ul>
        </div>
      )}
    </div>
  )
}
