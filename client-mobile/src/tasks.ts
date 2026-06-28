export interface Task {
  id: number;
  title: string;
  description: string | null;
  completed: boolean;
  metadata: unknown;
  parent_id?: number | null;
  children?: Task[];
}

export interface FlatTask extends Task {
  depth: number;
}

export function flattenTasks(tasks: Task[], depth = 0): FlatTask[] {
  return tasks.flatMap(task => {
    const { children = [], ...rest } = task;
    return [
      { ...rest, children, depth },
      ...flattenTasks(children, depth + 1),
    ];
  });
}
