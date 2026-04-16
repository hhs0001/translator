import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { PencilSimple, Trash } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { useSettingsStore } from '../../stores/settingsStore';
import { Template } from '../../types';

export function TemplateManager() {
  const { t } = useTranslation();
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
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">{t('settings.templates.title')}</h3>
        <Button variant="default" size="sm" onClick={openNewModal}>
          {t('settings.templates.newTemplate')}
        </Button>
      </div>

      {templates.length === 0 ? (
        <div className="text-center py-8 px-4 rounded-xl bg-muted/30 border border-dashed border-border">
          <p className="text-muted-foreground text-sm">
            {t('settings.templates.noTemplates')}
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          <AnimatePresence mode="popLayout">
            {templates.map((template) => (
              <motion.div
                key={template.id}
                layout
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.98 }}
                transition={{ type: 'spring', stiffness: 300, damping: 25 }}
                className="group flex items-start gap-3 p-4 rounded-xl bg-muted/30 hover:bg-muted/50 transition-colors"
              >
                {/* Content - takes available space */}
                <div className="flex-1 min-w-0 space-y-1">
                  <p className="font-medium text-sm text-foreground">
                    {template.name}
                  </p>
                  <p className="text-xs text-muted-foreground line-clamp-2 break-words">
                    {template.content}
                  </p>
                </div>

                {/* Actions - always visible, never shrink */}
                <div className="flex items-center gap-1 flex-shrink-0">
                  <Button 
                    size="sm" 
                    variant="ghost" 
                    onClick={() => openEditModal(template)}
                    className="h-8 w-8 p-0"
                  >
                    <PencilSimple className="w-4 h-4" />
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => openDeleteConfirm(template.id)}
                    className="h-8 w-8 p-0 text-destructive hover:text-destructive hover:bg-destructive/10"
                  >
                    <Trash className="w-4 h-4" />
                  </Button>
                </div>
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      )}

      {/* Edit/Create Modal */}
      <Dialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>
              {editingTemplate ? t('settings.templates.editTemplate') : t('settings.templates.newTemplateTitle')}
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label>{t('common.name')}</Label>
              <Input
                placeholder={t('settings.templates.nameExample')}
                value={name}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('settings.templates.promptContent')}</Label>
              <Textarea
                placeholder={t('settings.templates.promptPlaceholder')}
                value={content}
                onChange={(e) => setContent(e.target.value)}
                className="min-h-[200px] resize-none"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setIsModalOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button variant="default" onClick={handleSave} disabled={isSaving}>
              {isSaving ? t('common.saving') : t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Modal */}
      <Dialog open={isDeleteModalOpen} onOpenChange={setIsDeleteModalOpen}>
        <DialogContent className="sm:max-w-[400px]">
          <DialogHeader>
            <DialogTitle>{t('settings.templates.confirmDelete')}</DialogTitle>
          </DialogHeader>
          <p className="text-muted-foreground">
            {t('settings.templates.confirmDeleteMessage')}
          </p>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setIsDeleteModalOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button variant="destructive" onClick={handleDelete} disabled={isDeleting}>
              {isDeleting ? t('common.deleting') : t('common.delete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
