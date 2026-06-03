import React, { useState } from 'react'

export function Tabs({ tabs, defaultTab = 0, scrollable = false, activeTab: controlledActiveTab, onTabChange }) {
  const [uncontrolledActiveTab, setUncontrolledActiveTab] = useState(defaultTab)
  const activeTab = controlledActiveTab ?? uncontrolledActiveTab

  const handleTabChange = (index) => {
    if (controlledActiveTab === undefined) {
      setUncontrolledActiveTab(index)
    }
    onTabChange?.(index)
  }

  return (
    <div className={scrollable ? "flex h-full min-h-0 flex-col overflow-hidden" : ""}>
      {/* Tab buttons */}
      <div className="flex flex-wrap gap-2 border-b border-slate-800 flex-shrink-0">
        {tabs.map((tab, index) => (
          <button
            key={index}
            onClick={() => handleTabChange(index)}
            className={`px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === index
                ? 'border-b-2 border-red-500 text-white'
                : 'text-slate-400 hover:text-slate-300'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className={scrollable ? "pane-scrollbar mt-6 min-h-0 flex-1 overflow-y-scroll pr-1 pb-12" : "mt-6 pb-8"}>
        {tabs[activeTab]?.content}
      </div>
    </div>
  )
}
