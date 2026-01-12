import { useEffect, useState } from 'react';
import { MainLayout } from './components/layout/MainLayout';
import { TranslationPage } from './components/translation/TranslationPage';
import { ConfigPage } from './components/config/ConfigPage';
import { useSettingsStore } from './stores/settingsStore';
import './App.css';

function App() {
  const [activeTab, setActiveTab] = useState('translation');
  const { loadSettings, loadTemplates, checkFfmpeg } = useSettingsStore();

  useEffect(() => {
    // Load all data on startup
    loadSettings();
    loadTemplates();
    checkFfmpeg();
  }, []);

  return (
    <MainLayout activeTab={activeTab} onTabChange={setActiveTab}>
      {activeTab === 'translation' ? <TranslationPage /> : <ConfigPage />}
    </MainLayout>
  );
}

export default App;
