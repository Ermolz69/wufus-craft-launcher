import { ArrowLeft, Save, FolderOpen, RotateCcw, Loader2, Search } from 'lucide-react'
import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface SettingsProps {
  onBack: () => void
}

interface LauncherSettings {
  game_path: string
  ram_gb: number
  close_after_launch: boolean
  minimize_on_close: boolean
  java_path: string | null
}

interface JavaCheckResult {
  status: 'found' | 'too_old' | 'not_found'
  java_path: string | null
  version: number | null
  vendor: string | null
  minimum_required: number
}

export function SettingsPage({ onBack }: SettingsProps) {
  const [settings, setSettings] = useState<LauncherSettings | null>(null)
  const [isSaving, setIsSaving] = useState(false)
  const [logError, setLogError] = useState<string | null>(null)
  const [javaDetecting, setJavaDetecting] = useState(false)
  const [javaStatus, setJavaStatus] = useState<string | null>(null)

  const loadSettings = useCallback(async () => {
    try {
      const data = await invoke<LauncherSettings>('get_settings')
      setSettings(data)
    } catch (e) {
      console.error('Failed to load settings', e)
    }
  }, [])

  useEffect(() => { loadSettings() }, [loadSettings])

  const handleSave = async () => {
    if (!settings) return
    setIsSaving(true)
    try {
      await invoke('save_settings', { settings })
      onBack()
    } catch (e) {
      console.error('Failed to save settings', e)
      await loadSettings()
      setIsSaving(false)
    }
  }

  const handleReset = async () => {
    try {
      const defaultSettings = await invoke<LauncherSettings>('reset_settings')
      setSettings(defaultSettings)
    } catch (e) {
      console.error('Failed to reset settings', e)
    }
  }

  const handleDetectJava = async () => {
    setJavaDetecting(true)
    setJavaStatus(null)
    try {
      const result = await invoke<JavaCheckResult>('check_java')
      if (result.status === 'found' && result.java_path) {
        setSettings((s) => s ? { ...s, java_path: result.java_path } : s)
        setJavaStatus(`✓ Java ${result.version} (${result.vendor ?? 'unknown'}) detected`)
      } else if (result.status === 'too_old') {
        setJavaStatus(`Java ${result.version} found but ${result.minimum_required}+ is required`)
      } else {
        setJavaStatus('No compatible Java found')
      }
    } catch {
      setJavaStatus('Detection failed')
    } finally {
      setJavaDetecting(false)
    }
  }

  const handleOpenLogs = async () => {
    setLogError(null)
    try {
      await invoke('open_logs_folder')
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : String(e)
      console.error('Failed to open logs', e)
      setLogError(message)
    }
  }

  if (!settings) {
    return (
      <div className="screen-container center-all">
        <Loader2 size={48} className="animate-spin text-accent" />
      </div>
    )
  }

  return (
    <div className="screen-container animate-slide-up flex-col p-8">
      {/* Header */}
      <div className="flex items-center gap-4 mb-8">
        <button className="icon-btn" onClick={onBack}>
          <ArrowLeft />
        </button>
        <h2>Settings</h2>
      </div>

      {/* Content panel */}
      <div className="glass-panel flex flex-col gap-8 max-w-[600px] p-6 overflow-y-auto">
        {/* RAM slider */}
        <div className="flex flex-col gap-3">
          <label className="font-medium text-secondary">
            RAM Allocation (GB): {settings.ram_gb}GB
          </label>
          <input
            type="range"
            min="2"
            max="32"
            value={settings.ram_gb}
            onChange={(e) => setSettings({ ...settings, ram_gb: parseInt(e.target.value) })}
          />
          <div className="flex justify-between text-[0.8rem] text-muted">
            <span>2GB</span>
            <span>32GB</span>
          </div>
        </div>

        {/* Installation path */}
        <div className="flex flex-col gap-3">
          <label className="font-medium text-secondary">Installation Path</label>
          <input
            type="text"
            value={settings.game_path}
            onChange={(e) => setSettings({ ...settings, game_path: e.target.value })}
          />
        </div>

        {/* Java Path */}
        <div className="flex flex-col gap-3">
          <label className="font-medium text-secondary">
            Java Path{' '}
            <span className="text-muted text-[0.8rem] font-normal">(leave empty to auto-detect)</span>
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={settings.java_path ?? ''}
              onChange={(e) =>
                setSettings({ ...settings, java_path: e.target.value.trim() || null })
              }
              placeholder="Auto-detect"
            />
            <button
              className="btn-secondary shrink-0 px-3"
              onClick={handleDetectJava}
              disabled={javaDetecting}
              title="Auto-detect Java"
            >
              {javaDetecting ? (
                <Loader2 size={16} className="animate-spin" />
              ) : (
                <Search size={16} />
              )}
            </button>
          </div>
          {javaStatus && (
            <p className={`text-[0.8rem] ${javaStatus.startsWith('✓') ? 'text-success' : 'text-orange'}`}>
              {javaStatus}
            </p>
          )}
        </div>

        {/* Checkboxes */}
        <div className="flex flex-col gap-4">
          <label className="flex items-center gap-2 cursor-pointer font-medium text-secondary">
            <input
              type="checkbox"
              checked={settings.close_after_launch}
              onChange={(e) => setSettings({ ...settings, close_after_launch: e.target.checked })}
              className="accent-accent"
            />
            Close launcher after game starts
          </label>
          <label className="flex items-center gap-2 cursor-pointer font-medium text-secondary">
            <input
              type="checkbox"
              checked={settings.minimize_on_close}
              onChange={(e) => setSettings({ ...settings, minimize_on_close: e.target.checked })}
              className="accent-accent"
            />
            Minimize to tray on close
          </label>
        </div>

        {/* Utility buttons */}
        <div
          className="flex gap-4 pt-6"
          style={{ borderTop: '1px solid var(--border-strong)' }}
        >
          <button className="btn-secondary flex-1" onClick={handleReset}>
            <RotateCcw size={18} />
            Reset Defaults
          </button>
          <button className="btn-secondary flex-1" onClick={handleOpenLogs}>
            <FolderOpen size={18} />
            Open Logs
          </button>
        </div>

        {logError && <p className="text-danger text-xs">{logError}</p>}
      </div>

      {/* Save footer */}
      <div className="mt-8 max-w-[600px]">
        <button className="btn-primary" onClick={handleSave} disabled={isSaving}>
          {isSaving ? <Loader2 size={18} className="animate-spin" /> : <Save size={18} />}
          Save Changes
        </button>
      </div>
    </div>
  )
}
