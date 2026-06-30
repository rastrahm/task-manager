import React, { useEffect, useState } from 'react';
import {
  metadataToJson,
  parseMetadata,
  parseTagsInput,
  PRIORITIES,
  tagsToInput,
} from './metadata';

export default function TaskFormModal({ mode, onClose, onSubmit }) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState('');
  const [dueDate, setDueDate] = useState('');
  const [tags, setTags] = useState('');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!mode) {
      return;
    }

    if (mode.kind === 'edit') {
      const task = mode.task;
      setTitle(task.title);
      setDescription(task.description ?? '');
      const meta = parseMetadata(task.metadata);
      setPriority(meta.priority ?? '');
      setDueDate(meta.due_date ?? '');
      setTags(tagsToInput(meta.tags));
      return;
    }

    setTitle('');
    setDescription('');
    setPriority('');
    setDueDate('');
    setTags('');
  }, [mode]);

  if (!mode) {
    return null;
  }

  const modalTitle =
    mode.kind === 'edit'
      ? 'Editar tarea'
      : mode.parentId
        ? 'Nueva subtarea'
        : 'Nueva tarea';

  const handleSubmit = async (event) => {
    event.preventDefault();
    if (!title.trim()) {
      return;
    }

    const metadata = {
      priority: priority || undefined,
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
    <div className="task-modal-backdrop" onClick={onClose} role="presentation">
      <div
        className="task-modal-card"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="task-form-title"
      >
        <h3 id="task-form-title">{modalTitle}</h3>
        {mode.kind === 'create' && mode.parentTitle ? (
          <p className="task-modal-subtitle">De: {mode.parentTitle}</p>
        ) : null}

        <form onSubmit={handleSubmit} className="task-form">
          <label className="task-form-field">
            <span>Título *</span>
            <input
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="Título de la tarea"
              required
            />
          </label>

          <label className="task-form-field">
            <span>Descripción</span>
            <textarea
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="Detalles de la tarea..."
              rows={4}
            />
          </label>

          <fieldset className="task-form-field">
            <legend>Prioridad</legend>
            <div className="priority-row">
              <label className="priority-chip">
                <input
                  type="radio"
                  name="priority"
                  value=""
                  checked={!priority}
                  onChange={() => setPriority('')}
                />
                Ninguna
              </label>
              {PRIORITIES.map((item) => (
                <label key={item} className="priority-chip">
                  <input
                    type="radio"
                    name="priority"
                    value={item}
                    checked={priority === item}
                    onChange={() => setPriority(item)}
                  />
                  {item}
                </label>
              ))}
            </div>
          </fieldset>

          <label className="task-form-field">
            <span>Fecha límite (AAAA-MM-DD)</span>
            <input
              value={dueDate}
              onChange={(event) => setDueDate(event.target.value)}
              placeholder="2026-03-20"
            />
          </label>

          <label className="task-form-field">
            <span>Etiquetas (separadas por coma)</span>
            <input
              value={tags}
              onChange={(event) => setTags(event.target.value)}
              placeholder="casa, urgente, trabajo"
            />
          </label>

          <div className="task-form-actions">
            <button type="button" className="btn-secondary" onClick={onClose} disabled={saving}>
              Cancelar
            </button>
            <button type="submit" className="btn-primary" disabled={saving || !title.trim()}>
              {saving ? 'Guardando...' : mode.kind === 'edit' ? 'Guardar' : 'Crear'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export { metadataToJson };
