/**
 * Modelos de autenticación y usuarios alineados con el backend.
 * @module models
 */

/**
 * Usuario devuelto por el API (sin hash de contraseña).
 * @typedef {object} User
 * @property {number} id - Identificador numérico.
 * @property {string} username - Nombre de inicio de sesión.
 * @property {boolean} is_admin - Rol administrador.
 * @property {boolean} is_active - Cuenta habilitada.
 * @property {string} created_at - Fecha ISO de creación.
 * @property {string} updated_at - Fecha ISO de última actualización.
 */

/**
 * Respuesta de `POST /auth/login` o `POST /auth/refresh`.
 * @typedef {object} AuthResponse
 * @property {string} access_token
 * @property {string} refresh_token
 * @property {string} token_type
 * @property {number} expires_in - Segundos hasta caducidad del access token.
 * @property {User} user
 */

/**
 * Sesión persistida en `localStorage`.
 * @typedef {object} Session
 * @property {string} access_token
 * @property {string} refresh_token
 * @property {number} expires_at - Marca Unix (segundos) de caducidad del access token.
 * @property {User} user
 */

/**
 * Cuerpo de `POST /users` (solo administrador).
 * @typedef {object} CreateUserRequest
 * @property {string} username
 * @property {string} password
 * @property {boolean} is_admin
 */

/**
 * Cuerpo de `PUT /users/:id`.
 * @typedef {object} UpdateUserRequest
 * @property {string} [username]
 * @property {string} [password]
 * @property {boolean} [is_admin]
 * @property {boolean} [is_active]
 */

/**
 * Construye una {@link Session} a partir de la respuesta de autenticación.
 * @param {AuthResponse} auth - Respuesta del backend.
 * @returns {Session}
 */
export function sessionFromAuth(auth) {
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
 * @returns {boolean}
 */
export function isAccessExpiringSoon(session) {
  const now = Math.floor(Date.now() / 1000);
  return now >= session.expires_at - 60;
}
