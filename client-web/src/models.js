/** Modelos de autenticación y usuarios (alineados con el backend). */

export function sessionFromAuth(auth) {
  const expires_at = Math.floor(Date.now() / 1000) + auth.expires_in;
  return {
    access_token: auth.access_token,
    refresh_token: auth.refresh_token,
    expires_at,
    user: auth.user,
  };
}

export function isAccessExpiringSoon(session) {
  const now = Math.floor(Date.now() / 1000);
  return now >= session.expires_at - 60;
}
