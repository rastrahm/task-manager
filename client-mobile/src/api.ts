import { API_URL } from './config';
import { Task } from './tasks';

export interface CreateTaskPayload {
  title: string;
  description?: string | null;
  metadata?: Record<string, unknown>;
  parent_id?: number | null;
}

export interface UpdateTaskPayload {
  title: string;
  description?: string | null;
  completed: boolean;
  metadata: Record<string, unknown>;
  parent_id?: number | null;
}

async function parseResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    throw new Error(`HTTP error! status: ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchTasks(): Promise<Task[]> {
  const res = await fetch(API_URL);
  return parseResponse<Task[]>(res);
}

export async function createTask(payload: CreateTaskPayload): Promise<Task> {
  const res = await fetch(API_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  return parseResponse<Task>(res);
}

export async function updateTask(id: number, payload: UpdateTaskPayload): Promise<Task> {
  const res = await fetch(`${API_URL}/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  return parseResponse<Task>(res);
}

export async function toggleTask(id: number): Promise<void> {
  const res = await fetch(`${API_URL}/${id}/toggle`, { method: 'POST' });
  await parseResponse<boolean>(res);
}
