import React from 'react'

export function Button({
  children,
  variant = 'primary',
  onClick,
  disabled = false,
  className = ''
}) {
  const baseClass = variant === 'primary' ? 'btn-primary' : 'btn-secondary'
  const disabledClass = disabled ? 'opacity-50 cursor-not-allowed' : ''

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`${baseClass} ${disabledClass} ${className}`}
    >
      {children}
    </button>
  )
}
