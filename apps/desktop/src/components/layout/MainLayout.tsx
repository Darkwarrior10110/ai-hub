import { Sidebar } from '../sidebar/Sidebar';
import { ChatArea } from '../chat/ChatArea';
import { useUI } from '../../providers/UIProvider';
import './MainLayout.css';

export function MainLayout() {
  const { isSidebarCollapsed } = useUI();

  return (
    <div className={`main-layout ${isSidebarCollapsed ? 'sidebar-collapsed' : ''}`}>
      <Sidebar />
      <main className="main-content">
        <ChatArea />
      </main>
    </div>
  );
}
