import React, { useState, useRef, useEffect } from 'react'

export function YAMLViewer({ content, title = 'YAML Content' }) {
  const [isCollapsed, setIsCollapsed] = useState(false)
  const [isAtBottom, setIsAtBottom] = useState(false)
  const scrollRef = useRef(null)

  const handleScroll = (e) => {
    const element = e.target
    const isBottom = Math.abs(element.scrollHeight - element.scrollTop - element.clientHeight) < 5
    setIsAtBottom(isBottom)
  }

  useEffect(() => {
    // Check if content fits without scrolling
    if (scrollRef.current) {
      const hasScroll = scrollRef.current.scrollHeight > scrollRef.current.clientHeight
      setIsAtBottom(!hasScroll)
    }
  }, [content, isCollapsed])

  return (
    <div className="rounded-2xl border border-slate-800 bg-slate-950 flex flex-col h-full">
      <div className="flex items-center justify-between border-b border-slate-800 px-4 py-2 flex-shrink-0">
        <span className="text-sm font-medium text-slate-400">{title}</span>
        <button
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="text-xs text-slate-400 hover:text-white transition-colors"
        >
          {isCollapsed ? '▼ Expand' : '▲ Collapse'}
        </button>
      </div>
      {!isCollapsed && (
        <div className="relative flex-1 flex flex-col">
          <div
            ref={scrollRef}
            onScroll={handleScroll}
            className="p-4 overflow-auto flex-1"
          >
            <pre className="text-sm text-slate-300 whitespace-pre-wrap break-words">
              <code>{content}</code>
            </pre>
          </div>
          {/* End of content indicator */}
          {isAtBottom && (
            <div className="absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-slate-950 to-transparent pointer-events-none flex items-end justify-center pb-1">
              <span className="text-xs text-slate-600">— End of YAML —</span>
            </div>
          )}
          {/* Scroll indicator when not at bottom */}
          {!isAtBottom && (
            <div className="absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-slate-950 via-slate-950/80 to-transparent pointer-events-none flex items-end justify-center pb-2">
              <span className="text-xs text-slate-500 animate-pulse">▼ Scroll for more ▼</span>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
