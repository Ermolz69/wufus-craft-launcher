import { ArrowLeft, Save, FolderOpen, RotateCcw, Loader2 } from 'lucide-react'
import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import '../../../shared/styles/Screens.css'

interface SettingsProps {
  onBack: () => void
}

interface LauncherSettings {
  game_path: string
  ram_gb: number
  close_after_launch: boolean
  minimize_on_close: boolean
}

export function SettingsPage({ onBack }: SettingsProps) {
  const [settings, setSettings] = useState<LauncherSettings | null>(null)
  const [isSaving, setIsSaving] = useState(false)
  const [logError, setLogError] = useState<string | null>(null)

  const loadSettings = useCallback(async () => {
    try {
      const data = await invoke<LauncherSettings>('get_settings')
      setSettings(data)
    } catch (e) {
      console.error('Failed to load settings', e)
    }
  }, [])

  useEffect(() => {
    loadSettings()
  }, [loadSettings])

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
        <Loader2 className="animate-spin" size={48} />
      </div>
    )
  }

  return (
    <div className="screen-container animate-slide-up settings-screen">
      <div className="settings-header">
        <button className="icon-btn" onClick={onBack}>
          <ArrowLeft />
        </button>
        <h2>Settings</h2>
      </div>

      <div className="settings-content glass-panel" style={{ overflowY: 'auto', padding: '24px' }}>
        <div className="settings-group">
          <label>RAM Allocation (GB): {settings.ram_gb}GB</label>
          <input
            type="range"
            min="2"
            max="32"
            value={settings.ram_gb}
            onChange={(e) => setSettings({ ...settings, ram_gb: parseInt(e.target.value) })}
            className="range-slider"
          />
          <div className="range-labels">
            <span>2GB</span>
            <span>32GB</span>
          </div>
        </div>

        <div className="settings-group">
          <label>Installation Path</label>
          <div className="path-input" style={{ display: 'flex', gap: '8px' }}>
            <input
              type="text"
              value={settings.game_path}
              onChange={(e) => setSettings({ ...settings, game_path: e.target.value })}
              style={{ flex: 1 }}
            />
          </div>
        </div>

        <div className="settings-group" style={{ marginTop: '24px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.close_after_launch}
              onChange={(e) => setSettings({ ...settings, close_after_launch: e.target.checked })}
            />
            Close launcher after game starts
          </label>
        </div>

        <div className="settings-group" style={{ marginTop: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.minimize_on_close}
              onChange={(e) => setSettings({ ...settings, minimize_on_close: e.target.checked })}
            />
            Minimize to tray on close
          </label>
        </div>

        <div
          className="settings-group"
          style={{
            marginTop: '32px',
            borderTop: '1px solid rgba(255,255,255,0.1)',
            paddingTop: '24px',
            display: 'flex',
            gap: '16px',
          }}
        >
          <button
            className="btn-secondary"
            onClick={handleReset}
            style={{ flex: 1, display: 'flex', justifyContent: 'center', gap: '8px' }}
          >
            <RotateCcw size={18} />
            Reset Defaults
          </button>
          <button
            className="btn-secondary"
            onClick={handleOpenLogs}
            style={{ flex: 1, display: 'flex', justifyContent: 'center', gap: '8px' }}
          >
            <FolderOpen size={18} />
            Open Logs
          </button>
        </div>
        {logError && (
          <p style={{ color: 'var(--color-error, #e74c3c)', fontSize: '12px', marginTop: '8px' }}>
            {logError}
          </p>
        )}
      </div>

      <div className="settings-footer">
        <button className="btn-primary" onClick={handleSave} disabled={isSaving}>
          {isSaving ? <Loader2 className="animate-spin" size={18} /> : <Save size={18} />}
          Save Changes
        </button>
      </div>
    </div>
  )
}
