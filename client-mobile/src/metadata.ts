/**
 * Metadatos opcionales de tareas (`metadata` JSON del backend).
 * @module metadata
 */

/** Valores de prioridad admitidos en formularios y API. */
export const PRIORITIES = ['baja', 'media', 'alta'] as const;

/** Prioridad de una tarea. */
export type Priority = (typeof PRIORITIES)[number];

/**
 * Subconjunto tipado del campo `metadata` de una tarea.
 */
export interface TaskMetadata {
  priority?: Priority;
  /** Fecha en formato `AAAA-MM-DD`. */
  due_date?: string;
  tags?: string[];
}

/**
 * Parsea `metadata` desconocido a un objeto tipado; ignora campos inválidos.
 * @param {unknown} value - Valor `metadata` de una tarea.
 * @returns {TaskMetadata} Metadatos normalizados (puede estar vacío).
 */
export function parseMetadata(value: unknown): TaskMetadata {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return {};
  }
  const raw = value as Record<string, unknown>;
  const metadata: TaskMetadata = {};

  if (typeof raw.priority === 'string' && PRIORITIES.includes(raw.priority as Priority)) {
    metadata.priority = raw.priority as Priority;
  }
  if (typeof raw.due_date === 'string' && raw.due_date.trim()) {
    metadata.due_date = raw.due_date.trim();
  }
  if (Array.isArray(raw.tags)) {
    const tags = raw.tags.filter((tag): tag is string => typeof tag === 'string' && tag.trim().length > 0);
    if (tags.length > 0) {
      metadata.tags = tags;
    }
  }

  return metadata;
}

/**
 * Serializa metadatos para enviar al API (`POST` / `PUT` tareas).
 * @param {TaskMetadata} metadata - Metadatos del formulario.
 * @returns {Record<string, unknown>} Objeto JSON sin claves vacías.
 */
export function metadataToJson(metadata: TaskMetadata): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  if (metadata.priority) {
    result.priority = metadata.priority;
  }
  if (metadata.due_date) {
    result.due_date = metadata.due_date;
  }
  if (metadata.tags && metadata.tags.length > 0) {
    result.tags = metadata.tags;
  }
  return result;
}

/**
 * Texto compacto para mostrar bajo el título en la lista.
 * @param {TaskMetadata} metadata - Metadatos parseados.
 * @returns {string | null} Resumen legible o `null` si no hay datos.
 * @example
 * metadataSummary({ priority: 'alta', due_date: '2026-03-20', tags: ['casa'] });
 * // '[alta] · 2026-03-20 · #casa'
 */
export function metadataSummary(metadata: TaskMetadata): string | null {
  const parts: string[] = [];
  if (metadata.priority) {
    parts.push(`[${metadata.priority}]`);
  }
  if (metadata.due_date) {
    parts.push(metadata.due_date);
  }
  if (metadata.tags?.length) {
    parts.push(metadata.tags.map(tag => `#${tag}`).join(' '));
  }
  return parts.length > 0 ? parts.join(' · ') : null;
}

/**
 * Convierte texto del campo etiquetas (`"a, b"`) a array.
 * @param {string} input - Texto separado por comas.
 * @returns {string[] | undefined} Etiquetas o `undefined` si queda vacío.
 */
export function parseTagsInput(input: string): string[] | undefined {
  const tags = input
    .split(',')
    .map(tag => tag.trim())
    .filter(Boolean);
  return tags.length > 0 ? tags : undefined;
}

/**
 * Formatea etiquetas para rellenar el campo de texto del formulario.
 * @param {string[]} [tags] - Lista de etiquetas.
 * @returns {string} Texto unido por comas.
 */
export function tagsToInput(tags?: string[]): string {
  return tags?.join(', ') ?? '';
}
