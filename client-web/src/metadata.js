export const PRIORITIES = ['baja', 'media', 'alta'];

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

export function parseTagsInput(input) {
  const tags = input
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean);
  return tags.length > 0 ? tags : undefined;
}

export function tagsToInput(tags) {
  return tags?.join(', ') ?? '';
}
