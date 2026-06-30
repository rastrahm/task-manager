/**
 * Metadatos opcionales de tareas (`metadata` JSON del backend).
 * @module metadata
 */

/** Valores de prioridad admitidos en formularios y API. */
export const PRIORITIES = ['baja', 'media', 'alta'];

/**
 * Subconjunto tipado del campo `metadata`.
 * @typedef {object} TaskMetadata
 * @property {'baja' | 'media' | 'alta'} [priority]
 * @property {string} [due_date] - Formato `AAAA-MM-DD`.
 * @property {string[]} [tags]
 */

/**
 * Parsea `metadata` desconocido; ignora campos inválidos.
 * @param {unknown} value
 * @returns {TaskMetadata}
 */
export function parseMetadata(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return {};
  }

  const metadata = {};

  if (typeof value.priority === 'string' && PRIORITIES.includes(value.priority)) {
    metadata.priority = value.priority;
  }
  if (typeof value.due_date === 'string' && value.due_date.trim()) {
    metadata.due_date = value.due_date.trim();
  }
  if (Array.isArray(value.tags)) {
    const tags = value.tags.filter((tag) => typeof tag === 'string' && tag.trim().length > 0);
    if (tags.length > 0) {
      metadata.tags = tags;
    }
  }

  return metadata;
}

/**
 * Serializa metadatos para enviar al API.
 * @param {TaskMetadata} metadata
 * @returns {Record<string, unknown>}
 */
export function metadataToJson(metadata) {
  const result = {};
  if (metadata.priority) {
    result.priority = metadata.priority;
  }
  if (metadata.due_date) {
    result.due_date = metadata.due_date;
  }
  if (metadata.tags?.length) {
    result.tags = metadata.tags;
  }
  return result;
}

/**
 * Texto compacto para mostrar bajo el título en la lista.
 * @param {TaskMetadata} metadata
 * @returns {string | null}
 * @example
 * metadataSummary({ priority: 'alta', due_date: '2026-03-20', tags: ['casa'] });
 * // '[alta] · 2026-03-20 · #casa'
 */
export function metadataSummary(metadata) {
  const parts = [];
  if (metadata.priority) {
    parts.push(`[${metadata.priority}]`);
  }
  if (metadata.due_date) {
    parts.push(metadata.due_date);
  }
  if (metadata.tags?.length) {
    parts.push(metadata.tags.map((tag) => `#${tag}`).join(' '));
  }
  return parts.length > 0 ? parts.join(' · ') : null;
}

/**
 * Convierte texto del campo etiquetas (`"a, b"`) a array.
 * @param {string} input
 * @returns {string[] | undefined}
 */
export function parseTagsInput(input) {
  const tags = input
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean);
  return tags.length > 0 ? tags : undefined;
}

/**
 * Formatea etiquetas para el campo de texto del formulario.
 * @param {string[]} [tags]
 * @returns {string}
 */
export function tagsToInput(tags) {
  return tags?.join(', ') ?? '';
}
