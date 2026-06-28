import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Loader2, CheckCircle, AlertTriangle, XCircle, ExternalLink } from 'lucide-react'

interface LaunchingProps {
  minecraftVersion?: string
  loader?: string
  loaderVersion?: string
  onBack: () => void
}

type JavaStatus = 'found' | 'too_old' | 'not_found'

interface JavaCheckResult {
  status: JavaStatus
  java_path: string | null
  version: number | null
  vendor: string | null
  minimum_required: number
}

type PageState =
  | { phase: 'checking_java' }
  | { phase: 'launching' }
  | { phase: 'launched' }
  | { phase: 'java_error'; result: JavaCheckResult }
  | { phase: 'launch_error'; message: string }

const ADOPTIUM_URL = 'https://adoptium.net/temurin/releases/?version=21'

export function LaunchingPage({ minecraftVersion, loader, loaderVersion, onBack }: LaunchingProps) {
  const [state, setState] = useState<PageState>({ phase: 'checking_java' })

  const loaderLabel =
    loader && loader !== 'vanilla' ? `${loader} ${loaderVersion ?? ''}`.trim() : 'Vanilla'

  useEffect(() => {
    let cancelled = false

    async function run() {
      // Step 1 — verify Java
      let javaResult: JavaCheckResult
      try {
        javaResult = await invoke<JavaCheckResult>('check_java')
      } catch {
        if (!cancelled) {
          setState({
            phase: 'java_error',
            result: {
              status: 'not_found',
              java_path: null,
              version: null,
              vendor: null,
              minimum_required: 17,
            },
          })
        }
        return
      }

      if (cancelled) return

      if (javaResult.status !== 'found') {
        setState({ phase: 'java_error', result: javaResult })
        return
      }

      // Step 2 — spawn game process
      setState({ phase: 'launching' })

      try {
        // launch_minecraft: spawns game process, redirects its output to
        // {game_dir}/logs/latest.log, then returns. If close_after_launch is
        // set in settings, the launcher process exits before this resolves.
        await invoke('launch_minecraft')

        // Only reached when close_after_launch = false.
        if (!cancelled) setState({ phase: 'launched' })
      } catch (err) {
        const message =
          err instanceof Error
            ? err.message
            : typeof err === 'string'
              ? err
              : 'Unknown launch error.'
        if (!cancelled) setState({ phase: 'launch_error', message })
      }
    }

    run()
    return () => {
      cancelled = true
    }
  }, [])

  const openAdoptium = () =>
    invoke('plugin:opener|open_url', { url: ADOPTIUM_URL }).catch(() => undefined)

  const versionLine = minecraftVersion ? (
    <p className="text-secondary text-[0.9rem]">
      Minecraft {minecraftVersion} · {loaderLabel}
    </p>
  ) : null

  // ── Checking Java ──────────────────────────────────────────────────────────
  if (state.phase === 'checking_java') {
    return (
      <div className="screen-container animate-fade-in center-all">
        <div className="glass-panel flex flex-col items-center text-center gap-4 p-10 max-w-[380px]">
          <Loader2 size={36} className="animate-spin text-accent" />
          <h2>Checking Java</h2>
          <p className="text-secondary text-[0.9rem]">
            Looking for a compatible Java installation…
          </p>
        </div>
      </div>
    )
  }

  // ── Launching ──────────────────────────────────────────────────────────────
  if (state.phase === 'launching') {
    return (
      <div className="screen-container animate-fade-in center-all">
        <div className="glass-panel flex flex-col items-center text-center gap-5 p-10 max-w-[380px]">
          <Loader2 size={36} className="animate-spin text-accent" />
          <div className="flex flex-col gap-1">
            <h2 className="text-gradient">Launching Wufus Craft</h2>
            {versionLine}
          </div>
          <p className="text-muted text-[0.85rem]">Starting the game process…</p>
        </div>
      </div>
    )
  }

  // ── Game started (close_after_launch = false) ──────────────────────────────
  if (state.phase === 'launched') {
    return (
      <div className="screen-container animate-fade-in center-all">
        <div className="glass-panel flex flex-col items-center text-center gap-5 p-10 max-w-[380px]">
          <CheckCircle size={40} className="text-success" />
          <div className="flex flex-col gap-1">
            <h2>Game Started</h2>
            {versionLine}
          </div>
          <p className="text-secondary text-[0.85rem]">
            Minecraft is running. You can close the launcher.
          </p>
          <button className="btn-secondary" onClick={onBack}>
            Back to main menu
          </button>
        </div>
      </div>
    )
  }

  // ── Launch error (launch.json missing, classpath wrong, etc.) ─────────────
  if (state.phase === 'launch_error') {
    return (
      <div className="screen-container animate-fade-in center-all">
        <div className="glass-panel flex flex-col items-center text-center gap-5 p-10 max-w-[420px]">
          <XCircle size={40} className="text-danger" />
          <div className="flex flex-col gap-2">
            <h2>Launch Failed</h2>
            <p className="text-secondary text-[0.9rem] leading-relaxed">
              The game could not be started.
            </p>
          </div>
          <div
            className="w-full rounded-sm p-4 text-left font-mono text-[0.75rem] break-all"
            style={{
              background: 'rgba(239,68,68,0.08)',
              border: '1px solid rgba(239,68,68,0.25)',
              color: 'var(--color-secondary)',
            }}
          >
            {state.message}
          </div>
          <p className="text-muted text-[0.78rem]">
            Check the game log in{' '}
            <span className="text-secondary">{'{game_dir}'}/logs/latest.log</span> for
            details.
          </p>
          <button className="btn-secondary" onClick={onBack}>
            Back to main menu
          </button>
        </div>
      </div>
    )
  }

  // ── Java error ──────────────────────────────────────────────────────────────
  const { result } = state as Extract<typeof state, { phase: 'java_error' }>

  const isTooOld = result.status === 'too_old'
  const Icon = isTooOld ? AlertTriangle : XCircle
  const iconClass = isTooOld ? 'text-orange' : 'text-danger'

  const errorTitle = isTooOld
    ? `Java ${result.version} is too old`
    : 'Java not found'

  const errorDesc = isTooOld
    ? `Minecraft requires Java ${result.minimum_required}+. You have Java ${result.version} (${result.vendor ?? 'unknown vendor'}).`
    : `No Java ${result.minimum_required}+ installation was found on this computer.`

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="glass-panel flex flex-col items-center text-center gap-5 p-10 max-w-[420px]">
        <Icon size={44} className={iconClass} />

        <div className="flex flex-col gap-2">
          <h2>{errorTitle}</h2>
          <p className="text-secondary text-[0.9rem] leading-relaxed">{errorDesc}</p>
        </div>

        <div
          className="w-full rounded-sm p-4 flex flex-col gap-2 text-left"
          style={{ background: 'rgba(138,43,226,0.1)', border: '1px solid rgba(138,43,226,0.25)' }}
        >
          <p className="text-[0.85rem] font-semibold text-primary">
            Install Eclipse Temurin 21 (recommended)
          </p>
          <p className="text-[0.8rem] text-secondary leading-relaxed">
            Eclipse Temurin is a free, production-ready OpenJDK build. Download the
            Windows x64 installer (.msi) and run it — no configuration needed.
          </p>
          <button
            className="btn-primary mt-1 flex items-center gap-2 text-sm px-4 py-2 self-start"
            onClick={openAdoptium}
          >
            <ExternalLink size={14} />
            Download Java 21
          </button>
        </div>

        <p className="text-muted text-[0.78rem]">
          You can also set a custom Java path in{' '}
          <span className="text-secondary">Settings → Java Path</span>.
        </p>

        <button className="btn-secondary" onClick={onBack}>
          Back to main menu
        </button>
      </div>
    </div>
  )
}
