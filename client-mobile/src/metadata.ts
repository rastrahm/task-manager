export const PRIORITIES = ['baja', 'media', 'alta'] as const;
export type Priority = (typeof PRIORITIES)[number];

export interface TaskMetadata {
  priority?: Priority;
  due_date?: string;
  tags?: string[];
}

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

export function parseTagsInput(input: string): string[] | undefined {
  const tags = input
    .split(',')
    .map(tag => tag.trim())
    .filter(Boolean);
  return tags.length > 0 ? tags : undefined;
}

export function tagsToInput(tags?: string[]): string {
  return tags?.join(', ') ?? '';
}
