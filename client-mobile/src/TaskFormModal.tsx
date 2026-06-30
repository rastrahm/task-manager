import React, { useEffect, useMemo, useState } from 'react';
import {
  Button,
  Modal,
  Pressable,
  ScrollView,
  Text,
  TextInput,
  TouchableOpacity,
  View,
} from 'react-native';
import { AppTheme, createStyles } from './theme';
import { Task } from './tasks';
import {
  metadataToJson,
  parseMetadata,
  parseTagsInput,
  PRIORITIES,
  Priority,
  tagsToInput,
  TaskMetadata,
} from './metadata';

export type TaskFormMode =
  | { kind: 'create'; parentId?: number | null; parentTitle?: string }
  | { kind: 'edit'; task: Task };

interface TaskFormModalProps {
  visible: boolean;
  mode: TaskFormMode | null;
  theme: AppTheme;
  onClose: () => void;
  onSubmit: (values: {
    title: string;
    description?: string | null;
    metadata: TaskMetadata;
    parentId?: number | null;
    task?: Task;
  }) => Promise<void>;
}

const emptyMetadata = (): TaskMetadata => ({});

export function TaskFormModal({
  visible,
  mode,
  theme,
  onClose,
  onSubmit,
}: TaskFormModalProps) {
  const styles = useMemo(() => createStyles(theme), [theme]);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<Priority | undefined>();
  const [dueDate, setDueDate] = useState('');
  const [tags, setTags] = useState('');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!visible || !mode) {
      return;
    }

    if (mode.kind === 'edit') {
      const task = mode.task;
      setTitle(task.title);
      setDescription(task.description ?? '');
      const meta = parseMetadata(task.metadata);
      setPriority(meta.priority);
      setDueDate(meta.due_date ?? '');
      setTags(tagsToInput(meta.tags));
      return;
    }

    setTitle('');
    setDescription('');
    setPriority(undefined);
    setDueDate('');
    setTags('');
  }, [visible, mode]);

  const modalTitle =
    mode?.kind === 'edit'
      ? 'Editar tarea'
      : mode?.parentId
        ? 'Nueva subtarea'
        : 'Nueva tarea';

  const handleSave = async () => {
    if (!mode || !title.trim()) {
      return;
    }

    const metadata: TaskMetadata = {
      priority,
      due_date: dueDate.trim() || undefined,
      tags: parseTagsInput(tags),
    };

    setSaving(true);
    try {
      await onSubmit({
        title: title.trim(),
        description: description.trim() ? description.trim() : null,
        metadata,
        parentId: mode.kind === 'create' ? mode.parentId ?? null : mode.task.parent_id ?? null,
        task: mode.kind === 'edit' ? mode.task : undefined,
      });
      onClose();
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal visible={visible} transparent animationType="slide" onRequestClose={onClose}>
      <Pressable style={styles.modalBackdrop} onPress={onClose}>
        <Pressable style={styles.formModalCard} onPress={() => {}}>
          <ScrollView keyboardShouldPersistTaps="handled">
            <Text style={styles.modalTitle}>{modalTitle}</Text>
            {mode?.kind === 'create' && mode.parentTitle ? (
              <Text style={styles.modalSubtitle}>De: {mode.parentTitle}</Text>
            ) : null}

            <Text style={styles.formLabel}>Título *</Text>
            <TextInput
              style={styles.modalInput}
              value={title}
              onChangeText={setTitle}
              placeholder="Título de la tarea"
              placeholderTextColor={theme.placeholder}
            />

            <Text style={styles.formLabel}>Descripción</Text>
            <TextInput
              style={[styles.modalInput, styles.formTextArea]}
              value={description}
              onChangeText={setDescription}
              placeholder="Detalles de la tarea..."
              placeholderTextColor={theme.placeholder}
              multiline
              textAlignVertical="top"
            />

            <Text style={styles.formLabel}>Prioridad</Text>
            <View style={styles.priorityRow}>
              <PriorityChip
                label="Ninguna"
                active={!priority}
                onPress={() => setPriority(undefined)}
                styles={styles}
                theme={theme}
              />
              {PRIORITIES.map(item => (
                <PriorityChip
                  key={item}
                  label={item}
                  active={priority === item}
                  onPress={() => setPriority(item)}
                  styles={styles}
                  theme={theme}
                />
              ))}
            </View>

            <Text style={styles.formLabel}>Fecha límite (AAAA-MM-DD)</Text>
            <TextInput
              style={styles.modalInput}
              value={dueDate}
              onChangeText={setDueDate}
              placeholder="2026-03-20"
              placeholderTextColor={theme.placeholder}
              autoCapitalize="none"
            />

            <Text style={styles.formLabel}>Etiquetas (separadas por coma)</Text>
            <TextInput
              style={styles.modalInput}
              value={tags}
              onChangeText={setTags}
              placeholder="casa, urgente, trabajo"
              placeholderTextColor={theme.placeholder}
            />

            <View style={styles.modalActions}>
              <Button title="Cancelar" onPress={onClose} color="#6c757d" disabled={saving} />
              <Button
                title={saving ? 'Guardando...' : mode?.kind === 'edit' ? 'Guardar' : 'Crear'}
                onPress={handleSave}
                color={theme.primary}
                disabled={saving || !title.trim()}
              />
            </View>
          </ScrollView>
        </Pressable>
      </Pressable>
    </Modal>
  );
}

function PriorityChip({
  label,
  active,
  onPress,
  styles,
  theme,
}: {
  label: string;
  active: boolean;
  onPress: () => void;
  styles: ReturnType<typeof createStyles>;
  theme: AppTheme;
}) {
  return (
    <TouchableOpacity
      onPress={onPress}
      style={[styles.priorityChip, active && styles.priorityChipActive]}
    >
      <Text style={[styles.priorityChipText, active && { color: theme.primary }]}>{label}</Text>
    </TouchableOpacity>
  );
}

export { emptyMetadata, metadataToJson };
