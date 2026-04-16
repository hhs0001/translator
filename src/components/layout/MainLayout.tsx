import { ReactNode } from 'react';
import { motion } from 'framer-motion';
import { Navbar } from './Navbar';
import { LogsDrawer } from './LogsDrawer';

interface Props {
  children: ReactNode;
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function MainLayout({ children, activeTab, onTabChange }: Props) {
  return (
    <div className="min-h-[100dvh] bg-background">
      {/* Background gradient ambient effect */}
      <div className="fixed inset-0 z-0 pointer-events-none overflow-hidden">
        <div className="absolute -top-[40%] -left-[20%] w-[80%] h-[80%] rounded-full bg-primary/[0.03] blur-[120px]" />
        <div className="absolute -bottom-[40%] -right-[20%] w-[80%] h-[80%] rounded-full bg-primary/[0.02] blur-[120px]" />
      </div>

      <Navbar activeTab={activeTab} onTabChange={onTabChange} />
      
      <main className="relative z-10 container-premium pt-24 pb-8">
        <motion.div
          key={activeTab}
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ 
            duration: 0.3, 
            ease: [0.16, 1, 0.3, 1] 
          }}
        >
          {children}
        </motion.div>
      </main>
      
      <LogsDrawer />
    </div>
  );
}
