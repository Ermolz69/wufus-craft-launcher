import { useEffect } from 'react'
import { Loader2 } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import '../../../shared/styles/Screens.css'

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

        setTimeout(() => {
          if (isMounted) onComplete()
        }, 1000)
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : String(err)
        console.error('Initialization error:', err)
        if (isMounted) onError(message)
      }
    }

    init()

    return () => {
      isMounted = false
    }
  }, [onComplete, onError])

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="splash-content">
        <h1 className="text-gradient splash-logo">Wufus Craft</h1>
        <div className="splash-loader">
          <Loader2 className="spinner" size={32} color="var(--accent-primary)" />
          <span>Starting up...</span>
        </div>
      </div>
    </div>
  )
}
