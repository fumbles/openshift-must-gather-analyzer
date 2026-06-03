import React from 'react'
import { StatusBadge } from './StatusBadge'

export function Hero({
  status = 'healthy',
  statusLabel = null,
  title,
  subtitle,
  version,
  platform,
  stats,
  theme = 'dark',
}) {
  if (theme === 'light') {
    return (
      <section className="border-b border-slate-200 bg-slate-50/80">
        <div className="mx-auto max-w-7xl px-6 py-3">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
            <div className="min-w-0">
              <h1 className="text-2xl font-semibold tracking-tight text-slate-950">{title}</h1>
              <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-slate-600">
                <StatusBadge status={status}>{statusLabel || status}</StatusBadge>
                <span className="text-xs font-medium uppercase tracking-[0.12em] text-slate-500">Cluster overview</span>
                {subtitle && <span>{subtitle}</span>}
                {version && <span>OpenShift {version}</span>}
                {platform && <span>•</span>}
                {platform && <span>Platform: {platform}</span>}
              </div>
            </div>

            {stats && stats.length > 0 && (
              <div className="grid gap-2 sm:grid-cols-3 lg:min-w-[28rem]">
                {stats.map((stat, index) => (
                  <StatCard key={index} theme={theme} {...stat} />
                ))}
              </div>
            )}
          </div>
        </div>
      </section>
    )
  }

  return (
    <section className="relative overflow-hidden border-b border-slate-800/80">
      <div
        className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(239,68,68,0.14),transparent_28%),radial-gradient(circle_at_80%_20%,rgba(14,165,233,0.10),transparent_30%),linear-gradient(180deg,#020617_0%,#0b1220_100%)]"
      />

      <div className="relative mx-auto max-w-7xl px-6 py-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="min-w-0">
            <h1 className="text-2xl font-semibold tracking-tight text-white md:text-3xl">
              {title}
            </h1>

            <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-slate-300">
              <StatusBadge status={status}>{statusLabel || status}</StatusBadge>
              <span className="text-xs font-medium uppercase tracking-[0.12em] text-slate-400">Cluster overview</span>
              {subtitle && <span>{subtitle}</span>}
              {version && <span>OpenShift {version}</span>}
              {platform && <span>•</span>}
              {platform && <span>Platform: {platform}</span>}
            </div>
          </div>

          {stats && stats.length > 0 && (
            <div className="grid gap-2 sm:grid-cols-3 lg:min-w-[28rem]">
              {stats.map((stat, index) => (
                <StatCard key={index} theme={theme} {...stat} />
              ))}
            </div>
          )}
        </div>
      </div>
    </section>
  )
}

function StatCard({ label, value, icon, theme = 'dark' }) {
  return (
    <div className={`rounded-xl px-3 py-2 ${
      theme === 'light'
        ? 'border border-slate-200 bg-white'
        : 'border border-slate-800 bg-slate-900/60'
    }`}>
      <div className="flex items-center gap-3">
        {icon && (
          <div className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-lg ${
            theme === 'light' ? 'bg-blue-50 text-blue-600' : 'bg-red-500/10 text-red-300'
          }`}>
            {icon}
          </div>
        )}
        <div className="min-w-0">
          <div className={`text-lg font-semibold leading-none ${theme === 'light' ? 'text-slate-900' : 'text-white'}`}>{value}</div>
          <div className={`mt-1 text-xs uppercase tracking-wide ${theme === 'light' ? 'text-slate-500' : 'text-slate-400'}`}>{label}</div>
        </div>
      </div>
    </div>
  )
}
