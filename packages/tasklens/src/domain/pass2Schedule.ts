import type {EnrichedTask} from '../../src/types';

/**
 * Pass 2: Schedule Inheritance
 * Resolves actionable timeframe.
 * Updates the `schedule` property of tasks if inheriting.
 * @param tasks All tasks in the document.
 */
export function pass2ScheduleInheritance(tasks: EnrichedTask[]): void {
  // Traverse tasks, applying schedule inheritance
  // This might require a topological sort or iterative passes if hierarchy depth is not guaranteed.
  // For simplicity and given max depth 20, a recursive approach or multiple passes are viable.

  // The specification says: "If Task.Schedule is "Once" AND Parent exists, inherit definition."
  // This implies children's schedule should be overridden by parent's if the child is "Once".
  // However, the common pattern is that parent's schedule provides *defaults* if child has none.
  // Let's assume the latter for now: if a child task has no schedule, it inherits from its parent.

  // To handle inheritance correctly, we need to process tasks from roots downwards.
  // We'll iterate through tasks and apply inheritance. A task's parent must be processed first.

  // Create a map for faster access to tasks by ID
  const taskMap = new Map<string, EnrichedTask>();
  tasks.forEach(task => {
    taskMap.set(task.id, task);
  });

  // Sort tasks by parentId (roots first)
  const sortedTasks = [...tasks].sort((a, b) => {
    if (a.parentId === undefined && b.parentId !== undefined) return -1;
    if (a.parentId !== undefined && b.parentId === undefined) return 1;
    return 0; // Maintain original order otherwise
  });

  for (const task of sortedTasks) {
    if (task.parentId !== undefined) {
      const parent = taskMap.get(task.parentId);

      if (task.schedule.type === 'Once' && parent) {
        // Inherit dueDate and leadTime from parent if they are not explicitly set on the child
        // Or, perhaps, always take parent's schedule if type is 'Once'?
        // "inherit definition" strongly suggests taking the parent's entire schedule if the child
        // is designated as 'Once' and has a parent.
        if (parent.schedule.dueDate === undefined) {
          delete task.schedule.dueDate;
        } else {
          task.schedule.dueDate = parent.schedule.dueDate;
        }
        task.schedule.leadTime = parent.schedule.leadTime;
      }
    }
  }
}
