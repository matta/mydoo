export interface TodoItem {
  title: string;
  done: boolean;
  children: TodoList;
}

export interface TodoList {
  todos: Record<string, TodoItem>;
  todoOrder: string[];
}

export type TodoDoc = TodoList;
