import { AlertTriangle, RotateCcw } from 'lucide-react'
import '../../../shared/styles/Screens.css'

interface ErrorProps {
  onRetry: () => void
  message?: string
}

export function ErrorPage({ onRetry, message }: ErrorProps) {
  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="error-panel glass-panel">
        <AlertTriangle size={48} color="var(--danger)" className="error-icon" />
        <h2>Something went wrong</h2>
        <p className="error-text">
          {message ||
            'Failed to connect to the update server. Please check your internet connection and try again.'}
        </p>

        <button className="btn-primary" onClick={onRetry}>
          <RotateCcw size={18} />
          Try Again
        </button>
      </div>
    </div>
  )
}
