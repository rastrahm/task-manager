/**
 * Persistencia de la sesión JWT en AsyncStorage.
 * @module sessionStore
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { Session } from './models';

/** Clave de almacenamiento para la sesión serializada. */
const SESSION_KEY = '@task-manager/session';

/**
 * Lee la sesión guardada en el dispositivo.
 * @returns {Promise<Session | null>} Sesión parseada o `null` si no existe o es inválida.
 */
export async function loadSession(): Promise<Session | null> {
  try {
    const raw = await AsyncStorage.getItem(SESSION_KEY);
    if (!raw) {
      return null;
    }
    return JSON.parse(raw) as Session;
  } catch {
    return null;
  }
}

/**
 * Guarda la sesión en AsyncStorage.
 * @param {Session} session - Sesión a persistir.
 * @returns {Promise<void>}
 */
export async function saveSession(session: Session): Promise<void> {
  await AsyncStorage.setItem(SESSION_KEY, JSON.stringify(session));
}

/**
 * Elimina la sesión del almacenamiento local.
 * @returns {Promise<void>}
 */
export async function clearSession(): Promise<void> {
  await AsyncStorage.removeItem(SESSION_KEY);
}
