/** Modelos de autenticación y usuarios (alineados con el backend). */

export interface User {
  id: number;
  username: string;
  is_admin: boolean;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

export interface Session {
  access_token: string;
  refresh_token: string;
  expires_at: number;
  user: User;
}

export function sessionFromAuth(auth: AuthResponse): Session {
  const expires_at = Math.floor(Date.now() / 1000) + auth.expires_in;
  return {
    access_token: auth.access_token,
    refresh_token: auth.refresh_token,
    expires_at,
    user: auth.user,
  };
}

export function isAccessExpiringSoon(session: Session): boolean {
  const now = Math.floor(Date.now() / 1000);
  return now >= session.expires_at - 60;
}

export interface CreateUserRequest {
  username: string;
  password: string;
  is_admin: boolean;
}

export interface UpdateUserRequest {
  username?: string;
  password?: string;
  is_admin?: boolean;
  is_active?: boolean;
}
