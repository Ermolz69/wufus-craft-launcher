import { Play } from 'lucide-react'
import type { Screen } from '../../../shared/types/screens'

interface MainProps {
  onNavigate: (screen: Screen) => void
}

export function MainPage({ onNavigate }: MainProps) {
  return (
    <div className="screen-container animate-fade-in flex-col justify-between p-8">
      {/* News panel anchored to the bottom-left of the content area */}
      <div className="flex-1 flex items-end">
        <div className="glass-panel animate-slide-up p-6 max-w-[400px] mb-6">
          <h3 className="mb-3 text-[1.2rem]">Latest News</h3>
          <p className="text-secondary leading-relaxed">
            Welcome to Wufus Craft! The server is currently online. Enjoy the new update with custom
            features.
          </p>
        </div>
      </div>

      {/* Bottom bar */}
      <div className="glass-panel animate-slide-up flex items-center justify-between px-6 py-4">
        {/* User info */}
        <div className="flex items-center gap-3">
          <div
            className="w-10 h-10 rounded-sm shrink-0 bg-surface-hover"
            style={{ border: '1px solid var(--border-strong)' }}
          />
          <div className="flex flex-col">
            <span className="font-semibold text-[1.1rem]">Player123</span>
            <span className="text-[0.85rem] text-muted">Ready to play</span>
          </div>
        </div>

        <button
          className="btn-primary text-[1.2rem] px-12 py-3"
          onClick={() => onNavigate('update')}
        >
          <Play fill="currentColor" size={24} />
          <span>PLAY</span>
        </button>

        {/* Spacer to keep PLAY button visually centred */}
        <div className="w-[120px]" />
      </div>
    </div>
  )
}
