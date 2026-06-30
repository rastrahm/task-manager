/**
 * Funciones de alto nivel para el API de tareas.
 * @module api
 */

import { apiClient } from './apiClient';

/**
 * Tarea en formato árbol (`GET /tasks`).
 * @typedef {object} Task
 * @property {number} id
 * @property {string} title
 * @property {string | null} description
 * @property {boolean} completed
 * @property {unknown} metadata - Ver {@link module:metadata}.
 * @property {number | null} [parent_id]
 * @property {Task[]} [children] - Subtareas anidadas.
 */

/**
 * Cuerpo de `POST /tasks`.
 * @typedef {object} CreateTaskPayload
 * @property {string} title
 * @property {string | null} [description]
 * @property {Record<string, unknown>} [metadata]
 * @property {number | null} [parent_id]
 */

/**
 * Cuerpo de `PUT /tasks/:id`.
 * @typedef {object} UpdateTaskPayload
 * @property {string} title
 * @property {string | null} [description]
 * @property {boolean} completed
 * @property {Record<string, unknown>} metadata
 * @property {number | null} [parent_id]
 */

/**
 * Obtiene las tareas raíz con subtareas en `children`.
 * @returns {Promise<Task[]>}
 */
export async function fetchTasks() {
  return apiClient.get('/tasks');
}

/**
 * Crea una tarea o subtarea.
 * @param {CreateTaskPayload} payload
 * @returns {Promise<Task>}
 */
export async function createTask(payload) {
  return apiClient.post('/tasks', payload);
}

/**
 * Actualiza todos los campos editables de una tarea.
 * @param {number} id
 * @param {UpdateTaskPayload} payload
 * @returns {Promise<Task>}
 */
export async function updateTask(id, payload) {
  return apiClient.put(`/tasks/${id}`, payload);
}

/**
 * Invierte el estado `completed` de una tarea.
 * @param {number} id
 * @returns {Promise<boolean>}
 */
export async function toggleTask(id) {
  return apiClient.postEmpty(`/tasks/${id}/toggle`);
}
