import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import '../../../shared/styles/Screens.css'

interface UpdateProps {
  onComplete: () => void
  onError: (msg: string) => void
  onCancel: () => void
}

interface ProgressPayload {
  progress: number
  status: string
}

export function UpdatePage({ onComplete, onError, onCancel }: UpdateProps) {
  const [progress, setProgress] = useState(0)
  const [status, setStatus] = useState('Preparing update...')
  const [cancelling, setCancelling] = useState(false)

  useEffect(() => {
    const unlisteners = Promise.all([
      listen<ProgressPayload>('update:progress', ({ payload }) => {
        setProgress(payload.progress)
        setStatus(payload.status)
      }),
      listen<undefined>('update:complete', () => {
        onComplete()
      }),
      listen<string>('update:error', ({ payload }) => {
        onError(payload)
      }),
    ])

    return () => {
      unlisteners.then((fns) => fns.forEach((fn) => fn()))
    }
  }, [onComplete, onError])

  const handleCancel = async () => {
    setCancelling(true)
    try {
      await invoke('cancel_update')
    } catch {
      // best-effort: cancel_update may not be implemented yet
    }
    onCancel()
  }

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="update-panel glass-panel">
        <h2 className="text-gradient">Updating Wufus Craft</h2>
        <p className="update-status">
          {status}
          {progress > 0 ? ` ${progress}%` : ''}
        </p>

        <div className="progress-bar-bg">
          <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
        </div>

        <div className="update-actions">
          <button className="btn-secondary" onClick={handleCancel} disabled={cancelling}>
            {cancelling ? 'Cancelling...' : 'Cancel'}
          </button>
        </div>
      </div>
    </div>
  )
}
