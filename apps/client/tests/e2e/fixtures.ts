import type {Page} from '@playwright/test';
import {test as base} from '@playwright/test';

// Define the custom fixture type
type PlanFixture = {
  createTask: (title: string) => Promise<void>;
  selectTask: (title: string) => Promise<void>;
  addChild: (title: string) => Promise<void>;
  openTaskEditor: (title: string) => Promise<void>;
  clickMoveButton: () => Promise<void>;
  toggleExpand: (title: string) => Promise<void>;
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

    // 2. Expect "Create Task" modal to appear (implied by placeholder presence)
    const input = this.page.getByPlaceholder('What needs to be done?');
    await input.waitFor({state: 'visible', timeout: 3000});

    // 3. Fill and Save
    await input.fill(title);
    await input.press('Enter');

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
    await chevron.click({force: true});
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
    });
  },
});

export {expect} from '@playwright/test';
