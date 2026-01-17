import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useLogsStore } from '../../stores/logsStore';
import { LogEntry, LogLevel } from '../../types';
import { memo, useEffect, useMemo, useState } from 'react';
import { useShallow } from 'zustand/shallow';

const LEVEL_BADGE_CLASSES: Record<LogLevel, string> = {
  info: 'bg-muted text-muted-foreground',
  success: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300',
  warning: 'bg-amber-500/15 text-amber-700 dark:text-amber-300',
  error: 'bg-destructive/15 text-destructive',
};

const EMPTY_LOGS: LogEntry[] = [];
const MAX_VISIBLE_LOGS = 200;

const LogRow = memo(function LogRow({ log }: { log: LogEntry }) {
  return (
    <div className="p-3 hover:bg-muted/50">
      <div className="flex items-start gap-2">
        <Badge
          variant="outline"
          className={`border-transparent ${LEVEL_BADGE_CLASSES[log.level]}`}
        >
          {log.level}
        </Badge>
        <div className="flex-1 min-w-0">
          <p className="text-sm break-words">{log.message}</p>
          {log.file && (
            <p className="text-xs text-muted-foreground mt-1">
              Arquivo: {log.file}
            </p>
          )}
          <p className="text-xs text-muted-foreground mt-1">
            {log.timestampLabel}
          </p>
        </div>
      </div>
    </div>
  );
});

export function LogsDrawer() {
  const { logs, isOpen, closeDrawer, clearLogs, filter, setFilter } = useLogsStore(
    useShallow((s) => ({
      logs: s.isOpen ? s.logs : EMPTY_LOGS,
      isOpen: s.isOpen,
      closeDrawer: s.closeDrawer,
      clearLogs: s.clearLogs,
      filter: s.filter,
      setFilter: s.setFilter,
    }))
  );
  const [showAll, setShowAll] = useState(false);

  useEffect(() => {
    setShowAll(false);
  }, [filter, isOpen]);

  const filteredLogs = useMemo(() => {
    if (!isOpen) return EMPTY_LOGS;
    if (filter === 'all') return logs;
    return logs.filter((l) => l.level === filter);
  }, [filter, logs, isOpen]);

  const visibleLogs = useMemo(() => {
    return showAll ? filteredLogs : filteredLogs.slice(0, MAX_VISIBLE_LOGS);
  }, [filteredLogs, showAll]);

  const hiddenCount = Math.max(0, filteredLogs.length - visibleLogs.length);
  const canToggle = filteredLogs.length > MAX_VISIBLE_LOGS;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && closeDrawer()}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader className="flex flex-row items-center justify-between">
          <DialogTitle>Logs</DialogTitle>
          <div className="flex gap-2">
            <Select
              value={filter}
              onValueChange={(value) => setFilter(value as LogLevel | 'all')}
            >
              <SelectTrigger className="w-28">
                <SelectValue placeholder="Todos" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">Todos</SelectItem>
                <SelectItem value="info">Info</SelectItem>
                <SelectItem value="success">Sucesso</SelectItem>
                <SelectItem value="warning">Warning</SelectItem>
                <SelectItem value="error">Erro</SelectItem>
              </SelectContent>
            </Select>
            <Button size="sm" variant="ghost" onClick={clearLogs}>
              Limpar
            </Button>
          </div>
        </DialogHeader>

        <ScrollArea className="-mx-6 max-h-[60vh] border-t border-border">
          <div className="divide-y divide-border">
            {!isOpen ? null : (
              visibleLogs.length === 0 ? (
                <p className="p-4 text-center text-muted-foreground">
                  Nenhum log encontrado
                </p>
              ) : (
                visibleLogs.map((log) => (
                  <LogRow key={log.id} log={log} />
                ))
              )
            )}
          </div>
        </ScrollArea>
        {canToggle && (
          <div className="flex justify-center border-t border-border px-6 py-2">
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setShowAll((prev) => !prev)}
            >
              {showAll ? 'Mostrar menos' : `Mostrar mais (${hiddenCount})`}
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
