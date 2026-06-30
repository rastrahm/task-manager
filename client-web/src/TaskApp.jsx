import React, { useState, useEffect, useCallback } from 'react';
import { createTask, fetchTasks, toggleTask, updateTask } from './api';
import { metadataSummary, metadataToJson, parseMetadata } from './metadata';
import { getStoredTheme, toggleTheme as switchTheme } from './theme';
import TaskFormModal from './TaskFormModal';
import './TaskApp.css';

function TaskTree({ task, depth, onToggle, onOpenForm }) {
  const meta = parseMetadata(task.metadata);
  const metaLabel = metadataSummary(meta);

  return (
    <>
      <div
        className={`task-row${depth > 0 ? ' task-row-nested' : ''}`}
        style={{ '--depth': depth }}
      >
        <input
          type="checkbox"
          className="task-checkbox"
          checked={task.completed}
          onChange={() => onToggle(task.id)}
          aria-label={task.completed ? `Marcar pendiente: ${task.title}` : `Marcar completada: ${task.title}`}
          title={task.completed ? 'Marcar como pendiente' : 'Marcar como completada'}
        />
        <div className="task-item-body">
          <div className={`task-item-title${task.completed ? ' completed' : ''}`}>
            {task.title}
          </div>
          {task.description ? (
            <p className="task-item-description">{task.description}</p>
          ) : null}
          {metaLabel ? <p className="task-item-meta">{metaLabel}</p> : null}
        </div>
        <div className="task-item-actions">
          <button
            type="button"
            className="btn-action btn-edit"
            onClick={() => onOpenForm({ kind: 'edit', task })}
            title="Editar título, descripción y detalles"
          >
            <span className="btn-action-icon" aria-hidden="true">✎</span>
            Editar
          </button>
          <button
            type="button"
            className="btn-action btn-subtask"
            onClick={() =>
              onOpenForm({
                kind: 'create',
                parentId: task.id,
                parentTitle: task.title,
              })
            }
            title="Crear una subtarea dentro de esta tarea"
          >
            <span className="btn-action-icon" aria-hidden="true">＋</span>
            Subtarea
          </button>
        </div>
      </div>
      {task.children?.map((child) => (
        <TaskTree
          key={child.id}
          task={child}
          depth={depth + 1}
          onToggle={onToggle}
          onOpenForm={onOpenForm}
        />
      ))}
    </>
  );
}

export default function TaskApp() {
  const [tasks, setTasks] = useState([]);
  const [refreshing, setRefreshing] = useState(false);
  const [formMode, setFormMode] = useState(null);
  const [themeMode, setThemeMode] = useState(getStoredTheme);

  const loadTasks = useCallback(async () => {
    try {
      const data = await fetchTasks();
      setTasks(data);
    } catch (error) {
      console.error('Error fetching tasks:', error);
      alert('No se pudieron cargar las tareas. Asegúrate de que el backend esté corriendo.');
    }
  }, []);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await loadTasks();
    setRefreshing(false);
  }, [loadTasks]);

  const handleFormSubmit = async (values) => {
    try {
      if (values.task) {
        await updateTask(values.task.id, {
          title: values.title,
          description: values.description ?? null,
          completed: values.task.completed,
          metadata: metadataToJson(values.metadata),
          parent_id: values.parentId ?? null,
        });
      } else {
        await createTask({
          title: values.title,
          description: values.description ?? null,
          metadata: metadataToJson(values.metadata),
          parent_id: values.parentId ?? null,
        });
      }
      await loadTasks();
    } catch (error) {
      console.error('Error saving task:', error);
      alert('No se pudo guardar la tarea.');
      throw error;
    }
  };

  const handleToggleTask = async (id) => {
    try {
      await toggleTask(id);
      await loadTasks();
    } catch (error) {
      console.error('Error toggling task:', error);
      alert('No se pudo actualizar el estado de la tarea.');
    }
  };

  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  return (
    <div className="task-app">
      <div className="task-app-header">
        <h2>Gestor del Día a Día</h2>
        <div className="task-app-header-actions">
          <button
            type="button"
            className="icon-button"
            onClick={() => setThemeMode(switchTheme())}
            title={themeMode === 'light' ? 'Activar tema oscuro' : 'Activar tema claro'}
            aria-label={themeMode === 'light' ? 'Activar tema oscuro' : 'Activar tema claro'}
          >
            <span className="theme-icon" aria-hidden="true">
              {themeMode === 'light' ? '🌙' : '☀️'}
            </span>
          </button>
          <button
            type="button"
            className="icon-button"
            onClick={onRefresh}
            disabled={refreshing}
            title="Actualizar lista de tareas"
            aria-label="Actualizar tareas"
          >
            {refreshing ? '…' : '↻'}
          </button>
        </div>
      </div>

      <p className="task-app-hint">
        Marca el casillero para completar una tarea. Usa <strong>Editar</strong> para cambiar detalles
        o <strong>Subtarea</strong> para agregar pasos dentro de otra.
      </p>

      <div className="task-toolbar">
        <button
          type="button"
          className="btn-primary"
          onClick={() => setFormMode({ kind: 'create' })}
        >
          ＋ Nueva tarea
        </button>
      </div>

      <ul className="task-list">
        {tasks.length === 0 ? (
          <li className="task-empty">
            No hay tareas todavía. Pulsa <strong>Nueva tarea</strong> para empezar.
          </li>
        ) : (
          tasks.map((task) => (
            <li key={task.id} className="task-group">
              <TaskTree
                task={task}
                depth={0}
                onToggle={handleToggleTask}
                onOpenForm={setFormMode}
              />
            </li>
          ))
        )}
      </ul>

      <TaskFormModal
        mode={formMode}
        onClose={() => setFormMode(null)}
        onSubmit={handleFormSubmit}
      />
    </div>
  );
}
