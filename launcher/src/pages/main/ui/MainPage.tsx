import { useEffect, useState } from 'react'
import { Loader2, Play } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { NewsCard, type NewsItem } from '../../../entities/news'
import { ServerStatusWidget } from '../../../widgets/server-status/ui/ServerStatusWidget'

interface BuildReadiness {
  status: 'ready' | 'needs_update' | 'needs_repair'
  minecraft_version: string | null
  loader: string | null
  loader_version: string | null
  reason: string | null
}

interface MainProps {
  onNavigate: (screen: 'update' | 'settings' | 'launching') => void
  onRepair: () => void
  onError: (msg: string, kind?: string) => void
  onReady: (info: { minecraftVersion?: string; loader?: string; loaderVersion?: string }) => void
}

const MAX_NEWS = 5

export function MainPage({ onNavigate, onRepair, onError, onReady }: MainProps) {
  const [isChecking, setIsChecking] = useState(false)
  const [news, setNews] = useState<NewsItem[]>([])
  const [newsLoading, setNewsLoading] = useState(true)

  // Load news independently — does not block play or cause errors.
  useEffect(() => {
    invoke<NewsItem[]>('fetch_news')
      .then(setNews)
      .catch(() => {/* already handled server-side; empty list is fine */})
      .finally(() => setNewsLoading(false))
  }, [])

  const handlePlay = async () => {
    setIsChecking(true)
    try {
      const result = await invoke<BuildReadiness>('prepare_launch')
      switch (result.status) {
        case 'ready':
          onReady({
            minecraftVersion: result.minecraft_version ?? undefined,
            loader: result.loader ?? undefined,
            loaderVersion: result.loader_version ?? undefined,
          })
          onNavigate('launching')
          break
        case 'needs_update':
          onNavigate('update')
          break
        case 'needs_repair':
          onRepair()
          break
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      onError(msg, 'internal')
    } finally {
      setIsChecking(false)
    }
  }

  return (
    <div className="screen-container animate-fade-in flex-col justify-between p-8">
      {/* News panel */}
      <div className="flex-1 flex flex-col justify-end gap-2 pb-4 min-h-0">
        {newsLoading ? (
          /* Skeleton shown only briefly while the cache/network call resolves */
          <div className="flex flex-col gap-2 max-w-[440px]">
            {[1, 2].map((i) => (
              <div
                key={i}
                className="glass-panel p-3 h-[72px] animate-pulse"
                style={{ opacity: 0.4 }}
              />
            ))}
          </div>
        ) : news.length > 0 ? (
          <div className="flex flex-col gap-2 max-w-[440px] overflow-y-auto">
            {news.slice(0, MAX_NEWS).map((item) => (
              <NewsCard key={item.id} item={item} />
            ))}
          </div>
        ) : (
          /* No news — section simply disappears, layout stays intact */
          null
        )}
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
          onClick={handlePlay}
          disabled={isChecking}
        >
          {isChecking ? (
            <Loader2 size={22} className="animate-spin" />
          ) : (
            <Play fill="currentColor" size={24} />
          )}
          <span>{isChecking ? 'Checking...' : 'PLAY'}</span>
        </button>

        <ServerStatusWidget />
      </div>
    </div>
  )
}
