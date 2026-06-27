import { Play, Settings as SettingsIcon } from 'lucide-react';
import '../../../shared/styles/Screens.css';

interface MainProps {
  onNavigate: (screen: string) => void;
}

export function MainPage({ onNavigate }: MainProps) {
  return (
    <div className="screen-container animate-fade-in main-screen">
      
      <div className="main-content">
        <div className="news-panel glass-panel animate-slide-up">
          <h3>Latest News</h3>
          <p className="news-text">Welcome to Wufus Craft! The server is currently online. Enjoy the new update with custom features.</p>
        </div>
      </div>

      <div className="bottom-bar glass-panel animate-slide-up">
        <div className="user-info">
          <div className="avatar" />
          <div className="user-details">
            <span className="username">Player123</span>
            <span className="status">Ready to play</span>
          </div>
        </div>

        <button className="btn-primary play-btn" onClick={() => onNavigate('update')}>
          <Play fill="currentColor" size={24} />
          <span>PLAY</span>
        </button>

        <div className="actions">
          <button className="icon-btn" onClick={() => onNavigate('settings')}>
            <SettingsIcon size={20} />
          </button>
        </div>
      </div>
    </div>
  );
}
