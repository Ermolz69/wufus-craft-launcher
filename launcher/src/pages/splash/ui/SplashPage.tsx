import { useEffect } from 'react';
import { Loader2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import '../../../shared/styles/Screens.css';

interface SplashProps {
  onFinish: () => void;
  onError: (msg: string) => void;
}

export function SplashPage({ onFinish, onError }: SplashProps) {
  useEffect(() => {
    let isMounted = true;
    
    async function init() {
      try {
        await invoke('initialize_fs');
        
        // Add artificial delay just for splash feel
        setTimeout(() => {
          if (isMounted) onFinish();
        }, 1000);
      } catch (err: any) {
        console.error("Initialization error:", err);
        if (isMounted) onError(err);
      }
    }
    
    init();

    return () => {
      isMounted = false;
    };
  }, [onFinish, onError]);

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="splash-content">
        <h1 className="text-gradient splash-logo">Wufus Craft</h1>
        <div className="splash-loader">
          <Loader2 className="spinner" size={32} color="var(--accent-primary)" />
          <span>Starting up...</span>
        </div>
      </div>
    </div>
  );
}
