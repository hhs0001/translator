import { Modal, Button, Chip, Select, Label, ListBox } from '@heroui/react';
import { useLogsStore } from '../../stores/logsStore';
import { LogLevel } from '../../types';

const LEVEL_COLORS: Record<LogLevel, 'default' | 'success' | 'warning' | 'danger'> = {
  info: 'default',
  success: 'success',
  warning: 'warning',
  error: 'danger',
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
      second: '2-digit'
    });
  };

  return (
    <Modal isOpen={isOpen} onOpenChange={(open) => !open && closeDrawer()}>
      <Modal.Backdrop>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-[500px]">
            <Modal.CloseTrigger />
            <Modal.Header className="flex items-center justify-between">
              <Modal.Heading>Logs</Modal.Heading>
              <div className="flex gap-2">
                <Select
                  aria-label="Filtrar logs"
                  selectedKey={filter}
                  onSelectionChange={(key) => key && setFilter(key as LogLevel | 'all')}
                  className="w-28"
                  placeholder="Todos"
                >
                  <Label className="sr-only">Filtrar</Label>
                  <Select.Trigger>
                    <Select.Value />
                    <Select.Indicator />
                  </Select.Trigger>
                  <Select.Popover>
                    <ListBox>
                      <ListBox.Item id="all" textValue="Todos">
                        Todos
                        <ListBox.ItemIndicator />
                      </ListBox.Item>
                      <ListBox.Item id="info" textValue="Info">
                        Info
                        <ListBox.ItemIndicator />
                      </ListBox.Item>
                      <ListBox.Item id="success" textValue="Sucesso">
                        Sucesso
                        <ListBox.ItemIndicator />
                      </ListBox.Item>
                      <ListBox.Item id="warning" textValue="Warning">
                        Warning
                        <ListBox.ItemIndicator />
                      </ListBox.Item>
                      <ListBox.Item id="error" textValue="Erro">
                        Erro
                        <ListBox.ItemIndicator />
                      </ListBox.Item>
                    </ListBox>
                  </Select.Popover>
                </Select>
                <Button size="sm" variant="ghost" onPress={clearLogs}>
                  Limpar
                </Button>
              </div>
            </Modal.Header>

            <Modal.Body className="p-0">
              <div className="p-0">
                <div className="divide-y divide-default-200 dark:divide-default-100">
                  {filteredLogs.length === 0 ? (
                    <p className="p-4 text-center text-default-500">
                      Nenhum log encontrado
                    </p>
                  ) : (
                    filteredLogs.map((log) => (
                      <div key={log.id} className="p-3 hover:bg-default-100">
                        <div className="flex items-start gap-2">
                          <Chip size="sm" color={LEVEL_COLORS[log.level]} variant="soft">
                            {log.level}
                          </Chip>
                          <div className="flex-1 min-w-0">
                            <p className="text-sm break-words">{log.message}</p>
                            {log.file && (
                              <p className="text-xs text-default-400 mt-1">
                                Arquivo: {log.file}
                              </p>
                            )}
                            <p className="text-xs text-default-300 mt-1">
                              {formatTime(log.timestamp)}
                            </p>
                          </div>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </Modal.Body>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>
    </Modal>
  );
}
