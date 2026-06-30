/**
 * Cliente HTTP con JWT, renovación automática y persistencia de sesión.
 * @module apiClient
 */

import { API_BASE_URL } from './config';
import {
  AuthResponse,
  CreateUserRequest,
  isAccessExpiringSoon,
  Session,
  sessionFromAuth,
  UpdateUserRequest,
  User,
} from './models';
import { clearSession, loadSession, saveSession } from './sessionStore';

/** Timeout de red para peticiones al backend (ms). */
const HTTP_TIMEOUT_MS = 15_000;

/**
 * Error lanzado cuando el access token ya no es válido y no se pudo renovar.
 */
export class SessionExpiredError extends Error {
  /**
   * @param {string} [message='Sesión no válida; vuelve a iniciar sesión'] - Mensaje para el usuario.
   */
  constructor(message = 'Sesión no válida; vuelve a iniciar sesión') {
    super(message);
    this.name = 'SessionExpiredError';
  }
}

type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

/**
 * Cliente del API REST con sesión JWT en memoria y en AsyncStorage.
 * Renueva el access token antes de que expire y reintenta una vez ante `401`.
 */
class ApiClient {
  private session: Session | null = null;
  private refreshInFlight: Promise<void> | null = null;

  /** Usuario de la sesión actual o `null` si no hay sesión. */
  get currentUser(): User | null {
    return this.session?.user ?? null;
  }

  /** Nombre de usuario de la sesión activa. */
  get username(): string | null {
    return this.session?.user.username ?? null;
  }

  /** Indica si el usuario autenticado es administrador. */
  get isAdmin(): boolean {
    return this.session?.user.is_admin ?? false;
  }

  /**
   * Restaura la sesión desde AsyncStorage.
   * @returns {Promise<Session | null>} Sesión cargada o `null`.
   */
  async restoreSession(): Promise<Session | null> {
    const stored = await loadSession();
    if (stored) {
      this.session = stored;
    }
    return stored;
  }

  /**
   * Renueva los tokens si el access token expira en menos de 60 segundos.
   * @returns {Promise<void>}
   */
  async refreshSessionIfNeeded(): Promise<void> {
    if (!this.session || !isAccessExpiringSoon(this.session)) {
      return;
    }
    await this.refreshTokens();
  }

  /**
   * Autentica con usuario y contraseña; persiste la sesión.
   * @param {string} username - Nombre de usuario.
   * @param {string} password - Contraseña en texto plano (solo en tránsito HTTPS).
   * @returns {Promise<void>}
   * @throws {Error} Si las credenciales son incorrectas o hay error de red.
   */
  async login(username: string, password: string): Promise<void> {
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

    const auth = (await response.json()) as AuthResponse;
    await this.setSession(sessionFromAuth(auth));
  }

  /**
   * Revoca el refresh token en el servidor y borra la sesión local.
   * @returns {Promise<void>}
   */
  async logout(): Promise<void> {
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

  /** Elimina la sesión de memoria y de AsyncStorage sin llamar al servidor. */
  clearLocalSession(): void {
    this.session = null;
    void clearSession();
  }

  /**
   * Petición GET autenticada.
   * @template T
   * @param {string} path - Ruta relativa (ej. `/tasks`).
   * @returns {Promise<T>} Cuerpo JSON deserializado.
   */
  async get<T>(path: string): Promise<T> {
    return this.requestJson<T>('GET', path);
  }

  /**
   * Petición POST autenticada con cuerpo JSON.
   * @template T
   * @param {string} path - Ruta relativa.
   * @param {unknown} body - Cuerpo serializable a JSON.
   * @returns {Promise<T>}
   */
  async post<T>(path: string, body: unknown): Promise<T> {
    return this.requestJson<T>('POST', path, body);
  }

  /**
   * Petición POST autenticada sin cuerpo.
   * @template T
   * @param {string} path - Ruta relativa.
   * @returns {Promise<T>}
   */
  async postEmpty<T>(path: string): Promise<T> {
    return this.requestJson<T>('POST', path);
  }

  /**
   * Petición PUT autenticada con cuerpo JSON.
   * @template T
   * @param {string} path - Ruta relativa.
   * @param {unknown} body - Cuerpo serializable a JSON.
   * @returns {Promise<T>}
   */
  async put<T>(path: string, body: unknown): Promise<T> {
    return this.requestJson<T>('PUT', path, body);
  }

  /**
   * Petición DELETE autenticada.
   * @param {string} path - Ruta relativa.
   * @returns {Promise<void>}
   */
  async delete(path: string): Promise<void> {
    const response = await this.sendWithAuth('DELETE', path);
    if (!response.ok) {
      throw new Error(`El servidor respondió con error: ${response.status}`);
    }
  }

  /**
   * Lista todos los usuarios (requiere rol administrador).
   * @returns {Promise<User[]>}
   */
  async listUsers(): Promise<User[]> {
    return this.get<User[]>('/users');
  }

  /**
   * Crea un usuario (requiere rol administrador).
   * @param {CreateUserRequest} payload - Datos del nuevo usuario.
   * @returns {Promise<User>}
   */
  async createUser(payload: CreateUserRequest): Promise<User> {
    return this.post<User>('/users', payload);
  }

  /**
   * Actualiza un usuario por id (requiere rol administrador).
   * @param {number} id - Id del usuario.
   * @param {UpdateUserRequest} payload - Campos a modificar.
   * @returns {Promise<User>}
   */
  async updateUser(id: number, payload: UpdateUserRequest): Promise<User> {
    return this.put<User>(`/users/${id}`, payload);
  }

  /**
   * Elimina un usuario por id (requiere rol administrador).
   * @param {number} id - Id del usuario.
   * @returns {Promise<void>}
   */
  async deleteUser(id: number): Promise<void> {
    await this.delete(`/users/${id}`);
  }

  private async setSession(session: Session): Promise<void> {
    this.session = session;
    await saveSession(session);
  }

  private async requestJson<T>(
    method: HttpMethod,
    path: string,
    body?: unknown,
  ): Promise<T> {
    const response = await this.sendWithAuth(method, path, body);
    if (!response.ok) {
      throw new Error(`El servidor respondió con error: ${response.status}`);
    }
    return response.json() as Promise<T>;
  }

  private async sendWithAuth(
    method: HttpMethod,
    path: string,
    body?: unknown,
  ): Promise<Response> {
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

  private async refreshTokens(): Promise<void> {
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

  private async doRefreshTokens(): Promise<void> {
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

    const auth = (await response.json()) as AuthResponse;
    await this.setSession(sessionFromAuth(auth));
  }

  private async sendOnce(
    method: HttpMethod,
    path: string,
    body?: unknown,
  ): Promise<Response> {
    const accessToken = this.session?.access_token;
    if (!accessToken) {
      throw new SessionExpiredError('No hay sesión activa');
    }

    const headers: Record<string, string> = {
      Authorization: `Bearer ${accessToken}`,
    };
    const init: RequestInit = { method, headers };

    if (body !== undefined) {
      headers['Content-Type'] = 'application/json';
      init.body = JSON.stringify(body);
    }

    return this.fetchWithTimeout(`${API_BASE_URL}${path}`, init);
  }

  private async fetchWithTimeout(
    url: string,
    init: RequestInit = {},
  ): Promise<Response> {
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
