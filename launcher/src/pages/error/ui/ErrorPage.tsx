import { AlertTriangle, FileText, RotateCcw, Wrench } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'

interface ErrorProps {
  onRetry: () => void
  onRepair?: () => void
  message?: string
  kind?: string
}

interface ErrorInfo {
  title: string
  description: string
}

const ERROR_MESSAGES: Record<string, ErrorInfo> = {
  network: {
    title: 'No internet connection',
    description: 'Check your internet connection and try again.',
  },
  disk_space: {
    title: 'Not enough disk space',
    description: 'Free up some disk space and try again.',
  },
  file_access: {
    title: 'File access denied',
    description: 'Try running the launcher as administrator.',
  },
  internal: {
    title: 'Something went wrong',
    description: 'An unexpected error occurred. Check the logs for details.',
  },
}

function getErrorInfo(kind?: string): ErrorInfo {
  if (kind && kind in ERROR_MESSAGES) return ERROR_MESSAGES[kind]
  return ERROR_MESSAGES.internal
}

export function ErrorPage({ onRetry, onRepair, kind }: ErrorProps) {
  const { title, description } = getErrorInfo(kind)

  const openLogs = () => invoke('open_logs_folder').catch(() => undefined)

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="glass-panel flex flex-col items-center text-center gap-4 p-10 max-w-[400px]">
        <AlertTriangle size={48} className="text-danger mb-2" />
        <h2>{title}</h2>
        <p className="text-secondary leading-relaxed mb-3">{description}</p>

        <div className="flex gap-2 flex-wrap justify-center">
          <button className="btn-primary" onClick={onRetry}>
            <RotateCcw size={16} />
            Try again
          </button>
          {onRepair && (
            <button className="btn-secondary" onClick={onRepair}>
              <Wrench size={16} />
              Repair
            </button>
          )}
          <button className="btn-secondary" onClick={openLogs}>
            <FileText size={16} />
            Open logs
          </button>
        </div>
      </div>
    </div>
  )
}
