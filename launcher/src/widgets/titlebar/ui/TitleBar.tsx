import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, X } from 'lucide-react';
import './TitleBar.css';

export function TitleBar() {
  const appWindow = getCurrentWindow();

  return (
    <div data-tauri-drag-region className="titlebar">
      <div className="titlebar-logo">
        Wufus Craft
      </div>
      <div className="titlebar-controls">
        <div 
          className="titlebar-btn" 
          onClick={() => appWindow.minimize()}
        >
          <Minus size={16} />
        </div>
        <div 
          className="titlebar-btn close" 
          onClick={() => appWindow.close()}
        >
          <X size={16} />
        </div>
      </div>
    </div>
  );
}
