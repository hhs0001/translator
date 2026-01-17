import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useLogsStore } from '../../stores/logsStore';
import { LogLevel } from '../../types';

const LEVEL_BADGE_CLASSES: Record<LogLevel, string> = {
  info: 'bg-muted text-muted-foreground',
  success: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300',
  warning: 'bg-amber-500/15 text-amber-700 dark:text-amber-300',
  error: 'bg-destructive/15 text-destructive',
};

export function LogsDrawer() {
  const { logs, isOpen, closeDrawer, clearLogs, filter, setFilter } = useLogsStore();

  const filteredLogs = filter === 'all'
    ? logs
    : logs.filter((l) => l.level === filter);

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('pt-BR', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

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
            {filteredLogs.length === 0 ? (
              <p className="p-4 text-center text-muted-foreground">
                Nenhum log encontrado
              </p>
            ) : (
              filteredLogs.map((log) => (
                <div key={log.id} className="p-3 hover:bg-muted/50">
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
                        {formatTime(log.timestamp)}
                      </p>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </ScrollArea>
      </DialogContent>
    </Dialog>
  );
}
