/**
 * Persistencia de la sesión JWT en `localStorage`.
 * @module sessionStore
 */

/** Clave de almacenamiento en el navegador. */
const SESSION_KEY = 'task-manager/session';

/**
 * Lee la sesión guardada en el navegador.
 * @returns {module:models.Session|null}
 */
export function loadSession() {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (!raw) {
      return null;
    }
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

/**
 * Guarda la sesión en `localStorage`.
 * @param {module:models.Session} session
 * @returns {Promise<void>}
 */
export async function saveSession(session) {
  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
}

/**
 * Elimina la sesión del almacenamiento local.
 * @returns {Promise<void>}
 */
export async function clearSession() {
  localStorage.removeItem(SESSION_KEY);
}
