import type { ReactNode } from "react"
import { MainContent } from "./MainContent"
import { Sidebar } from "./Sidebar"

interface AppLayoutProps {
  children?: ReactNode
  onOpenToolbox?: () => void
  onNavigateToMain?: () => void
}

export function AppLayout({ children, onOpenToolbox, onNavigateToMain }: AppLayoutProps) {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background pb-6">
      <Sidebar onOpenToolbox={onOpenToolbox} onNavigateToMain={onNavigateToMain} />
      {children || <MainContent />}
    </div>
  )
}
