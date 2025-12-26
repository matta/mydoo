import type {Page} from '@playwright/test';
import {test as base, expect} from '@playwright/test';

// Define the custom fixture type
type PlanFixture = {
  createTask: (title: string) => Promise<void>;
  selectTask: (title: string) => Promise<void>;
  addChild: (title: string) => Promise<void>;
  openTaskEditor: (title: string) => Promise<void>;
  clickMoveButton: () => Promise<void>;
  toggleExpand: (title: string) => Promise<void>;
  completeTask: (title: string) => Promise<void>;
  clearCompletedTasks: () => Promise<void>;
  verifyTaskVisible: (title: string) => Promise<void>;
  verifyTaskHidden: (title: string) => Promise<void>;
  verifyTaskCompleted: (title: string) => Promise<void>;
};

// Create the fixture class/helper
class PlanPage {
  private readonly page: Page;
  constructor(page: Page) {
    this.page = page;
  }

  async createTask(title: string) {
    await this.page.getByRole('button', {name: 'Plan'}).click();

    // Strategy: Open the Create Modal

    // 1. Try "Add First Task" button (Empty State)
    const addFirst = this.page.getByRole('button', {name: 'Add First Task'});
    if (await addFirst.isVisible()) {
      await addFirst.click();
    } else {
      // 2. Try "Append Row" (Bottom of list)
      // We added data-testid="append-row-button" in PlanViewContainer
      const appendRow = this.page.getByTestId('append-row-button');
      if (await appendRow.isVisible()) {
        await appendRow.click();
      } else {
        // 3. Try "Add Task at Top" (Mobile Bottom Bar)
        const addTop = this.page.getByLabel('Add Task at Top');
        if (await addTop.isVisible()) {
          await addTop.click();
        } else {
          // 4. Fallback: Keyboard shortcut 'n' if implemented, or just 'Enter' on a selected task?
          // If nothing matches, we might be in a state where we can't create.
          // But let's try the keyboard as a last resort.
          await this.page.keyboard.press('n');
        }
      }
    }

    // Modal should now be open. Wait for input.
    const input = this.page.getByPlaceholder('What needs to be done?');
    await input.waitFor({state: 'visible', timeout: 2000});
    await input.fill(title);
    await input.press('Enter');

    // Wait for modal to close or input to clear?
    // Usually "Enter" creates and keeps modal open for next task?
    // If so, we are done. If it closes, we are done.
    // Let's assume we want to close it if we only create one?
    // Or just leave it. The next step usually selects something else.
    // Ideally we close it to return to base state.
    await this.page.keyboard.press('Escape');

    // VERIFY: Wait for the task to appear in the list
    // This ensures subsequent steps don't fail due to race conditions.
    await this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first()
      .waitFor();
  }

  async selectTask(title: string) {
    // Wait for the task to be visible (important after tree expansions)
    const el = this.page.getByText(title, {exact: true}).first();
    await el.waitFor({state: 'visible', timeout: 5000});
    await el.click();
  }

  async addChild(title: string) {
    // Strategy: Assume Task Editor is open (via selectTask).

    // 1. Check if "Edit Task" modal is open and visible
    const editModal = this.page.getByRole('dialog', {name: 'Edit Task'});

    // We expect it to be visible because selectTask opens it.
    // If explicit wait is needed:
    await editModal.waitFor({state: 'visible', timeout: 3000});

    // Click "Add Child" in the footer
    await editModal.getByRole('button', {name: 'Add Child'}).click();

    // 2. Expect "Create Task" modal to appear
    const createModal = this.page.getByRole('dialog', {name: 'Create Task'});
    await createModal.waitFor({state: 'visible', timeout: 3000});

    const input = createModal.getByPlaceholder('What needs to be done?');
    await input.waitFor({state: 'visible', timeout: 3000});

    console.log('Fixture: Filling title Task B');
    await input.click();
    await input.fill(title);

    console.log('Fixture: Pressing Enter');
    await input.press('Enter');
    console.log('Fixture: Pressed Enter');

    // VERIFY: Check if task exists in list (should be visible if auto-expanded)
    // Or at least check if parent now has children?
    // We can't easily check visibility because it might be inside collapsed parent (on desktop).
    // But on Desktop, handleCreate calls expandAll.
    // So it SHOULD be visible.
    try {
      await this.page
        .getByText(title)
        .waitFor({state: 'visible', timeout: 2000});
      console.log(
        `Fixture: addChild - ${title} created and visible immediately.`,
      );
    } catch (_e) {
      console.log(`Fixture: addChild - ${title} NOT visible immediately.`);
    }

    // 4. Close Create Modal
    await this.page.keyboard.press('Escape');
  }

  // To keep it simple and robust for this step:
  // I will assume `createTask` creates at root.
  // `addChild` assumes we opened the editor for the parent? or explicitly finds the parent?
  // Let's implement `openTaskEditor` properly.

  async openTaskEditor(title: string) {
    await this.page.getByText(title, {exact: true}).click();
    await expect(
      this.page.getByRole('dialog', {name: 'Edit Task'}),
    ).toBeVisible();
  }

  async clickMoveButton() {
    await this.page.getByRole('button', {name: 'Move...'}).click();
  }

  async toggleExpand(title: string) {
    // Find the task row first
    const row = this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first();
    await row.scrollIntoViewIfNeeded();

    // Find the chevron button within the row (aria-label="Toggle expansion")
    const chevron = row.getByLabel('Toggle expansion');

    if (await chevron.isVisible()) {
      console.log(`Fixture: toggleExpand checking ${title}.`);
      // Check if valid first
      const isExpanded = await chevron.getAttribute('data-expanded');
      console.log(`Fixture: isExpanded attr: ${isExpanded}`);
      if (isExpanded !== 'true') {
        console.log(`Fixture: Clicking toggle for ${title}`);
        await chevron.dispatchEvent('click', {bubbles: true});
      } else {
        console.log(`Fixture: Already expanded ${title}`);
      }
    } else {
      console.log(`Fixture: Chevron not visible for ${title}`);
    }
  }

  async completeTask(title: string) {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first();

    const checkbox = taskRow.getByRole('checkbox');
    await checkbox.click();
  }

  async clearCompletedTasks() {
    await this.page.getByRole('button', {name: 'Do'}).click();
    await this.page.getByRole('button', {name: 'Refresh'}).click();
  }

  async verifyTaskVisible(title: string) {
    await expect(this.page.getByText(title).first()).toBeVisible();
  }

  async verifyTaskHidden(title: string) {
    await expect(this.page.getByText(title).first()).toBeHidden();
  }

  async verifyTaskCompleted(title: string) {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first();
    const titleText = taskRow.getByText(title).first();

    await expect(taskRow).toBeVisible();
    await expect(titleText).toHaveCSS('text-decoration-line', 'line-through');
  }
}

export const test = base.extend<{plan: PlanFixture}>({
  plan: async ({page}, use) => {
    const planPage = new PlanPage(page);

    // Define the fixture methods using the helper class
    await use({
      createTask: title => planPage.createTask(title),
      selectTask: title => planPage.selectTask(title),
      addChild: title => planPage.addChild(title),
      openTaskEditor: title => planPage.openTaskEditor(title),
      clickMoveButton: () => planPage.clickMoveButton(),
      toggleExpand: title => planPage.toggleExpand(title),
      completeTask: title => planPage.completeTask(title),
      clearCompletedTasks: () => planPage.clearCompletedTasks(),
      verifyTaskVisible: title => planPage.verifyTaskVisible(title),
      verifyTaskHidden: title => planPage.verifyTaskHidden(title),
      verifyTaskCompleted: title => planPage.verifyTaskCompleted(title),
    });
  },
});

export {expect} from '@playwright/test';
