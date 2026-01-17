import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { useState } from 'react';
import { useSettingsStore } from '../../stores/settingsStore';
import { Template } from '../../types';

export function TemplateManager() {
  const { templates, addTemplate, updateTemplate, deleteTemplate } = useSettingsStore();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [templateToDelete, setTemplateToDelete] = useState<string | null>(null);
  const [editingTemplate, setEditingTemplate] = useState<Template | null>(null);
  const [name, setName] = useState('');
  const [content, setContent] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

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

    setIsSaving(true);
    try {
      if (editingTemplate) {
        await updateTemplate(editingTemplate.id, name, content);
      } else {
        await addTemplate(name, content);
      }
      setIsModalOpen(false);
    } catch (error) {
      console.error('Failed to save template:', error);
    } finally {
      setIsSaving(false);
    }
  };

  const openDeleteConfirm = (id: string) => {
    setTemplateToDelete(id);
    setIsDeleteModalOpen(true);
  };

  const handleDelete = async () => {
    if (!templateToDelete) return;

    setIsDeleting(true);
    try {
      await deleteTemplate(templateToDelete);
      setIsDeleteModalOpen(false);
      setTemplateToDelete(null);
    } catch (error) {
      console.error('Failed to delete template:', error);
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Templates de Prompt</h3>
        <Button variant="default" size="sm" onClick={openNewModal}>
          + Novo Template
        </Button>
      </div>

      {templates.length === 0 ? (
        <p className="text-muted-foreground text-center py-4">
          Nenhum template criado ainda
        </p>
      ) : (
        <div className="space-y-2">
          {templates.map((template) => (
            <div
              key={template.id}
              className="flex items-center justify-between p-3 bg-muted/50 rounded-lg"
            >
              <div>
                <p className="font-medium">{template.name}</p>
                <p className="text-sm text-muted-foreground truncate max-w-md">
                  {template.content}
                </p>
              </div>
              <div className="flex gap-2">
                <Button size="sm" variant="ghost" onClick={() => openEditModal(template)}>
                  Editar
                </Button>
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => openDeleteConfirm(template.id)}
                >
                  Excluir
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal de Edição/Criação */}
      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>{editingTemplate ? 'Editar Template' : 'Novo Template'}</DialogTitle>
          </DialogHeader>
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
              <Textarea
                placeholder="Digite o prompt..."
                value={content}
                onChange={(e) => setContent(e.target.value)}
                className="min-h-[200px]"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setIsModalOpen(false)}>
              Cancelar
            </Button>
            <Button variant="default" onClick={handleSave} disabled={isSaving}>
              {isSaving ? 'Salvando...' : 'Salvar'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal de Confirmação de Exclusão */}
      <Dialog open={isDeleteModalOpen} onOpenChange={setIsDeleteModalOpen}>
        <DialogContent className="sm:max-w-[400px]">
          <DialogHeader>
            <DialogTitle>Confirmar Exclusão</DialogTitle>
          </DialogHeader>
          <p>Tem certeza que deseja excluir este template? Esta ação não pode ser desfeita.</p>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setIsDeleteModalOpen(false)}>
              Cancelar
            </Button>
            <Button variant="destructive" onClick={handleDelete} disabled={isDeleting}>
              {isDeleting ? 'Excluindo...' : 'Excluir'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
