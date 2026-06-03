import React, { useMemo, useState } from 'react'
import { YAMLViewer } from './YAMLViewer'

function getLogKey(log, index) {
  return `${log.container || 'unknown'}-${index}`
}

export function ContainerLogs({
  logs = [],
  podName = null,
  namespace = null,
  viewer = 'pre',
  viewerClassName = 'max-h-80',
}) {
  const entries = useMemo(
    () => logs.map((log, index) => ({ ...log, key: getLogKey(log, index) })),
    [logs]
  )
  const [selectedKey, setSelectedKey] = useState('')
  const selectedLog = entries.find((log) => log.key === selectedKey) || entries[0]

  if (!selectedLog) {
    return null
  }

  const command =
    podName && namespace
      ? `oc logs pod/${podName} -n ${namespace} -c ${selectedLog.container}`
      : null

  return (
    <div className="space-y-3">
      <div className="rounded-xl border border-slate-800 bg-slate-900/80 px-4 py-3">
        <div className="flex flex-wrap items-center gap-3">
          <label className="text-xs font-semibold uppercase tracking-wider text-slate-500">
            Container ({entries.length})
          </label>
          {entries.length > 1 ? (
            <select
              value={selectedLog.key}
              onChange={(event) => setSelectedKey(event.target.value)}
              className="min-w-0 max-w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm text-slate-200 focus:border-red-500 focus:outline-none focus:ring-1 focus:ring-red-500"
            >
              {entries.map((log) => (
                <option key={log.key} value={log.key}>
                  {log.container}
                </option>
              ))}
            </select>
          ) : (
            <span className="text-sm font-medium text-slate-300">
              {selectedLog.container}
            </span>
          )}
        </div>
        {command && (
          <code className="mt-3 block break-all rounded bg-slate-950 px-2 py-1 text-xs text-emerald-400">
            {command}
          </code>
        )}
        {selectedLog.path && (
          <div className="mt-3 break-all font-mono text-xs text-slate-500">
            {selectedLog.path}
          </div>
        )}
      </div>
      {viewer === 'yaml' ? (
        <div className={viewerClassName}>
          <YAMLViewer
            title={`Container Logs: ${selectedLog.container}`}
            content={selectedLog.content || '# No logs available'}
          />
        </div>
      ) : (
        <pre className={`${viewerClassName} overflow-auto rounded-xl border border-slate-800 bg-slate-900 p-4 text-sm text-slate-300 whitespace-pre-wrap break-words`}>
          <code>{selectedLog.content || '# No logs available'}</code>
        </pre>
      )}
    </div>
  )
}
