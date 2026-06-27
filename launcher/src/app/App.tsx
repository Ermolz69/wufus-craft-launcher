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
  const [errorMsg, setErrorMsg] = useState<string>('')

  const handleError = (msg: string) => {
    setErrorMsg(msg)
    setCurrentScreen('error')
  }

  const renderScreen = () => {
    switch (currentScreen) {
      case 'splash':
        return <SplashPage onComplete={() => setCurrentScreen('main')} onError={handleError} />
      case 'main':
        return (
          <MainPage
            onNavigate={(s) => {
              if (isScreen(s)) setCurrentScreen(s)
            }}
          />
        )
      case 'update':
        return (
          <UpdatePage
            onComplete={() => setCurrentScreen('main')}
            onError={handleError}
            onCancel={() => setCurrentScreen('main')}
          />
        )
      case 'settings':
        return <SettingsPage onBack={() => setCurrentScreen('main')} />
      case 'error':
        return <ErrorPage message={errorMsg} onRetry={() => setCurrentScreen('splash')} />
    }
  }

  return <Layout>{renderScreen()}</Layout>
}

export default App
