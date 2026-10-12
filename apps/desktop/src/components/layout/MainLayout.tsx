import { Sidebar } from '../sidebar/Sidebar';
import { GatewayDashboard } from '../dashboard/GatewayDashboard';
import { useUI } from '../../providers/UIProvider';
import './MainLayout.css';

export function MainLayout() {
  const { isSidebarCollapsed } = useUI();

  return (
    <div className={`main-layout ${isSidebarCollapsed ? 'sidebar-collapsed' : ''}`}>
      <Sidebar />
      <main className="main-content">
        <GatewayDashboard />
      </main>
    </div>
  );
}
