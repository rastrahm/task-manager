/**
 * Modelo de tareas en árbol y utilidades para listas planas.
 * @module tasks
 */

/**
 * Tarea tal como la devuelve `GET /tasks`: puede incluir subtareas en `children`.
 */
export interface Task {
  id: number;
  title: string;
  description: string | null;
  completed: boolean;
  /** Objeto JSON libre; ver {@link metadata} para campos tipados. */
  metadata: unknown;
  parent_id?: number | null;
  children?: Task[];
}

/**
 * Tarea con nivel de anidación para renderizar en `FlatList`.
 */
export interface FlatTask extends Task {
  /** Profundidad en el árbol (0 = raíz). */
  depth: number;
}

/**
 * Aplana un árbol de tareas en orden de recorrido en profundidad.
 * @param {Task[]} tasks - Tareas raíz con `children` anidados.
 * @param {number} [depth=0] - Nivel inicial de sangría.
 * @returns {FlatTask[]} Lista lineal con `depth` para indentación en UI.
 */
export function flattenTasks(tasks: Task[], depth = 0): FlatTask[] {
  return tasks.flatMap(task => {
    const { children = [], ...rest } = task;
    return [
      { ...rest, children, depth },
      ...flattenTasks(children, depth + 1),
    ];
  });
}
