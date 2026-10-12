import { UIProvider } from './providers/UIProvider';
import { MainLayout } from './components/layout/MainLayout';
import { SettingsModal } from './components/settings/SettingsModal';

function App() {
  return (
    <UIProvider>
      <MainLayout />
      <SettingsModal />
    </UIProvider>
  );
}

export default App;
