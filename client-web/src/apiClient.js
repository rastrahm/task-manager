/**
 * Cliente HTTP con JWT, renovación automática y persistencia de sesión.
 * @module apiClient
 */

import { API_BASE_URL } from './config';
import { isAccessExpiringSoon, sessionFromAuth } from './models';
import { clearSession, loadSession, saveSession } from './sessionStore';

/** Timeout de red para peticiones al backend (ms). */
const HTTP_TIMEOUT_MS = 15_000;

/**
 * Error lanzado cuando el access token ya no es válido y no se pudo renovar.
 */
export class SessionExpiredError extends Error {
  /**
   * @param {string} [message='Sesión no válida; vuelve a iniciar sesión']
   */
  constructor(message = 'Sesión no válida; vuelve a iniciar sesión') {
    super(message);
    this.name = 'SessionExpiredError';
  }
}

/**
 * Cliente del API REST con sesión JWT en memoria y en `localStorage`.
 * Renueva el access token antes de que expire y reintenta una vez ante `401`.
 */
class ApiClient {
  /** @type {module:models.Session|null} */
  session = null;

  /** @type {Promise<void> | null} */
  refreshInFlight = null;

  /** @returns {module:models.User|null} */
  get currentUser() {
    return this.session?.user ?? null;
  }

  /** @returns {string | null} */
  get username() {
    return this.session?.user.username ?? null;
  }

  /** @returns {boolean} */
  get isAdmin() {
    return this.session?.user.is_admin ?? false;
  }

  /**
   * Restaura la sesión desde `localStorage`.
   * @returns {Promise<module:models.Session|null>}
   */
  async restoreSession() {
    const stored = loadSession();
    if (stored) {
      this.session = stored;
    }
    return stored;
  }

  /**
   * Renueva los tokens si el access token expira en menos de 60 segundos.
   * @returns {Promise<void>}
   */
  async refreshSessionIfNeeded() {
    if (!this.session || !isAccessExpiringSoon(this.session)) {
      return;
    }
    await this.refreshTokens();
  }

  /**
   * Autentica con usuario y contraseña; persiste la sesión.
   * @param {string} username
   * @param {string} password
   * @returns {Promise<void>}
   * @throws {Error} Si las credenciales son incorrectas o hay error de red.
   */
  async login(username, password) {
    const response = await this.fetchWithTimeout(`${API_BASE_URL}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username: username.trim(),
        password,
      }),
    });

    if (response.status === 401) {
      throw new Error('Usuario o contraseña incorrectos');
    }
    if (!response.ok) {
      throw new Error(`Error de autenticación: ${response.status}`);
    }

    const auth = await response.json();
    await this.setSession(sessionFromAuth(auth));
  }

  /**
   * Revoca el refresh token en el servidor y borra la sesión local.
   * @returns {Promise<void>}
   */
  async logout() {
    const refreshToken = this.session?.refresh_token;
    if (refreshToken) {
      try {
        await fetch(`${API_BASE_URL}/auth/logout`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ refresh_token: refreshToken }),
        });
      } catch {
        // Ignorar errores de red al cerrar sesión.
      }
    }
    this.session = null;
    await clearSession();
  }

  /** Elimina la sesión de memoria y de `localStorage` sin llamar al servidor. */
  clearLocalSession() {
    this.session = null;
    void clearSession();
  }

  /**
   * Petición GET autenticada.
   * @template T
   * @param {string} path - Ruta relativa (ej. `/tasks`).
   * @returns {Promise<T>}
   */
  async get(path) {
    return this.requestJson('GET', path);
  }

  /**
   * Petición POST autenticada con cuerpo JSON.
   * @template T
   * @param {string} path
   * @param {unknown} body
   * @returns {Promise<T>}
   */
  async post(path, body) {
    return this.requestJson('POST', path, body);
  }

  /**
   * Petición POST autenticada sin cuerpo.
   * @template T
   * @param {string} path
   * @returns {Promise<T>}
   */
  async postEmpty(path) {
    return this.requestJson('POST', path);
  }

  /**
   * Petición PUT autenticada con cuerpo JSON.
   * @template T
   * @param {string} path
   * @param {unknown} body
   * @returns {Promise<T>}
   */
  async put(path, body) {
    return this.requestJson('PUT', path, body);
  }

  /**
   * Petición DELETE autenticada.
   * @param {string} path
   * @returns {Promise<void>}
   */
  async delete(path) {
    const response = await this.sendWithAuth('DELETE', path);
    if (!response.ok) {
      throw new Error(`El servidor respondió con error: ${response.status}`);
    }
  }

  /**
   * Lista todos los usuarios (requiere rol administrador).
   * @returns {Promise<module:models.User[]>}
   */
  async listUsers() {
    return this.get('/users');
  }

  /**
   * Crea un usuario (requiere rol administrador).
   * @param {module:models.CreateUserRequest} payload
   * @returns {Promise<module:models.User>}
   */
  async createUser(payload) {
    return this.post('/users', payload);
  }

  /**
   * Actualiza un usuario por id (requiere rol administrador).
   * @param {number} id
   * @param {module:models.UpdateUserRequest} payload
   * @returns {Promise<module:models.User>}
   */
  async updateUser(id, payload) {
    return this.put(`/users/${id}`, payload);
  }

  /**
   * Elimina un usuario por id (requiere rol administrador).
   * @param {number} id
   * @returns {Promise<void>}
   */
  async deleteUser(id) {
    await this.delete(`/users/${id}`);
  }

  /**
   * @param {module:models.Session} session
   * @private
   */
  async setSession(session) {
    this.session = session;
    await saveSession(session);
  }

  /**
   * @param {'GET' | 'POST' | 'PUT' | 'DELETE'} method
   * @param {string} path
   * @param {unknown} [body]
   * @returns {Promise<unknown>}
   * @private
   */
  async requestJson(method, path, body) {
    const response = await this.sendWithAuth(method, path, body);
    if (!response.ok) {
      throw new Error(`El servidor respondió con error: ${response.status}`);
    }
    return response.json();
  }

  /**
   * @param {'GET' | 'POST' | 'PUT' | 'DELETE'} method
   * @param {string} path
   * @param {unknown} [body]
   * @returns {Promise<Response>}
   * @private
   */
  async sendWithAuth(method, path, body) {
    if (this.session && isAccessExpiringSoon(this.session)) {
      await this.refreshTokens();
    }

    let response = await this.sendOnce(method, path, body);
    if (response.status === 401) {
      await this.refreshTokens();
      response = await this.sendOnce(method, path, body);
      if (response.status === 401) {
        this.clearLocalSession();
        throw new SessionExpiredError();
      }
    }
    return response;
  }

  /** @private */
  async refreshTokens() {
    if (this.refreshInFlight) {
      await this.refreshInFlight;
      return;
    }

    this.refreshInFlight = this.doRefreshTokens();
    try {
      await this.refreshInFlight;
    } finally {
      this.refreshInFlight = null;
    }
  }

  /** @private */
  async doRefreshTokens() {
    const refreshToken = this.session?.refresh_token;
    if (!refreshToken) {
      this.clearLocalSession();
      throw new SessionExpiredError();
    }

    const response = await this.fetchWithTimeout(`${API_BASE_URL}/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });

    if (!response.ok) {
      this.clearLocalSession();
      throw new SessionExpiredError('La sesión expiró; vuelve a iniciar sesión');
    }

    const auth = await response.json();
    await this.setSession(sessionFromAuth(auth));
  }

  /**
   * @param {'GET' | 'POST' | 'PUT' | 'DELETE'} method
   * @param {string} path
   * @param {unknown} [body]
   * @returns {Promise<Response>}
   * @private
   */
  async sendOnce(method, path, body) {
    const accessToken = this.session?.access_token;
    if (!accessToken) {
      throw new SessionExpiredError('No hay sesión activa');
    }

    const headers = {
      Authorization: `Bearer ${accessToken}`,
    };
    const init = { method, headers };

    if (body !== undefined) {
      headers['Content-Type'] = 'application/json';
      init.body = JSON.stringify(body);
    }

    return this.fetchWithTimeout(`${API_BASE_URL}${path}`, init);
  }

  /**
   * @param {string} url
   * @param {RequestInit} [init]
   * @returns {Promise<Response>}
   * @private
   */
  async fetchWithTimeout(url, init = {}) {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), HTTP_TIMEOUT_MS);

    try {
      return await fetch(url, { ...init, signal: controller.signal });
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error(
          `No se pudo conectar con el backend (${HTTP_TIMEOUT_MS / 1000}s máx.)`,
        );
      }
      throw new Error(
        `No se pudo conectar con el backend: ${
          error instanceof Error ? error.message : 'error de red'
        }`,
      );
    } finally {
      clearTimeout(timeout);
    }
  }
}

/** Instancia singleton del cliente API usada en toda la aplicación. */
export const apiClient = new ApiClient();
