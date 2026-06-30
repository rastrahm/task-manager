/**
 * Funciones de alto nivel para el API de tareas.
 * @module api
 */

import { apiClient } from './apiClient';
import { Task } from './tasks';

/** Cuerpo de `POST /tasks`. */
export interface CreateTaskPayload {
  title: string;
  description?: string | null;
  metadata?: Record<string, unknown>;
  /** Id del padre si se crea una subtarea. */
  parent_id?: number | null;
}

/** Cuerpo de `PUT /tasks/:id`. */
export interface UpdateTaskPayload {
  title: string;
  description?: string | null;
  completed: boolean;
  metadata: Record<string, unknown>;
  parent_id?: number | null;
}

/**
 * Obtiene las tareas raíz con subtareas anidadas en `children`.
 * @returns {Promise<Task[]>} Árbol de tareas del usuario autenticado.
 */
export async function fetchTasks(): Promise<Task[]> {
  return apiClient.get<Task[]>('/tasks');
}

/**
 * Crea una tarea o subtarea.
 * @param {CreateTaskPayload} payload - Datos de la nueva tarea.
 * @returns {Promise<Task>} Tarea creada (formato plano del backend).
 */
export async function createTask(payload: CreateTaskPayload): Promise<Task> {
  return apiClient.post<Task>('/tasks', payload);
}

/**
 * Actualiza todos los campos editables de una tarea.
 * @param {number} id - Id de la tarea.
 * @param {UpdateTaskPayload} payload - Nuevos valores.
 * @returns {Promise<Task>} Tarea actualizada.
 */
export async function updateTask(
  id: number,
  payload: UpdateTaskPayload,
): Promise<Task> {
  return apiClient.put<Task>(`/tasks/${id}`, payload);
}

/**
 * Invierte el estado `completed` de una tarea.
 * @param {number} id - Id de la tarea.
 * @returns {Promise<boolean>} `true` si el toggle fue exitoso.
 */
export async function toggleTask(id: number): Promise<boolean> {
  return apiClient.postEmpty<boolean>(`/tasks/${id}/toggle`);
}
