import { useEffect } from 'react'
import { Loader2 } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'

interface SplashProps {
  onComplete: () => void
  onError: (msg: string) => void
}

export function SplashPage({ onComplete, onError }: SplashProps) {
  useEffect(() => {
    let isMounted = true

    async function init() {
      try {
        await invoke('initialize_fs')
        setTimeout(() => { if (isMounted) onComplete() }, 1000)
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : String(err)
        console.error('Initialization error:', err)
        if (isMounted) onError(message)
      }
    }

    init()
    return () => { isMounted = false }
  }, [onComplete, onError])

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="flex flex-col items-center gap-8">
        <h1 className="text-gradient text-5xl font-extrabold tracking-tight">Wufus Craft</h1>
        <div className="flex items-center gap-3 text-secondary font-medium">
          <Loader2 size={32} className="animate-spin text-accent" />
          <span>Starting up...</span>
        </div>
      </div>
    </div>
  )
}
