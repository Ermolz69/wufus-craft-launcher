import { useEffect, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { IconLogo } from '../../../shared/icons/IconLogo'
import { IconMinimize } from '../../../shared/icons/IconMinimize'
import { IconMaximize } from '../../../shared/icons/IconMaximize'
import { IconRestore } from '../../../shared/icons/IconRestore'
import { IconClose } from '../../../shared/icons/IconClose'
import { IconSettings } from '../../../shared/icons/IconSettings'

interface TitleBarProps {
  onSettings?: () => void
}

export function TitleBar({ onSettings }: TitleBarProps) {
  const [isMaximized, setIsMaximized] = useState(false)

  useEffect(() => {
    const win = getCurrentWindow()
    win.isMaximized().then(setIsMaximized).catch(() => undefined)

    let unlisten: (() => void) | undefined
    win
      .onResized(async () => { setIsMaximized(await win.isMaximized()) })
      .then((fn) => { unlisten = fn })
      .catch(() => undefined)

    return () => unlisten?.()
  }, [])

  // startDragging() is called imperatively on the drag zones only.
  // The button area is intentionally NOT a drag zone, so buttons always
  // receive click events — data-tauri-drag-region is NOT used anywhere.
  const startDrag = (e: React.MouseEvent) => {
    if (e.button !== 0) return
    getCurrentWindow().startDragging().catch(() => undefined)
  }

  const minimize      = () => getCurrentWindow().minimize().catch(() => undefined)
  const toggleMaximize = () => getCurrentWindow().toggleMaximize().catch(() => undefined)
  const close         = () => getCurrentWindow().close().catch(() => undefined)

  const btnBase =
    'flex items-center justify-center h-full cursor-pointer select-none ' +
    'bg-transparent border-0 rounded-none p-0 m-0 outline-none ' +
    'text-secondary transition-colors duration-150 ' +
    'hover:bg-surface-hover hover:text-primary ' +
    '[&>*]:pointer-events-none'

  return (
    <div className="absolute inset-x-0 top-0 h-9 z-[9999] flex items-stretch select-none">
      {/* Drag zone: logo + title */}
      <div
        className="flex items-center gap-2 pl-4 pr-3 shrink-0 cursor-move"
        onMouseDown={startDrag}
      >
        <IconLogo size={18} className="shrink-0 pointer-events-none" />
        <span className="text-[13px] font-semibold text-secondary whitespace-nowrap pointer-events-none">
          Wufus Craft
        </span>
      </div>

      {/* Drag zone: center stretch */}
      <div className="flex-1 min-w-0 cursor-move" onMouseDown={startDrag} />

      {/* Non-drag zone: window controls */}
      <div className="flex items-stretch shrink-0">
        {onSettings && (
          <>
            <button
              className={`${btnBase} w-[42px]`}
              title="Settings"
              onClick={onSettings}
            >
              <IconSettings size={15} />
            </button>
            <div
              className="w-px my-2 mx-0.5 shrink-0 pointer-events-none"
              style={{ background: 'var(--color-border-subtle)' }}
              aria-hidden="true"
            />
          </>
        )}

        <button className={`${btnBase} w-[46px]`} title="Minimize" onClick={minimize}>
          <IconMinimize size={12} />
        </button>

        <button
          className={`${btnBase} w-[46px]`}
          title={isMaximized ? 'Restore' : 'Maximize'}
          onClick={toggleMaximize}
        >
          {isMaximized ? <IconRestore size={12} /> : <IconMaximize size={12} />}
        </button>

        <button
          className={`${btnBase} w-[46px] hover:!bg-danger hover:!text-white`}
          title="Close"
          onClick={close}
        >
          <IconClose size={12} />
        </button>
      </div>
    </div>
  )
}
