import {expect, type Page} from '@playwright/test';

/**
 * PlanFixture - The contract for E2E test helpers.
 *
 * This interface defines all actions available to tests via the `plan` fixture.
 * PlanPage implements this interface, ensuring type safety and enabling
 * click-through navigation from test call sites to implementations.
 */
export interface PlanFixture {
  // Core Task Operations
  createTask: (title: string) => Promise<void>;
  addFirstTask: (title: string) => Promise<void>;
  addChild: (title: string) => Promise<void>;
  addSibling: (targetTitle: string, siblingTitle: string) => Promise<void>;
  openTaskEditor: (title: string) => Promise<void>;
  clickMoveButton: () => Promise<void>;
  toggleExpand: (title: string, shouldExpand?: boolean) => Promise<void>;
  completeTask: (title: string) => Promise<void>;
  clearCompletedTasks: () => Promise<void>;
  editTaskTitle: (title: string, newTitle: string) => Promise<void>;
  deleteTask: (title: string, expectedDescendants?: number) => Promise<void>;

  // Verification Helpers
  verifyTaskVisible: (title: string) => Promise<void>;
  verifyTaskHidden: (title: string) => Promise<void>;
  verifyTaskCompleted: (title: string) => Promise<void>;
  verifyFocusedByLabel: (label: string) => Promise<void>;

  // Mobile Helpers
  mobileDrillDown: (title: string) => Promise<void>;
  mobileNavigateUpLevel: () => Promise<void>;
  mobileVerifyViewTitle: (title: string) => Promise<void>;
  mobileVerifyMobileBottomBar: () => Promise<void>;

  // Move Picker Helpers
  openMovePicker: (title: string) => Promise<void>;
  moveTaskTo: (targetTitle: string) => Promise<void>;
  verifyMovePickerExcludes: (title: string) => Promise<void>;

  // Plan View Specific
  findInPlan: (title: string) => Promise<void>;

  // Navigation
  switchToPlanView: () => Promise<void>;
  switchToDoView: () => Promise<void>;

  // Lifecycle / Setup
  primeWithSampleData: () => Promise<void>;

  // Document Management
  getCurrentDocumentId: () => Promise<string | undefined>;
  createNewDocument: () => Promise<void>;
  switchToDocument: (id: string) => Promise<void>;
}

/**
 * PlanPage - Implementation of PlanFixture.
 *
 * This class implements all fixture methods. Using `implements PlanFixture` ensures:
 * - TypeScript verifies all interface methods are implemented
 * - Method signatures match exactly
 * - Click-through navigation works from test sites to these implementations
 */
export class PlanPage implements PlanFixture {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  // --- Core Task Operations ---

  async createTask(title: string): Promise<void> {
    await this.switchToPlanView();

    const addFirst = this.page.getByRole('button', {name: 'Add First Task'});
    const appendRow = this.page.getByTestId('append-row-button');
    const addTop = this.page.getByLabel('Add Task at Top');

    if (await addFirst.isVisible()) {
      await addFirst.click();
    } else if (await appendRow.isVisible()) {
      await appendRow.click();
    } else if (await addTop.isVisible()) {
      await addTop.click();
    } else {
      throw new Error('No create task trigger found (Empty, Append, or Top)');
    }

    await expect(
      this.page.getByRole('heading', {name: 'Create Task'}),
    ).toBeVisible();

    await this.page.getByRole('textbox', {name: 'Title'}).fill(title);
    await this.page.keyboard.press('Enter');

    // Wait for creation
    await expect(
      this.page.locator(`[data-testid="task-item"]`, {hasText: title}).first(),
    ).toBeVisible();
  }

  async addFirstTask(title: string): Promise<void> {
    await this.page.getByRole('button', {name: 'Add First Task'}).click();
    const modal = this.page.getByRole('dialog', {name: 'Create Task'});
    await this.verifyFocusedByLabel('Title');
    await modal.getByRole('textbox', {name: 'Title'}).fill(title);
    await modal.getByRole('button', {name: 'Create Task'}).click();
    await expect(modal).not.toBeVisible();
  }

  async addChild(title: string): Promise<void> {
    // Strategy: Assume Task Editor is open (clicking a task title opens it).
    const editModal = this.page.getByRole('dialog', {name: 'Edit Task'});
    await editModal.waitFor({state: 'visible', timeout: 3000});

    // Click "Add Child" in the footer
    await editModal.getByRole('button', {name: 'Add Child'}).click();

    // Expect "Create Task" modal to appear
    const createModal = this.page.getByRole('dialog', {name: 'Create Task'});
    await createModal.waitFor({state: 'visible', timeout: 3000});
    await this.verifyFocusedByLabel('Title');
    await createModal.getByRole('textbox', {name: 'Title'}).fill(title);
    await createModal.getByRole('button', {name: 'Create Task'}).click();
    await expect(createModal).not.toBeVisible();
  }

  async addSibling(targetTitle: string, siblingTitle: string): Promise<void> {
    const row = this.page
      .locator(`[data-testid="task-item"]`, {hasText: targetTitle})
      .first();
    await row.hover();
    await row.getByTestId('task-menu-trigger').click();
    await this.page.getByRole('menuitem', {name: 'Add Sibling'}).click();

    const modal = this.page.getByRole('dialog', {name: 'Create Task'});
    await this.verifyFocusedByLabel('Title');
    await modal.getByRole('textbox', {name: 'Title'}).fill(siblingTitle);
    await modal.getByRole('button', {name: 'Create Task'}).click();
    await expect(modal).not.toBeVisible();
  }

  async openTaskEditor(title: string): Promise<void> {
    await this.page.getByText(title, {exact: true}).click();
    await expect(
      this.page.getByRole('dialog', {name: 'Edit Task'}),
    ).toBeVisible();
  }

  async clickMoveButton(): Promise<void> {
    await this.page.getByRole('button', {name: 'Move...'}).click();
  }

  async toggleExpand(
    title: string,
    shouldExpand: boolean = true,
  ): Promise<void> {
    // Find the task row first
    const row = this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first();
    await row.scrollIntoViewIfNeeded();

    // Find the chevron button within the row (aria-label="Toggle expansion")
    const chevron = row.getByLabel('Toggle expansion');

    if (await chevron.isVisible()) {
      const isExpandedAttr = await chevron.getAttribute('data-expanded');
      const isExpanded = isExpandedAttr === 'true';

      if (isExpanded !== shouldExpand) {
        await chevron.dispatchEvent('click', {bubbles: true});
      }
    }
  }

  async completeTask(title: string): Promise<void> {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first();

    const checkbox = taskRow.getByRole('checkbox');
    await checkbox.click();
  }

  async clearCompletedTasks(): Promise<void> {
    await this.switchToDoView();
    await this.page.getByRole('button', {name: 'Refresh'}).click();
  }

  async editTaskTitle(title: string, newTitle: string): Promise<void> {
    await this.openTaskEditor(title);
    const modal = this.page.getByRole('dialog', {name: 'Edit Task'});
    await this.verifyFocusedByLabel('Title');
    await modal.getByRole('textbox', {name: 'Title'}).fill(newTitle);
    await modal.getByRole('button', {name: 'Save Changes'}).click();
    await expect(modal).not.toBeVisible();
  }

  async deleteTask(title: string, expectedDescendants?: number): Promise<void> {
    await this.openTaskEditor(title);
    const modal = this.page.getByRole('dialog', {name: 'Edit Task'});

    // Setup dialog handler for cascade confirm
    this.page.once('dialog', dialog => {
      if (expectedDescendants !== undefined) {
        expect(dialog.message()).toContain(
          `${expectedDescendants} descendants`,
        );
      }
      dialog.accept();
    });

    await modal.getByRole('button', {name: 'Delete'}).click();
    await expect(modal).not.toBeVisible();
  }

  // --- Verification Helpers ---

  async verifyTaskVisible(title: string): Promise<void> {
    await expect(this.page.getByText(title).first()).toBeVisible();
  }

  async verifyTaskHidden(title: string): Promise<void> {
    await expect(
      this.page.getByText(title, {exact: true}).first(),
    ).toBeHidden();
  }

  async verifyTaskCompleted(title: string): Promise<void> {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first();
    const titleText = taskRow.getByText(title).first();

    await expect(taskRow).toBeVisible();
    await expect(titleText).toHaveCSS('text-decoration-line', 'line-through');
  }

  async verifyFocusedByLabel(label: string): Promise<void> {
    await expect(this.page.getByLabel(label)).toBeFocused();
  }

  // --- Navigation ---

  async switchToPlanView(): Promise<void> {
    // Works for both Desktop (Sidebar) and Mobile (Footer)
    // We target 'nav' (Desktop Navbar) or 'footer' (Mobile Bottom Bar) to exclude Breadcrumbs (in 'main')
    // On Mobile: Navbar (Hidden), Footer (Visible). last() gets Footer.
    // On Desktop: Navbar (Visible), Footer (Absent). last() gets Navbar.
    await this.page
      .locator('nav, footer')
      .getByRole('button', {name: 'Plan'})
      .last()
      .click();
  }

  async switchToDoView(): Promise<void> {
    await this.page
      .locator('nav, footer')
      .getByRole('button', {name: 'Do'})
      .last()
      .click();
  }

  // --- Mobile Helpers ---

  async mobileDrillDown(title: string): Promise<void> {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, {hasText: title})
      .first();
    await taskRow.getByLabel('Drill down').click();
  }

  async mobileNavigateUpLevel(): Promise<void> {
    await this.page.getByLabel('Up Level').click();
  }

  async mobileVerifyViewTitle(title: string): Promise<void> {
    // In mobile drill-down, the title might be the breadcrumb button
    await expect(this.page.getByRole('button', {name: title})).toBeVisible();
  }

  async mobileVerifyMobileBottomBar(): Promise<void> {
    await expect(this.page.getByLabel('Add Task at Top')).toBeVisible();
    await expect(this.page.getByLabel('Up Level')).toBeVisible();
  }

  // --- Move Picker Helpers ---

  async openMovePicker(title: string): Promise<void> {
    // Open task editor then click Move button
    await this.openTaskEditor(title);
    await this.clickMoveButton();
    await expect(
      this.page.getByRole('dialog', {name: /^Move "/}),
    ).toBeVisible();
  }

  async moveTaskTo(targetTitle: string): Promise<void> {
    const picker = this.page.getByRole('dialog', {name: /^Move "/});
    await picker.getByText(targetTitle, {exact: true}).click();
  }

  async verifyMovePickerExcludes(title: string): Promise<void> {
    // Verify a task is NOT visible as a valid move target (cycle prevention)
    const picker = this.page.getByRole('dialog', {name: /^Move "/});
    const target = picker.getByText(title, {exact: true});
    await expect(target).not.toBeVisible();
  }

  // --- Plan View Specific ---

  async findInPlan(title: string): Promise<void> {
    await this.openTaskEditor(title);
    const modal = this.page.getByRole('dialog', {name: 'Edit Task'});
    await modal.getByRole('button', {name: 'Find in Plan'}).click();
    await expect(modal).not.toBeVisible();
  }

  // --- Lifecycle / Setup ---

  async primeWithSampleData(): Promise<void> {
    // Clear localStorage first to ensure clean state
    await this.page.goto('/');
    await this.page.evaluate(() => localStorage.clear());

    // Now navigate to seed URL
    await this.page.goto('/?seed=true');
    // Ensure the app is loaded by waiting for the Plan button
    await expect(
      this.page
        .locator('nav, footer')
        .getByRole('button', {name: 'Plan'})
        .last(),
    ).toBeVisible();
  }

  // --- Document Management ---

  async getCurrentDocumentId(): Promise<string | undefined> {
    // Open Options menu
    await this.page.getByRole('button', {name: 'Options'}).click();

    // Click Connection
    await this.page.getByRole('menuitem', {name: 'Connection'}).click();

    // Find the modal
    const modal = this.page.getByRole('dialog', {name: 'Connection Info'});
    await expect(modal).toBeVisible();

    // Get the ID from the Code block using a stable test ID
    const id = await modal.getByTestId('document-id').textContent();

    // Close the modal
    await modal.getByRole('button', {name: 'Close'}).click();
    await expect(modal).not.toBeVisible();

    return id?.trim() || undefined;
  }

  async createNewDocument(): Promise<void> {
    // Open Options menu
    await this.page.getByRole('button', {name: 'Options'}).click();

    // Click Connection
    await this.page.getByRole('menuitem', {name: 'Connection'}).click();

    // Find the modal
    const modal = this.page.getByRole('dialog', {name: 'Connection Info'});
    await expect(modal).toBeVisible();

    // Click "Create New Document"
    await modal.getByTestId('reset-document-button').click();

    await expect(
      this.page
        .locator('nav, footer')
        .getByRole('button', {name: 'Plan'})
        .last(),
    ).toBeVisible();
  }

  async switchToDocument(id: string): Promise<void> {
    // Open Options menu
    await this.page.getByRole('button', {name: 'Options'}).click();

    // Click Connection
    await this.page.getByRole('menuitem', {name: 'Connection'}).click();

    // Find the modal
    const modal = this.page.getByRole('dialog', {name: 'Connection Info'});
    await expect(modal).toBeVisible();

    // Fill the ID
    await modal.getByTestId('connect-document-input').fill(id);

    // Click "Connect"
    await modal.getByTestId('connect-document-button').click();

    // The page should reload. Wait for the app to be ready.
    await expect(
      this.page
        .locator('nav, footer')
        .getByRole('button', {name: 'Plan'})
        .last(),
    ).toBeVisible();
  }
}
