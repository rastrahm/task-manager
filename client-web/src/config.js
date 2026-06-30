/**
 * Configuración de red del cliente web (Vite).
 * @module config
 */

/**
 * URL base del API REST sin barra final.
 * Definible con la variable de entorno `VITE_API_BASE_URL`.
 * @type {string}
 * @example
 * // Por defecto en desarrollo
 * 'http://localhost:5040'
 */
export const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:5040';

/**
 * URL del endpoint de tareas.
 * @deprecated Usar {@link apiClient} o funciones de {@link module:api}.
 * @type {string}
 */
export const API_URL = `${API_BASE_URL}/tasks`;
