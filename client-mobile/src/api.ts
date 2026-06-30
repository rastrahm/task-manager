import { apiClient } from './apiClient';
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

export async function fetchTasks(): Promise<Task[]> {
  return apiClient.get<Task[]>('/tasks');
}

export async function createTask(payload: CreateTaskPayload): Promise<Task> {
  return apiClient.post<Task>('/tasks', payload);
}

export async function updateTask(
  id: number,
  payload: UpdateTaskPayload,
): Promise<Task> {
  return apiClient.put<Task>(`/tasks/${id}`, payload);
}

export async function toggleTask(id: number): Promise<boolean> {
  return apiClient.postEmpty<boolean>(`/tasks/${id}/toggle`);
}
