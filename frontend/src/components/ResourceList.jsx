import React from 'react'
import { ResourceCard } from './ResourceCard'

export function ResourceList({ resources, onResourceClick, selectedIndex = 0 }) {
  return (
    <div className="space-y-1.5">
      {resources.map((resource, index) => (
        <div
          key={index}
          className={`transition-all ${
            index === selectedIndex
              ? 'resource-selected-ring ring-2 ring-red-500 ring-offset-1 ring-offset-slate-950'
              : ''
          }`}
        >
          <ResourceCard
            resource={resource}
            onClick={() => onResourceClick(resource, index)}
            isSelected={index === selectedIndex}
          />
        </div>
      ))}
    </div>
  )
}
