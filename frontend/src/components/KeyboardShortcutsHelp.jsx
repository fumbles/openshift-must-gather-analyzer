import React from 'react'
import { getShortcutDescriptions } from '../hooks/useKeyboardShortcuts'

export function KeyboardShortcutsHelp({ isOpen, onClose }) {
  if (!isOpen) return null

  const shortcuts = getShortcutDescriptions()

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="relative mx-4 w-full max-w-2xl rounded-3xl border border-slate-800 bg-slate-900 p-8 shadow-2xl">
        {/* Close button */}
        <button
          onClick={onClose}
          className="absolute right-6 top-6 text-slate-400 hover:text-slate-300"
          aria-label="Close"
        >
          <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>

        {/* Header */}
        <div className="mb-6">
          <h2 className="text-2xl font-semibold text-white">Keyboard Shortcuts</h2>
          <p className="mt-2 text-sm text-slate-400">
            Use these shortcuts to navigate faster
          </p>
        </div>

        {/* Shortcuts grid */}
        <div className="grid gap-6 md:grid-cols-2">
          {shortcuts.map((category, idx) => (
            <div key={idx}>
              <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">
                {category.category}
              </h3>
              <div className="space-y-2">
                {category.shortcuts.map((shortcut, sidx) => (
                  <div key={sidx} className="flex items-center justify-between">
                    <span className="text-sm text-slate-300">{shortcut.description}</span>
                    <div className="flex gap-1">
                      {shortcut.keys.map((key, kidx) => (
                        <kbd
                          key={kidx}
                          className="rounded-lg border border-slate-700 bg-slate-800 px-2 py-1 text-xs font-semibold text-slate-300"
                        >
                          {key}
                        </kbd>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="mt-8 border-t border-slate-800 pt-6">
          <p className="text-center text-sm text-slate-400">
            Press <kbd className="rounded border border-slate-700 bg-slate-800 px-2 py-0.5 text-xs font-semibold text-slate-300">?</kbd> to toggle this help
          </p>
        </div>
      </div>
    </div>
  )
}
