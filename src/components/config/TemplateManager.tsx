import { Card, Button, Modal, TextArea, Label, Input } from '@heroui/react';
import { useState } from 'react';
import { useSettingsStore } from '../../stores/settingsStore';
import { Template } from '../../types';

export function TemplateManager() {
  const { templates, addTemplate, updateTemplate, deleteTemplate } = useSettingsStore();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<Template | null>(null);
  const [name, setName] = useState('');
  const [content, setContent] = useState('');

  const openNewModal = () => {
    setEditingTemplate(null);
    setName('');
    setContent('');
    setIsModalOpen(true);
  };

  const openEditModal = (template: Template) => {
    setEditingTemplate(template);
    setName(template.name);
    setContent(template.content);
    setIsModalOpen(true);
  };

  const handleSave = async () => {
    if (!name.trim() || !content.trim()) return;

    try {
      if (editingTemplate) {
        await updateTemplate(editingTemplate.id, name, content);
      } else {
        await addTemplate(name, content);
      }
      setIsModalOpen(false);
    } catch (error) {
      console.error('Failed to save template:', error);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm('Tem certeza que deseja excluir este template?')) {
      await deleteTemplate(id);
    }
  };

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Templates de Prompt</h3>
        <Button variant="primary" size="sm" onPress={openNewModal}>
          + Novo Template
        </Button>
      </div>

      {templates.length === 0 ? (
        <p className="text-default-500 text-center py-4">
          Nenhum template criado ainda
        </p>
      ) : (
        <div className="space-y-2">
          {templates.map((template) => (
            <div
              key={template.id}
              className="flex items-center justify-between p-3 bg-default-100 rounded-lg"
            >
              <div>
                <p className="font-medium">{template.name}</p>
                <p className="text-sm text-default-500 truncate max-w-md">
                  {template.content}
                </p>
              </div>
              <div className="flex gap-2">
                <Button size="sm" variant="ghost" onPress={() => openEditModal(template)}>
                  Editar
                </Button>
                <Button
                  size="sm"
                  variant="danger"
                  onPress={() => handleDelete(template.id)}
                >
                  Excluir
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      <Modal isOpen={isModalOpen} onOpenChange={setIsModalOpen}>
        <Modal.Trigger>
          <Button variant="primary" className="hidden">Open</Button>
        </Modal.Trigger>
        <Modal.Backdrop />
        <Modal.Container>
          <Modal.Dialog>
            <Modal.Header>
              <Modal.Heading>{editingTemplate ? 'Editar Template' : 'Novo Template'}</Modal.Heading>
            </Modal.Header>
            <Modal.Body>
              <div className="space-y-4">
                <div className="flex flex-col gap-1">
                  <Label>Nome</Label>
                  <Input
                    placeholder="Ex: Anime Informal"
                    value={name}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
                  />
                </div>
                <div className="flex flex-col gap-1">
                  <Label>Conteúdo do Prompt</Label>
                  <TextArea
                    placeholder="Digite o prompt..."
                    value={content}
                    onChange={(e) => setContent(e.target.value)}
                  />
                </div>
              </div>
            </Modal.Body>
            <Modal.Footer>
              <Button variant="ghost" onPress={() => setIsModalOpen(false)}>
                Cancelar
              </Button>
              <Button variant="primary" onPress={handleSave}>
                Salvar
              </Button>
            </Modal.Footer>
            <Modal.CloseTrigger />
          </Modal.Dialog>
        </Modal.Container>
      </Modal>
    </Card>
  );
}
