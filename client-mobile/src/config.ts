/**
 * Configuración de red del cliente móvil.
 * @module config
 */

import { Platform } from 'react-native';

/** Host del backend en desarrollo según la plataforma (`10.0.2.2` en Android emulador). */
const DEV_HOST = Platform.OS === 'android' ? '10.0.2.2' : 'localhost';

/**
 * URL base del API REST sin barra final.
 * 
 * @example
 * // Android emulador
 * 'http://10.0.2.2:5040'
 * @example
 * // iOS simulador
 * 'http://localhost:5040'
 */
export const API_BASE_URL = `http://${DEV_HOST}:5040`;

/**
 * URL del endpoint de tareas.
 * @deprecated Usar {@link apiClient} o funciones de {@link api}.
 * 
 */
export const API_URL = `${API_BASE_URL}/tasks`;
