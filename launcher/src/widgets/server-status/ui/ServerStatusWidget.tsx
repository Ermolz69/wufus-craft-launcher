import { useCallback, useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface ServerStatusResult {
  state: 'online' | 'offline'
  players: number | null
  max_players: number | null
  ping_ms: number | null
  version: string | null
}

type UIState =
  | { kind: 'checking' }
  | { kind: 'online'; players: number; maxPlayers: number; ping: number }
  | { kind: 'offline' }

const POLL_INTERVAL_MS = 5 * 60 * 1000 // 5 minutes

export function ServerStatusWidget() {
  const [status, setStatus] = useState<UIState>({ kind: 'checking' })
  const pendingRef = useRef(false)

  const check = useCallback(async () => {
    // Skip if a check is already in flight (3s TCP timeout keeps this tight)
    if (pendingRef.current) return
    pendingRef.current = true

    try {
      const result = await invoke<ServerStatusResult>('check_server_status')
      if (result.state === 'online') {
        setStatus({
          kind: 'online',
          players: result.players ?? 0,
          maxPlayers: result.max_players ?? 0,
          ping: result.ping_ms ?? 0,
        })
      } else {
        setStatus({ kind: 'offline' })
      }
    } catch {
      setStatus({ kind: 'offline' })
    } finally {
      pendingRef.current = false
    }
  }, [])

  useEffect(() => {
    check()
    const id = setInterval(check, POLL_INTERVAL_MS)
    return () => clearInterval(id)
  }, [check])

  const dot = (color: string) => (
    <span
      className={`inline-block w-2 h-2 rounded-full shrink-0 ${color}`}
      aria-hidden="true"
    />
  )

  if (status.kind === 'checking') {
    return (
      <div className="flex flex-col items-end gap-0.5 w-[120px]">
        <div className="flex items-center gap-1.5">
          {dot('bg-muted opacity-60 animate-pulse')}
          <span className="text-muted text-[0.82rem]">Checking…</span>
        </div>
      </div>
    )
  }

  if (status.kind === 'offline') {
    return (
      <div className="flex flex-col items-end gap-0.5 w-[120px]">
        <div className="flex items-center gap-1.5">
          {dot('bg-danger')}
          <span className="text-[0.82rem] font-medium text-primary">Offline</span>
        </div>
        <span className="text-muted text-[0.72rem]">Server unavailable</span>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-end gap-0.5 w-[120px]">
      <div className="flex items-center gap-1.5">
        {dot('bg-success')}
        <span className="text-[0.82rem] font-medium text-primary">Online</span>
      </div>
      <span className="text-muted text-[0.72rem]">
        {status.players}/{status.maxPlayers} · {status.ping} ms
      </span>
    </div>
  )
}
