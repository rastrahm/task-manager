import {
  metadataSummary,
  metadataToJson,
  parseMetadata,
  parseTagsInput,
} from '../src/metadata';

describe('metadata', () => {
  it('parsea prioridad, fecha y etiquetas', () => {
    expect(
      parseMetadata({
        priority: 'alta',
        due_date: '2026-03-20',
        tags: ['casa', 'urgente'],
      }),
    ).toEqual({
      priority: 'alta',
      due_date: '2026-03-20',
      tags: ['casa', 'urgente'],
    });
  });

  it('genera resumen legible', () => {
    expect(
      metadataSummary({
        priority: 'media',
        due_date: '2026-06-01',
        tags: ['trabajo'],
      }),
    ).toBe('[media] · 2026-06-01 · #trabajo');
  });

  it('convierte a JSON omitiendo campos vacíos', () => {
    expect(metadataToJson({ priority: 'baja' })).toEqual({ priority: 'baja' });
    expect(metadataToJson({})).toEqual({});
  });

  it('parsea etiquetas desde texto', () => {
    expect(parseTagsInput('casa, urgente , trabajo')).toEqual([
      'casa',
      'urgente',
      'trabajo',
    ]);
    expect(parseTagsInput('   ')).toBeUndefined();
  });
});
