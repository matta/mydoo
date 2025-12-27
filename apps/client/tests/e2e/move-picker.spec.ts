import {test} from './fixtures';

test.describe('Move Picker Interactions', () => {
  test.beforeEach(async ({plan}) => {
    await plan.primeWithSampleData();
  });

  test('Move Task to Another Parent', async ({plan}) => {
    // Create a fresh hierarchy for predictable testing
    const rootTask = 'Move Root';
    const childTask = 'Move Child';
    const targetTask = 'Move Target';

    await test.step('Setup Hierarchy', async () => {
      // Create independent tasks
      await plan.createTask(rootTask);
      await plan.createTask(targetTask);

      // Create child under root
      // (Using previous flow: Select Root -> Add Child)
      await plan.openTaskEditor(rootTask);
      await plan.addChild(childTask);

      // Verify initial state
      await plan.toggleExpand(rootTask);
      await plan.verifyTaskVisible(childTask);
    });

    await test.step('Move Child to Target', async () => {
      // 1. Open Move Picker for Child
      // Note: Child must be visible. It is visible because we expanded root.
      await plan.openMovePicker(childTask);

      // 2. Select Target (move happens immediately on selection)
      await plan.moveTaskTo(targetTask);
    });

    await test.step('Verify Move Result', async () => {
      // Expand Target to reveal moved child
      await plan.toggleExpand(targetTask);

      // Verify Child is now under Target
      await plan.verifyTaskVisible(childTask);
    });
  });

  test('Prevents Moving Task to Own Descendant (Cycle Detection)', async ({
    plan,
  }) => {
    // Build hierarchy: Parent -> Child -> Grandchild
    const parent = 'Cycle Parent';
    const child = 'Cycle Child';
    const grandchild = 'Cycle Grandchild';

    await test.step('Setup Three-Level Hierarchy', async () => {
      await plan.createTask(parent);
      await plan.openTaskEditor(parent);
      await plan.addChild(child);

      await plan.toggleExpand(parent);
      await plan.openTaskEditor(child);
      await plan.addChild(grandchild);
    });

    await test.step('Verify Descendants Are Excluded From Move Picker', async () => {
      // Open Move Picker for Parent
      await plan.openMovePicker(parent);

      // Child should NOT be a valid target (would create cycle)
      await plan.verifyMovePickerExcludes(child);

      // Grandchild should also NOT be a valid target
      await plan.verifyMovePickerExcludes(grandchild);
    });
  });
});
