const STORAGE_KEY = 'task-manager-theme';

export function getStoredTheme() {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === 'dark' ? 'dark' : 'light';
}

export function applyTheme(mode) {
  document.documentElement.dataset.theme = mode;
  localStorage.setItem(STORAGE_KEY, mode);
}

export function initTheme() {
  applyTheme(getStoredTheme());
}

export function toggleTheme() {
  const next = getStoredTheme() === 'light' ? 'dark' : 'light';
  applyTheme(next);
  return next;
}
