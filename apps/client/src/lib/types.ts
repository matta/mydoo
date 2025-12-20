export interface TodoItem {
  title: string;
  done: boolean;
  children: TodoList;
}

export interface TodoList {
  todos: { [id: string]: TodoItem };
  todoOrder: string[];
}

export interface TodoDoc extends TodoList {}
