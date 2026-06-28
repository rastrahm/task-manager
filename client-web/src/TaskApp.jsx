import React, { useState, useEffect, useCallback } from 'react';

const API_URL = 'http://localhost:5040/tasks';

function TaskItem({ task, onToggle, onAddSubtask }) {
  const [subtaskTitle, setSubtaskTitle] = useState('');
  const [showSubtaskForm, setShowSubtaskForm] = useState(false);

  const handleAddSubtask = async (e) => {
    e.preventDefault();
    if (!subtaskTitle.trim()) return;
    await onAddSubtask(task.id, subtaskTitle.trim());
    setSubtaskTitle('');
    setShowSubtaskForm(false);
  };

  return (
    <li style={{ margin: '10px 0', listStyle: 'none' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <input
          type="checkbox"
          checked={task.completed}
          onChange={() => onToggle(task.id)}
        />
        <span style={{ textDecoration: task.completed ? 'line-through' : 'none' }}>
          {task.title}
        </span>
        <button
          type="button"
          onClick={() => setShowSubtaskForm((v) => !v)}
          title="Agregar subtarea"
          aria-label={`Agregar subtarea a ${task.title}`}
          style={{
            marginLeft: 'auto',
            fontSize: '14px',
            padding: '2px 8px',
            cursor: 'pointer',
          }}
        >
          + Subtarea
        </button>
      </div>
      {showSubtaskForm && (
        <form
          onSubmit={handleAddSubtask}
          style={{ display: 'flex', gap: '8px', marginTop: '8px', marginLeft: '24px' }}
        >
          <input
            value={subtaskTitle}
            onChange={(e) => setSubtaskTitle(e.target.value)}
            placeholder="Nueva subtarea..."
            autoFocus
          />
          <button type="submit">Agregar</button>
        </form>
      )}
      {task.children?.length > 0 && (
        <ul style={{ margin: '8px 0 0 0', paddingLeft: '24px' }}>
          {task.children.map((child) => (
            <TaskItem
              key={child.id}
              task={child}
              onToggle={onToggle}
              onAddSubtask={onAddSubtask}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

export default function TaskApp() {
  const [tasks, setTasks] = useState([]);
  const [newTitle, setNewTitle] = useState('');
  const [refreshing, setRefreshing] = useState(false);

  const fetchTasks = useCallback(async () => {
    try {
      const res = await fetch(API_URL);
      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
      const data = await res.json();
      setTasks(data);
    } catch (error) {
      console.error('Error fetching tasks:', error);
      alert('No se pudieron cargar las tareas. Asegúrate de que el backend esté corriendo.');
    }
  }, []);

  const onRefresh = useCallback(async () => {
    setRefreshing(true);
    await fetchTasks();
    setRefreshing(false);
  }, [fetchTasks]);

  const createRootTask = async (e) => {
    e.preventDefault();
    if (!newTitle.trim()) return;

    await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title: newTitle.trim() }),
    });

    setNewTitle('');
    await fetchTasks();
  };

  const addSubtask = async (parentId, title) => {
    await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title, parent_id: parentId }),
    });
    await fetchTasks();
  };

  const toggleTask = async (id) => {
    await fetch(`${API_URL}/${id}/toggle`, { method: 'POST' });
    await fetchTasks();
  };

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  return (
    <div style={{ padding: '20px', fontFamily: 'sans-serif', textAlign: 'left' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '20px' }}>
        <h2 style={{ margin: 0 }}>Gestor del Día a Día</h2>
        <button
          type="button"
          onClick={onRefresh}
          disabled={refreshing}
          title="Actualizar tareas"
          aria-label="Actualizar tareas"
          style={{
            width: '44px',
            height: '44px',
            borderRadius: '50%',
            border: '1px solid #ddd',
            backgroundColor: '#fff',
            cursor: refreshing ? 'wait' : 'pointer',
            fontSize: '22px',
            color: '#007bff',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            opacity: refreshing ? 0.7 : 1,
          }}
        >
          {refreshing ? '…' : '↻'}
        </button>
      </div>
      <form onSubmit={createRootTask} style={{ display: 'flex', gap: '8px', alignItems: 'center', marginBottom: '16px' }}>
        <input
          value={newTitle}
          onChange={(e) => setNewTitle(e.target.value)}
          placeholder="Nueva tarea..."
        />
        <button type="submit">Agregar</button>
      </form>
      <ul style={{ padding: 0 }}>
        {tasks.map((task) => (
          <TaskItem
            key={task.id}
            task={task}
            onToggle={toggleTask}
            onAddSubtask={addSubtask}
          />
        ))}
      </ul>
    </div>
  );
}
