const API_URL = 'http://localhost:5040/tasks';

async function parseResponse(res) {
  if (!res.ok) {
    throw new Error(`HTTP error! status: ${res.status}`);
  }
  return res.json();
}

export async function fetchTasks() {
  const res = await fetch(API_URL);
  return parseResponse(res);
}

export async function createTask(payload) {
  const res = await fetch(API_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export async function updateTask(id, payload) {
  const res = await fetch(`${API_URL}/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export async function toggleTask(id) {
  const res = await fetch(`${API_URL}/${id}/toggle`, { method: 'POST' });
  return parseResponse(res);
}
