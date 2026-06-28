import { flattenTasks, Task } from '../src/tasks';

describe('flattenTasks', () => {
  it('aplaniza tareas con hijos anidados', () => {
    const tree: Task[] = [
      {
        id: 1,
        title: 'Padre',
        description: null,
        completed: false,
        metadata: {},
        children: [
          {
            id: 2,
            title: 'Hijo',
            description: null,
            completed: true,
            metadata: {},
            parent_id: 1,
            children: [],
          },
        ],
      },
    ];

    expect(flattenTasks(tree)).toEqual([
      expect.objectContaining({ id: 1, depth: 0, title: 'Padre' }),
      expect.objectContaining({ id: 2, depth: 1, title: 'Hijo' }),
    ]);
  });
});
