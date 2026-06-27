import React from 'react'
import { TitleBar } from '../../titlebar/ui/TitleBar'
import './Layout.css'

interface LayoutProps {
  children: React.ReactNode
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="layout">
      <TitleBar />
      <div className="layout-content">{children}</div>

      {/* Dynamic Background Effect */}
      <div className="bg-gradient-spot" />
    </div>
  )
}
