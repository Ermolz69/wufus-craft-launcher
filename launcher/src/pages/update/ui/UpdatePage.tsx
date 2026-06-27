import { useEffect, useState } from 'react';
import '../../../shared/styles/Screens.css';

interface UpdateProps {
  onFinish: () => void;
  onError: () => void;
}

export function UpdatePage({ onFinish, onError }: UpdateProps) {
  const [progress, setProgress] = useState(0);

  // Simulate update progress
  useEffect(() => {
    const interval = setInterval(() => {
      setProgress(p => {
        if (p >= 100) {
          clearInterval(interval);
          onFinish();
          return 100;
        }
        return p + 2;
      });
    }, 50);
    return () => clearInterval(interval);
  }, [onFinish]);

  return (
    <div className="screen-container animate-fade-in center-all">
      <div className="update-panel glass-panel">
        <h2 className="text-gradient">Updating Wufus Craft</h2>
        <p className="update-status">Downloading assets... {progress}%</p>
        
        <div className="progress-bar-bg">
          <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
        </div>
        
        <div className="update-actions">
          <button className="btn-secondary" onClick={onError}>Simulate Error</button>
        </div>
      </div>
    </div>
  );
}
