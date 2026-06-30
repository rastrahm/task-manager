import { Platform } from 'react-native';

const DEV_HOST = Platform.OS === 'android' ? '10.0.2.2' : 'localhost';

export const API_BASE_URL = `http://${DEV_HOST}:5040`;

/** @deprecated Usar apiClient; se mantiene para compatibilidad interna. */
export const API_URL = `${API_BASE_URL}/tasks`;
