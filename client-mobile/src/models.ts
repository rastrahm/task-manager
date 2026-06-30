/**
 * Modelos de autenticación y usuarios alineados con el backend.
 * @module models
 */

/**
 * Usuario devuelto por el API (sin hash de contraseña).
 */
export interface User {
  /** Identificador numérico del usuario. */
  id: number;
  /** Nombre de inicio de sesión único. */
  username: string;
  /** Si el usuario puede administrar otros usuarios y ver todas las tareas. */
  is_admin: boolean;
  /** Cuenta habilitada para iniciar sesión. */
  is_active: boolean;
  /** Fecha de creación en formato ISO del backend. */
  created_at: string;
  /** Fecha de última actualización en formato ISO del backend. */
  updated_at: string;
}

/**
 * Respuesta de `POST /auth/login` o `POST /auth/refresh`.
 */
export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  /** Segundos hasta la caducidad del access token. */
  expires_in: number;
  user: User;
}

/**
 * Sesión persistida localmente: tokens, caducidad y usuario.
 */
export interface Session {
  access_token: string;
  refresh_token: string;
  /** Marca de tiempo Unix (segundos) de caducidad del access token. */
  expires_at: number;
  user: User;
}

/**
 * Construye una {@link Session} a partir de la respuesta de autenticación.
 * @param {AuthResponse} auth - Respuesta del backend con tokens y usuario.
 * @returns {Session} Sesión lista para persistir en AsyncStorage.
 */
export function sessionFromAuth(auth: AuthResponse): Session {
  const expires_at = Math.floor(Date.now() / 1000) + auth.expires_in;
  return {
    access_token: auth.access_token,
    refresh_token: auth.refresh_token,
    expires_at,
    user: auth.user,
  };
}

/**
 * Indica si el access token caduca en menos de 60 segundos.
 * @param {Session} session - Sesión actual.
 * @returns {boolean} `true` si conviene renovar el token pronto.
 */
export function isAccessExpiringSoon(session: Session): boolean {
  const now = Math.floor(Date.now() / 1000);
  return now >= session.expires_at - 60;
}

/** Cuerpo de `POST /users` (solo administrador). */
export interface CreateUserRequest {
  username: string;
  password: string;
  is_admin: boolean;
}

/** Cuerpo de `PUT /users/:id`; los campos omitidos no se modifican. */
export interface UpdateUserRequest {
  username?: string;
  password?: string;
  is_admin?: boolean;
  is_active?: boolean;
}
