import React, { useState, useEffect } from 'react';

export default function TaskApp() {
  const [tasks, setTasks] = useState([]);
  const [newTitle, setNewTitle] = useState('');
  const API_URL = "http://localhost:8080/tasks";

  const fetchTasks = async () => {
    const res = await fetch(API_URL);
    const data = await res.json();
    setTasks(data);
  };

  const createTask = async (e) => {
    e.preventDefault();
    if (!newTitle) return;
    await fetch(
      API_URL, 
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: newTitle })
      }
    );
    setNewTitle('');
    fetchTasks();
  };

  const toggleTask = async (id) => {
    await fetch(`${API_URL}/${id}/toggle`, { method: 'POST' });
    fetchTasks();
  };

  useEffect(() => { fetchTasks(); }, []);

  return (
    <div style={{ padding: '20px', fontFamily: 'sans-serif' }}>
      <h2>Gestor del Día a Día</h2>
      <form onSubmit={createTask}>
        <input 
          value={newTitle} 
          onChange={e => setNewTitle(e.target.value)} 
          placeholder="Nueva tarea..." 
        />
        <button type="submit">Agregar</button>
      </form>
      <ul>
        {tasks.map(task => (
          <li key={task.id} style={{ margin: '10px 0' }}>
            <input 
              type="checkbox" 
              checked={task.completed} 
              onChange={() => toggleTask(task.id)} 
            />
            <span style={{ textDecoration: task.completed ? 'line-through' : 'none', marginLeft: '10px' }}>
              {task.title}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}