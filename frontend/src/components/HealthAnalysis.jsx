import React from 'react'
import { StatusBadge } from './StatusBadge'
import { ContainerLogs } from './ContainerLogs'

export function HealthAnalysis({ analysis, logs = [], podName = null, namespace = null }) {
  if (!analysis) {
    return null
  }

  const getSeverityColor = (severity) => {
    switch (severity.toLowerCase()) {
      case 'critical':
        return 'text-red-400 bg-red-500/10 border-red-500/20'
      case 'error':
        return 'text-red-400 bg-red-500/10 border-red-500/20'
      case 'warning':
        return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
      case 'info':
        return 'text-blue-400 bg-blue-500/10 border-blue-500/20'
      default:
        return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
    }
  }

  const getCategoryIcon = (category) => {
    switch (category.toLowerCase()) {
      case 'availability':
        return '🔴'
      case 'performance':
        return '⚡'
      case 'configuration':
        return '⚙️'
      case 'capacity':
        return '📊'
      case 'security':
        return '🔒'
      case 'network':
        return '🌐'
      case 'storage':
        return '💾'
      case 'health':
        return '❤️'
      default:
        return '📋'
    }
  }

  const getHealthScoreColor = (score) => {
    if (score >= 90) return 'text-emerald-400'
    if (score >= 70) return 'text-amber-400'
    if (score >= 50) return 'text-orange-400'
    return 'text-red-400'
  }

  const getHealthScoreLabel = (score) => {
    if (score >= 90) return 'Excellent'
    if (score >= 70) return 'Good'
    if (score >= 50) return 'Fair'
    return 'Poor'
  }

  return (
    <div className="space-y-6">
      {/* Health Score */}
      <div className="rounded-2xl border border-slate-800 bg-slate-900 p-6">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-sm font-semibold uppercase tracking-wider text-slate-400">
              Health Score
            </h3>
            <p className="mt-2 text-sm text-slate-300">{analysis.summary}</p>
          </div>
          <div className="text-center">
            <div className={`text-5xl font-bold ${getHealthScoreColor(analysis.health_score)}`}>
              {analysis.health_score}
            </div>
            <div className="mt-1 text-sm text-slate-400">
              {getHealthScoreLabel(analysis.health_score)}
            </div>
          </div>
        </div>
      </div>

      {/* Issues */}
      {analysis.issues && analysis.issues.length > 0 && (
        <div>
          <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Issues ({analysis.issues.length})
          </h3>
          <div className="space-y-3">
            {analysis.issues.map((issue, idx) => (
              <div
                key={idx}
                className={`rounded-2xl border p-4 ${getSeverityColor(issue.severity)}`}
              >
                <div className="flex items-start gap-3">
                  <span className="text-2xl">{getCategoryIcon(issue.category)}</span>
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <h4 className="font-semibold">{issue.title}</h4>
                      <span className="rounded-full border border-current px-2 py-0.5 text-xs uppercase">
                        {issue.severity}
                      </span>
                    </div>
                    <p className="mt-2 text-sm opacity-90">{issue.description}</p>
                    {issue.affected_component && (
                      <p className="mt-2 text-xs opacity-75">
                        Affected: {issue.affected_component}
                      </p>
                    )}
                    {issue.detected_at && (
                      <p className="mt-1 text-xs opacity-75">
                        Detected: {issue.detected_at}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recommendations */}
      {analysis.recommendations && analysis.recommendations.length > 0 && (
        <div>
          <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Recommendations ({analysis.recommendations.length})
          </h3>
          <div className="space-y-3">
            {analysis.recommendations.map((rec, idx) => (
              <div
                key={idx}
                className="rounded-2xl border border-slate-800 bg-slate-900 p-4"
              >
                <div className="flex items-start gap-3">
                  <span className="text-2xl">💡</span>
                  <div className="flex-1">
                    <h4 className="font-semibold text-slate-200">{rec.title}</h4>
                    <p className="mt-2 text-sm text-slate-300">{rec.description}</p>
                    {rec.action && (
                      <div className="mt-3">
                        <p className="text-xs font-semibold uppercase tracking-wider text-slate-400">
                          Action
                        </p>
                        <code className="mt-1 block rounded-lg bg-slate-950 p-2 text-sm text-emerald-400">
                          {rec.action}
                        </code>
                      </div>
                    )}
                    {rec.documentation_url && (
                      <a
                        href={rec.documentation_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="mt-2 inline-flex items-center gap-1 text-sm text-red-400 hover:text-red-300"
                      >
                        📚 Documentation
                        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                      </a>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* No Issues */}
      {(!analysis.issues || analysis.issues.length === 0) &&
       (!analysis.recommendations || analysis.recommendations.length === 0) && (
        <div className="rounded-2xl border border-emerald-500/20 bg-emerald-500/10 p-6 text-center">
          <div className="text-4xl">✅</div>
          <p className="mt-2 text-emerald-400">No issues detected</p>
          <p className="mt-1 text-sm text-emerald-300/75">This resource is healthy</p>
        </div>
      )}

      {logs.length > 0 && (
        <div>
          <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Pod Logs ({logs.length})
          </h3>
          <ContainerLogs
            logs={logs}
            podName={podName}
            namespace={namespace}
          />
        </div>
      )}
    </div>
  )
}
