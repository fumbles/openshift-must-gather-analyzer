import React from 'react'

export function Card({ title, children, className = '', onClick }) {
  return (
    <div className={`card ${className}`} onClick={onClick}>
      {title && <h3 className="text-lg font-semibold text-white mb-4">{title}</h3>}
      {children}
    </div>
  )
}
