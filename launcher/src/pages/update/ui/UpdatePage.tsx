import { useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

interface UpdateProps {
  mode?: 'update' | 'repair'
  onComplete: () => void
  onError: (msg: string, kind: string) => void
  onCancel: () => void
}

type UpdateStage = 'checking_files' | 'downloading' | 'finalizing'

interface ProgressSnapshot {
  total_files: number
  completed_files: number
  failed_files: number
  total_bytes: number
  downloaded_bytes: number
  bytes_per_sec: number
  remaining_bytes: number
}

type UpdaterEvent =
  | { type: 'stage'; payload: { stage: UpdateStage } }
  | { type: 'progress'; payload: ProgressSnapshot }
  | { type: 'done'; payload: Record<string, number> }
  | { type: 'error'; payload: { kind: string; message: string } }
  | { type: 'cancelled' }

const STAGE_LABELS: Record<UpdateStage, string> = {
  checking_files: 'Checking files...',
  downloading: 'Downloading updates...',
  finalizing: 'Finishing up...',
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function formatSpeed(bps: number): string {
  if (bps < 1024) return `${bps.toFixed(0)} B/s`
  if (bps < 1024 * 1024) return `${(bps / 1024).toFixed(0)} KB/s`
  return `${(bps / (1024 * 1024)).toFixed(1)} MB/s`
}

export function UpdatePage({ mode = 'update', onComplete, onError, onCancel }: UpdateProps) {
  const [stage, setStage] = useState<UpdateStage>('checking_files')
  const [progress, setProgress] = useState<ProgressSnapshot | null>(null)
  const [cancelling, setCancelling] = useState(false)

  const cbRef = useRef({ onComplete, onError, onCancel })
  useEffect(() => {
    cbRef.current = { onComplete, onError, onCancel }
  }, [onComplete, onError, onCancel])

  useEffect(() => {
    const command = mode === 'repair' ? 'start_repair' : 'start_update'
    invoke(command).catch((e: unknown) => {
      cbRef.current.onError(String(e), 'internal')
    })

    let unlistenFn: (() => void) | undefined
    listen<UpdaterEvent>('updater_event', ({ payload }) => {
      switch (payload.type) {
        case 'stage':
          setStage(payload.payload.stage)
          break
        case 'progress':
          setProgress(payload.payload)
          break
        case 'done':
          cbRef.current.onComplete()
          break
        case 'error':
          cbRef.current.onError(payload.payload.message, payload.payload.kind)
          break
        case 'cancelled':
          cbRef.current.onCancel()
          break
      }
    }).then((fn) => {
      unlistenFn = fn
    })

    return () => {
      unlistenFn?.()
    }
  }, [mode])

  const handleCancel = async () => {
    setCancelling(true)
    await invoke('cancel_update').catch(() => undefined)
  }

  const barPercent =
    progress && progress.total_bytes > 0
      ? Math.round((progress.downloaded_bytes / progress.total_bytes) * 100)
      : 0

  const title = mode === 'repair' ? 'Repairing Wufus Craft' : 'Updating Wufus Craft'

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="glass-panel flex flex-col items-center gap-5 p-8 w-[400px]">
        <h2 className="text-gradient">{title}</h2>
        <p className="text-secondary text-[0.95rem]">{STAGE_LABELS[stage]}</p>

        {/* Progress bar */}
        <div className="w-full h-2 bg-surface rounded-full overflow-hidden">
          <div
            className="h-full bg-accent transition-[width] duration-100"
            style={{
              width: `${barPercent}%`,
              boxShadow: '0 0 10px var(--accent-glow)',
            }}
          />
        </div>

        {progress && stage === 'downloading' && (
          <div className="flex justify-between w-full text-[0.8rem] text-muted">
            <span>
              {progress.completed_files}/{progress.total_files} files
            </span>
            <span>{formatSpeed(progress.bytes_per_sec)}</span>
            <span>{formatBytes(progress.remaining_bytes)} left</span>
          </div>
        )}

        <div className="mt-3">
          <button className="btn-secondary" onClick={handleCancel} disabled={cancelling}>
            {cancelling ? 'Cancelling...' : 'Cancel'}
          </button>
        </div>
      </div>
    </div>
  )
}
