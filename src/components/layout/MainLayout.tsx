import { ReactNode } from 'react';
import { Navbar } from './Navbar';
import { LogsDrawer } from './LogsDrawer';

interface Props {
  children: ReactNode;
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function MainLayout({ children, activeTab, onTabChange }: Props) {
  return (
    <div className="min-h-screen bg-background">
      <Navbar activeTab={activeTab} onTabChange={onTabChange} />
      <main className="container mx-auto p-4 pt-20">
        {children}
      </main>
      <LogsDrawer />
    </div>
  );
}
