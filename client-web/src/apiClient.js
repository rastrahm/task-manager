import { API_BASE_URL } from './config';
import { isAccessExpiringSoon, sessionFromAuth } from './models';
import { clearSession, loadSession, saveSession } from './sessionStore';

const HTTP_TIMEOUT_MS = 15_000;

export class SessionExpiredError extends Error {
  constructor(message = 'Sesión no válida; vuelve a iniciar sesión') {
    super(message);
    this.name = 'SessionExpiredError';
  }
}

class ApiClient {
  /** @type {import('./models.js').Session | null} */
  session = null;

  /** @type {Promise<void> | null} */
  refreshInFlight = null;

  get currentUser() {
    return this.session?.user ?? null;
  }

  get username() {
    return this.session?.user.username ?? null;
  }

  get isAdmin() {
    return this.session?.user.is_admin ?? false;
  }

  async restoreSession() {
    const stored = loadSession();
    if (stored) {
      this.session = stored;
    }
    return stored;
  }

  async refreshSessionIfNeeded() {
    if (!this.session || !isAccessExpiringSoon(this.session)) {
      return;
    }
    await this.refreshTokens();
  }

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

  clearLocalSession() {
    this.session = null;
    void clearSession();
  }

  async get(path) {
    return this.requestJson('GET', path);
  }

  async post(path, body) {
    return this.requestJson('POST', path, body);
  }

  async postEmpty(path) {
    return this.requestJson('POST', path);
  }

  async put(path, body) {
    return this.requestJson('PUT', path, body);
  }

  async delete(path) {
    const response = await this.sendWithAuth('DELETE', path);
    if (!response.ok) {
      throw new Error(`El servidor respondió con error: ${response.status}`);
    }
  }

  async listUsers() {
    return this.get('/users');
  }

  async createUser(payload) {
    return this.post('/users', payload);
  }

  async updateUser(id, payload) {
    return this.put(`/users/${id}`, payload);
  }

  async deleteUser(id) {
    await this.delete(`/users/${id}`);
  }

  async setSession(session) {
    this.session = session;
    await saveSession(session);
  }

  async requestJson(method, path, body) {
    const response = await this.sendWithAuth(method, path, body);
    if (!response.ok) {
      throw new Error(`El servidor respondió con error: ${response.status}`);
    }
    return response.json();
  }

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

export const apiClient = new ApiClient();
