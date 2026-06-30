import { apiClient } from './apiClient';

export async function fetchTasks() {
  return apiClient.get('/tasks');
}

export async function createTask(payload) {
  return apiClient.post('/tasks', payload);
}

export async function updateTask(id, payload) {
  return apiClient.put(`/tasks/${id}`, payload);
}

export async function toggleTask(id) {
  return apiClient.postEmpty(`/tasks/${id}/toggle`);
}
