import { useState } from 'react'
import { Layout } from '../widgets/layout/ui/Layout'
import { SplashPage } from '../pages/splash/ui/SplashPage'
import { MainPage } from '../pages/main/ui/MainPage'
import { UpdatePage } from '../pages/update/ui/UpdatePage'
import { SettingsPage } from '../pages/settings/ui/SettingsPage'
import { ErrorPage } from '../pages/error/ui/ErrorPage'
import { LaunchingPage } from '../pages/launching/ui/LaunchingPage'
import { type Screen } from '../shared/types/screens'

interface BuildInfo {
  minecraftVersion?: string
  loader?: string
  loaderVersion?: string
}

function App() {
  const [currentScreen, setCurrentScreen] = useState<Screen>('splash')
  const [errorMsg, setErrorMsg] = useState('')
  const [errorKind, setErrorKind] = useState('internal')
  const [updateMode, setUpdateMode] = useState<'update' | 'repair'>('update')
  const [buildInfo, setBuildInfo] = useState<BuildInfo>({})

  const handleError = (msg: string, kind = 'internal') => {
    setErrorMsg(msg)
    setErrorKind(kind)
    setCurrentScreen('error')
  }

  const handleRepair = () => {
    setUpdateMode('repair')
    setCurrentScreen('update')
  }

  const handleSettings = () => setCurrentScreen('settings')

  const handleReady = (info: BuildInfo) => setBuildInfo(info)

  const renderScreen = () => {
    switch (currentScreen) {
      case 'splash':
        return <SplashPage onComplete={() => setCurrentScreen('main')} onError={handleError} />

      case 'main':
        return (
          <MainPage
            onNavigate={(s) => {
              setUpdateMode('update')
              setCurrentScreen(s)
            }}
            onRepair={handleRepair}
            onError={handleError}
            onReady={handleReady}
          />
        )

      case 'update':
        return (
          <UpdatePage
            mode={updateMode}
            onComplete={() => setCurrentScreen('main')}
            onError={handleError}
            onCancel={() => setCurrentScreen('main')}
          />
        )

      case 'launching':
        return (
          <LaunchingPage
            minecraftVersion={buildInfo.minecraftVersion}
            loader={buildInfo.loader}
            loaderVersion={buildInfo.loaderVersion}
            onBack={() => setCurrentScreen('main')}
          />
        )

      case 'settings':
        return <SettingsPage onBack={() => setCurrentScreen('main')} />

      case 'error':
        return (
          <ErrorPage
            message={errorMsg}
            kind={errorKind}
            onRetry={() => setCurrentScreen('splash')}
            onRepair={handleRepair}
          />
        )
    }
  }

  return <Layout onSettings={handleSettings}>{renderScreen()}</Layout>
}

export default App
