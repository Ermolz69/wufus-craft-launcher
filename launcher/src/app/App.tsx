import { useState } from "react";
import { Layout } from "../widgets/layout/ui/Layout";
import { SplashPage } from "../pages/splash/ui/SplashPage";
import { MainPage } from "../pages/main/ui/MainPage";
import { UpdatePage } from "../pages/update/ui/UpdatePage";
import { SettingsPage } from "../pages/settings/ui/SettingsPage";
import { ErrorPage } from "../pages/error/ui/ErrorPage";

type Screen = 'splash' | 'main' | 'update' | 'settings' | 'error';

function App() {
  const [currentScreen, setCurrentScreen] = useState<Screen>('splash');
  const [errorMsg, setErrorMsg] = useState<string>('');

  const handleError = (msg: string) => {
    setErrorMsg(msg);
    setCurrentScreen('error');
  };

  const renderScreen = () => {
    switch (currentScreen) {
      case 'splash':
        return (
          <SplashPage 
            onFinish={() => setCurrentScreen('main')} 
            onError={handleError} 
          />
        );
      case 'main':
        return <MainPage onNavigate={(s) => setCurrentScreen(s as Screen)} />;
      case 'update':
        return (
          <UpdatePage 
            onFinish={() => setCurrentScreen('main')} 
            onError={() => handleError("Update failed")} 
          />
        );
      case 'settings':
        return <SettingsPage onBack={() => setCurrentScreen('splash')} />;
      case 'error':
        return (
          <ErrorPage 
            message={errorMsg}
            onRetry={() => {
              // Usually we'd want to go back to splash to retry initialization
              setCurrentScreen('splash');
            }} 
          />
        );
      default:
        return <MainPage onNavigate={(s) => setCurrentScreen(s as Screen)} />;
    }
  };

  return (
    <Layout>
      {renderScreen()}
    </Layout>
  );
}

export default App;
