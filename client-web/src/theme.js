/**
 * Tema claro/oscuro persistido en `localStorage` y aplicado al documento.
 * @module theme
 */

/** @type {string} */
const STORAGE_KEY = 'task-manager-theme';

/**
 * Modo de color de la interfaz.
 * @typedef {'light' | 'dark'} ThemeMode
 */

/**
 * Lee el tema guardado o devuelve `'light'` por defecto.
 * @returns {ThemeMode}
 */
export function getStoredTheme() {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === 'dark' ? 'dark' : 'light';
}

/**
 * Aplica el tema al elemento `<html>` y lo persiste.
 * @param {ThemeMode} mode
 */
export function applyTheme(mode) {
  document.documentElement.dataset.theme = mode;
  localStorage.setItem(STORAGE_KEY, mode);
}

/**
 * Inicializa el tema al arrancar la aplicación (llamar desde `main.jsx`).
 */
export function initTheme() {
  applyTheme(getStoredTheme());
}

/**
 * Alterna entre tema claro y oscuro.
 * @returns {ThemeMode} El modo activo tras el cambio.
 */
export function toggleTheme() {
  const next = getStoredTheme() === 'light' ? 'dark' : 'light';
  applyTheme(next);
  return next;
}
