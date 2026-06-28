import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { ExternalLink } from 'lucide-react'

export interface NewsItem {
  id: string
  title: string
  body: string
  date: string
  image_url: string | null
  link_url: string | null
}

interface NewsCardProps {
  item: NewsItem
}

export function NewsCard({ item }: NewsCardProps) {
  const [imgFailed, setImgFailed] = useState(false)

  const open = () => {
    if (item.link_url) {
      invoke('plugin:opener|open_url', { url: item.link_url }).catch(() => undefined)
    }
  }

  return (
    <div
      role={item.link_url ? 'button' : undefined}
      tabIndex={item.link_url ? 0 : undefined}
      onClick={item.link_url ? open : undefined}
      onKeyDown={
        item.link_url ? (e) => e.key === 'Enter' && open() : undefined
      }
      className={[
        'glass-panel flex items-start gap-3 p-3 transition-colors duration-150',
        item.link_url
          ? 'cursor-pointer hover:bg-surface-hover focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent'
          : '',
      ].join(' ')}
    >
      {item.image_url && !imgFailed && (
        <img
          src={item.image_url}
          alt=""
          loading="lazy"
          className="w-[52px] h-[52px] rounded-sm object-cover shrink-0"
          onError={() => setImgFailed(true)}
        />
      )}

      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <div className="flex items-start justify-between gap-2">
          <span className="font-semibold text-[0.88rem] leading-snug line-clamp-1 text-primary">
            {item.title}
          </span>
          {item.link_url && (
            <ExternalLink size={11} className="text-muted shrink-0 mt-0.5" />
          )}
        </div>
        <span className="text-muted text-[0.73rem]">{item.date}</span>
        <p className="text-secondary text-[0.8rem] leading-relaxed line-clamp-2 mt-0.5">
          {item.body}
        </p>
      </div>
    </div>
  )
}
