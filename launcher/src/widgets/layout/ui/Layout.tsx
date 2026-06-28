import React from 'react'
import { TitleBar } from '../../titlebar/ui/TitleBar'

interface LayoutProps {
  children: React.ReactNode
  onSettings?: () => void
}

export function Layout({ children, onSettings }: LayoutProps) {
  return (
    <div className="h-screen w-screen flex flex-col relative">
      <TitleBar onSettings={onSettings} />

      {/* Content area: pushed below titlebar, sits above glow spot */}
      <div className="flex-1 flex flex-col mt-9 relative z-10 overflow-hidden">
        {children}
      </div>

      {/* Ambient accent glow */}
      <div className="bg-glow-spot" />
    </div>
  )
}
