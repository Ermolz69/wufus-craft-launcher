import { useState } from 'react'
import { Layout } from '../widgets/layout/ui/Layout'
import { SplashPage } from '../pages/splash/ui/SplashPage'
import { MainPage } from '../pages/main/ui/MainPage'
import { UpdatePage } from '../pages/update/ui/UpdatePage'
import { SettingsPage } from '../pages/settings/ui/SettingsPage'
import { ErrorPage } from '../pages/error/ui/ErrorPage'
import { type Screen, isScreen } from '../shared/types/screens'

function App() {
  const [currentScreen, setCurrentScreen] = useState<Screen>('splash')
  const [errorMsg, setErrorMsg] = useState('')
  const [errorKind, setErrorKind] = useState('internal')
  const [updateMode, setUpdateMode] = useState<'update' | 'repair'>('update')

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

  const renderScreen = () => {
    switch (currentScreen) {
      case 'splash':
        return <SplashPage onComplete={() => setCurrentScreen('main')} onError={handleError} />
      case 'main':
        return (
          <MainPage
            onNavigate={(s) => {
              if (isScreen(s)) {
                setUpdateMode('update')
                setCurrentScreen(s)
              }
            }}
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
