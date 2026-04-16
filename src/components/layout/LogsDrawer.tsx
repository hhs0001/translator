import { useTranslation } from "react-i18next";
import { motion, AnimatePresence } from "framer-motion";
import { 
  Trash, 
  Faders,
  CheckCircle,
  WarningCircle,
  Info,
  Prohibit,
  FileText
} from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useLogsStore } from "../../stores/logsStore";
import { LogEntry, LogLevel } from "../../types";
import { memo, useEffect, useMemo, useState } from "react";
import { useShallow } from "zustand/shallow";

const LEVEL_CONFIG: Record<LogLevel, { 
  icon: React.ComponentType<{ className?: string }>;
  className: string;
  bgClass: string;
}> = {
  info: { 
    icon: Info, 
    className: 'text-info',
    bgClass: 'bg-info/10',
  },
  success: { 
    icon: CheckCircle, 
    className: 'text-success',
    bgClass: 'bg-success/10',
  },
  warning: { 
    icon: WarningCircle, 
    className: 'text-warning',
    bgClass: 'bg-warning/10',
  },
  error: { 
    icon: Prohibit, 
    className: 'text-error',
    bgClass: 'bg-error/10',
  },
};

const EMPTY_LOGS: LogEntry[] = [];
const MAX_VISIBLE_LOGS = 200;

const LogRow = memo(function LogRow({
  log,
}: {
  log: LogEntry;
}) {
  const config = LEVEL_CONFIG[log.level];
  const Icon = config.icon;
  
  return (
    <motion.div 
      layout
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      className="p-3 rounded-lg hover:bg-muted/50 transition-colors"
    >
      <div className="flex items-start gap-3">
        <div className={`flex-shrink-0 w-6 h-6 rounded-md flex items-center justify-center ${config.bgClass}`}>
          <Icon className={`w-3.5 h-3.5 ${config.className}`} />
        </div>
        <div className="flex-1 min-w-0 space-y-1">
          <p className="text-sm text-foreground">{log.message}</p>
          {log.file && (
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <FileText className="w-3 h-3" />
              <span className="truncate">{log.file}</span>
            </div>
          )}
          <p className="text-[10px] text-muted-foreground/70 font-mono">
            {log.timestampLabel}
          </p>
        </div>
      </div>
    </motion.div>
  );
});

export function LogsDrawer() {
  const { t } = useTranslation();
  const { logs, isOpen, closeDrawer, clearLogs, filter, setFilter } =
    useLogsStore(
      useShallow((s) => ({
        logs: s.isOpen ? s.logs : EMPTY_LOGS,
        isOpen: s.isOpen,
        closeDrawer: s.closeDrawer,
        clearLogs: s.clearLogs,
        filter: s.filter,
        setFilter: s.setFilter,
      })),
    );
  const [showAll, setShowAll] = useState(false);

  useEffect(() => {
    setShowAll(false);
  }, [filter, isOpen]);

  const filteredLogs = useMemo(() => {
    if (!isOpen) return EMPTY_LOGS;
    if (filter === "all") return logs;
    return logs.filter((l) => l.level === filter);
  }, [filter, logs, isOpen]);

  const visibleLogs = useMemo(() => {
    return showAll ? filteredLogs : filteredLogs.slice(0, MAX_VISIBLE_LOGS);
  }, [filteredLogs, showAll]);

  const hiddenCount = Math.max(0, filteredLogs.length - visibleLogs.length);
  const canToggle = filteredLogs.length > MAX_VISIBLE_LOGS;
  
  // Count by level
  const counts = useMemo(() => ({
    all: logs.length,
    info: logs.filter(l => l.level === 'info').length,
    success: logs.filter(l => l.level === 'success').length,
    warning: logs.filter(l => l.level === 'warning').length,
    error: logs.filter(l => l.level === 'error').length,
  }), [logs]);

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && closeDrawer()}>
      <DialogContent className="sm:max-w-[550px] max-h-[85vh] overflow-hidden flex flex-col p-0 gap-0">
        {/* Header */}
        <DialogHeader className="flex flex-row items-center justify-between p-5 pb-3 border-b border-border/50 flex-shrink-0">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
              <Faders className="w-4 h-4 text-primary" />
            </div>
            <DialogTitle className="text-title">{t("logs.title")}</DialogTitle>
          </div>
          <div className="flex items-center gap-2">
            <Select
              value={filter}
              onValueChange={(value) => setFilter(value as LogLevel | "all")}
            >
              <SelectTrigger className="w-32 h-8 text-xs">
                <SelectValue placeholder={t("logs.filter.all")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all" className="text-xs">
                  {t("logs.filter.all")} ({counts.all})
                </SelectItem>
                <SelectItem value="info" className="text-xs">
                  {t("logs.filter.info")} ({counts.info})
                </SelectItem>
                <SelectItem value="success" className="text-xs">
                  {t("logs.filter.success")} ({counts.success})
                </SelectItem>
                <SelectItem value="warning" className="text-xs">
                  {t("logs.filter.warning")} ({counts.warning})
                </SelectItem>
                <SelectItem value="error" className="text-xs">
                  {t("logs.filter.error")} ({counts.error})
                </SelectItem>
              </SelectContent>
            </Select>
            <Button 
              size="sm" 
              variant="ghost" 
              onClick={clearLogs}
              className="h-8 px-2 text-muted-foreground hover:text-destructive"
            >
              <Trash className="w-4 h-4" />
            </Button>
          </div>
        </DialogHeader>

        {/* Logs Content */}
        <ScrollArea className="flex-1 px-2">
          <div className="py-2 space-y-1">
            <AnimatePresence mode="popLayout">
              {!isOpen ? null : visibleLogs.length === 0 ? (
                <motion.div 
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="flex flex-col items-center justify-center py-12 text-center"
                >
                  <div className="w-12 h-12 rounded-2xl bg-muted flex items-center justify-center mb-3">
                    <Info className="w-6 h-6 text-muted-foreground" />
                  </div>
                  <p className="text-sm text-muted-foreground">
                    {t("logs.empty")}
                  </p>
                </motion.div>
              ) : (
                visibleLogs.map((log, index) => (
                  <motion.div
                    key={log.id}
                    initial={{ opacity: 0, x: -8 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 8 }}
                    transition={{ delay: index * 0.02 }}
                  >
                    <LogRow log={log} />
                  </motion.div>
                ))
              )}
            </AnimatePresence>
          </div>
        </ScrollArea>
        
        {/* Footer */}
        {canToggle && (
          <div className="flex justify-center border-t border-border/50 px-5 py-3">
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setShowAll((prev) => !prev)}
              className="text-xs"
            >
              {showAll
                ? t("logs.showLess")
                : t("logs.showMore", { count: hiddenCount })}
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
