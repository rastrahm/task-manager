export const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:5040';

/** @deprecated Usar apiClient; se mantiene para compatibilidad interna. */
export const API_URL = `${API_BASE_URL}/tasks`;
