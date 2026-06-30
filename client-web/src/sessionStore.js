const SESSION_KEY = 'task-manager/session';

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

export async function saveSession(session) {
  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
}

export async function clearSession() {
  localStorage.removeItem(SESSION_KEY);
}
