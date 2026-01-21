import { expect, type Page } from "@playwright/test";

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
  createRoutineTask: (
    title: string,
    config: {
      frequency: string;
      interval: number;
      leadTimeVal: number;
      leadTimeUnit: string;
    },
  ) => Promise<void>;
  refreshDoList: () => Promise<void>;
  advanceTime: (minutes: number) => Promise<void>;
  setDueDate: (dateString: string) => Promise<void>;
  setLeadTime: (value: number, unit: string) => Promise<void>;
  setSequential: (title: string, shouldBeSequential: boolean) => Promise<void>;
  createTaskWithDueDate: (
    title: string,
    config: { dueDate: string; leadTimeVal: number; leadTimeUnit: string },
  ) => Promise<void>;
  createTaskInDoView: (title: string) => Promise<void>;

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
  setupClock: () => Promise<void>;

  // Document Management
  getCurrentDocumentId: () => Promise<string | undefined>;
  createNewDocument: () => Promise<void>;
  switchToDocument: (id: string) => Promise<void>;
  downloadDocument: () => Promise<string>;
  uploadDocument: (filePath: string) => Promise<void>;
  getDetailedDocumentUrl: () => Promise<string>;

  setClock: (now: Date) => Promise<void>;
  closeEditor: () => Promise<void>;
  setTaskDueDate: (dateString: string) => Promise<void>;
  setTaskLeadTime: (value: number, unit: string) => Promise<void>;
  verifyTaskUrgency: (taskTitle: string, urgency: string) => Promise<void>;
  verifyNoDueDateIndicator: (taskTitle: string) => Promise<void>;
  verifyDueDateText: (taskTitle: string, text: string) => Promise<void>;
  verifyDueDateTextContains: (taskTitle: string, part: string) => Promise<void>;
  waitForAppReady: () => Promise<void>;

  // Sync Settings
  openSyncSettings: () => Promise<void>;
  closeSyncSettings: () => Promise<void>;
  setSyncServerUrl: (url: string) => Promise<void>;
  saveSyncSettings: () => Promise<void>;
  verifySyncServerUrl: (url: string) => Promise<void>;

  goto: (path?: string) => Promise<void>;
  evaluate: <T>(fn: () => T) => Promise<T>;
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

  async waitForAppReady(): Promise<void> {
    // Also wait for any pending reloads/hydration
    await this.page.waitForLoadState("load");
    try {
      await this.page.waitForLoadState("networkidle", { timeout: 2000 });
    } catch (_e) {
      // Ignore timeout if network never settles (e.g. constant polling)
    }
  }

  private async getMemoryHeads(): Promise<string> {
    const el = this.page.locator("[data-memory-heads]");
    await expect(el).toBeAttached({ timeout: 5000 });
    return (await el.getAttribute("data-memory-heads")) || "";
  }

  private async waitForPersistence(action: () => Promise<void>): Promise<void> {
    // 1. Capture initial state
    const initialHeads = await this.getMemoryHeads();

    // 2. Perform action
    await action();

    // 3. Wait for Memory Heads to change (sanity check that action worked)
    // This confirms the Store processed the action.
    // We expect heads to change because actions imply mutation.
    await expect(async () => {
      const currentHeads = await this.getMemoryHeads();
      expect(currentHeads).not.toBe(initialHeads);
    }).toPass({ timeout: 5000 });

    // 4. Wait for Consistency: Persisted Heads == Memory Heads
    // This confirms that what represents the current state is safely on disk.
    await expect(async () => {
      const memoryHeads = await this.getMemoryHeads();
      const persistedEl = this.page.locator("[data-persisted-heads]");
      const persistedHeads = await persistedEl.getAttribute(
        "data-persisted-heads",
      );

      expect(persistedHeads).toBe(memoryHeads);
    }).toPass({ timeout: 15000 });
  }

  // --- Core Task Operations ---

  async createTask(title: string): Promise<void> {
    const addFirst = this.page.getByRole("button", { name: "Add First Task" });
    const appendRow = this.page.getByTestId("append-row-button");
    const addTop = this.page.getByLabel("Add Task at Top");

    if (await addFirst.isVisible()) {
      await addFirst.click();
    } else if (await appendRow.isVisible()) {
      await appendRow.click();
    } else if (await addTop.isVisible()) {
      await addTop.click();
    } else {
      const input = this.page
        .getByTestId("plan-task-input")
        .getByPlaceholder("Add a new task...");
      await expect(input).toBeVisible();
      await input.fill(title);
      await this.waitForPersistence(async () => {
        await this.page.keyboard.press("Enter");
      });

      await this.waitForAppReady();
      // Wait for creation
      await expect(
        this.page.locator(`[data-testid="task-item"]`, { hasText: title }),
      ).toBeVisible();
      return;
    }

    const modal = this.page.getByRole("dialog", { name: "Create Task" });
    await expect(modal).toBeVisible({ timeout: 5000 });
    // Now that we added id="task-title-input" and matching label for, getByLabel should work perfectly
    // Focus might be stolen by Dialog focus trap
    await this.page.getByLabel("Title").fill(title);
    await this.waitForPersistence(async () => {
      await this.page.keyboard.press("Enter");
    });

    // Wait for modal to close to ensure creation process is done
    await expect(modal).toBeHidden();

    // Wait for creation
    await expect(
      this.page.locator(`[data-testid="task-item"]`, { hasText: title }),
    ).toBeVisible();
  }

  async createTaskInDoView(title: string): Promise<void> {
    await this.switchToDoView();
    const input = this.page
      .getByTestId("do-task-input")
      .getByPlaceholder("Add a new task...");
    await expect(input).toBeVisible();
    await input.fill(title);
    await this.page.keyboard.press("Enter");

    await expect(this.page.getByText(title).first()).toBeVisible();
  }

  async addFirstTask(title: string): Promise<void> {
    await this.createTask(title);
  }

  async addChild(title: string): Promise<void> {
    const editModal = this.page.getByRole("dialog", { name: "Edit Task" });
    if (await editModal.isVisible()) {
      await editModal.getByRole("button", { name: "Add Child" }).click();
    } else {
      // Fallback: If no editor, we'd need the parent title passed in.
      // For now, these steps usually follow "open editor for parent".
      throw new Error(
        "addChild expects Task Editor to be open for the parent task.",
      );
    }

    // Expect "Create Task" modal to appear
    const createModal = this.page.getByRole("dialog", { name: "Create Task" });
    await expect(createModal).toBeVisible({ timeout: 3000 });

    await createModal.getByRole("textbox", { name: "Title" }).fill(title);
    await createModal.getByRole("button", { name: "Create Task" }).click();
    await expect(createModal).not.toBeVisible();
    await this.waitForAppReady();

    const child = this.page.getByText(title, { exact: true }).first();
    await expect(child).toBeVisible();
  }

  async addSibling(targetTitle: string, siblingTitle: string): Promise<void> {
    const row = this.page
      .locator(`[data-testid="task-item"]`, { hasText: targetTitle })
      .first();

    await expect(row).toBeVisible({ timeout: 5000 });

    const style = await row.getAttribute("style");
    const isRoot = !style || style.includes("padding-left: 0px");

    if (isRoot) {
      const input = this.page
        .getByTestId("plan-task-input")
        .getByPlaceholder("Add a new task...");
      await expect(input).toBeVisible();
      await input.fill(siblingTitle);
      await this.page.keyboard.press("Enter");
    } else {
      // Non-root sibling: Hover and click "Add Subtask" on PARENT.
      // But we don't know parent here. This is a limitation.
      // For now, BDD tests mainly add siblings to root.
      throw new Error(
        `Add Sibling not implemented for non-root tasks in Dioxus UI yet. Target: ${targetTitle}`,
      );
    }

    await this.waitForAppReady();
    await expect(
      this.page.getByText(siblingTitle, { exact: true }).first(),
    ).toBeVisible();
  }

  async openTaskEditor(title: string): Promise<void> {
    const modal = this.page.getByRole("dialog", { name: "Edit Task" });

    if (await modal.isVisible()) {
      const titleInput = modal.getByRole("textbox", { name: "Title" });
      const currentTitle = await titleInput.inputValue();
      if (currentTitle === title) {
        return; // Already open
      }
      // If open but wrong title, validation or close logic would go here.
      // For now, assume we might need to close it or just click the other task (which might close it).
      // Let's press escape to close it to be safe.
      await this.page.keyboard.press("Escape");
      await expect(modal).not.toBeVisible();
    }

    // Retry loop: If task not found, try navigating up (up to 5 levels)
    // This handles cases where we are drilled down into a child task, but we want to edit the parent
    // (which is now in the header/breadcrumb and not in the list).
    for (let i = 0; i < 5; i++) {
      const row = this.page
        .locator(`[data-testid="task-item"]`, { hasText: title })
        .first();
      if ((await row.count()) > 0 && (await row.isVisible())) {
        await row.click();
        await expect(modal).toBeVisible();
        return;
      }

      // Not found visible in list. Check if we can go up.
      const upLevel = this.page.getByLabel("Up Level");
      if ((await upLevel.isVisible()) && (await upLevel.isEnabled())) {
        // Capture breadcrumb count before navigating up
        const breadcrumbs = this.page.locator(
          ".mantine-Breadcrumbs-root button",
        );
        const countBefore = await breadcrumbs.count();

        await upLevel.click();

        // Wait for breadcrumb count to decrease, indicating nav is complete
        await expect(breadcrumbs).toHaveCount(countBefore - 1);
      } else {
        // Cannot go up further, and item not found.
        break;
      }
    }

    // Final attempt (or failure if loop exhausted)
    const finalRow = this.page
      .locator(`[data-testid="task-item"]`, { hasText: title })
      .first();
    await finalRow.waitFor({ state: "visible" });
    await finalRow.click();
    await expect(modal).toBeVisible();
  }

  async clickMoveButton(): Promise<void> {
    await this.page.getByRole("button", { name: "Move..." }).click();
  }

  async toggleExpand(
    title: string,
    shouldExpand: boolean = true,
  ): Promise<void> {
    const row = this.page
      .locator(`[data-testid="task-item"]`, { hasText: title })
      .first();
    await expect(row).toBeVisible({ timeout: 5000 });

    const toggle = row.getByLabel("Toggle expansion");

    // If the toggle isn't visible, the task might not have children yet (or just got its first one)
    // Wait a bit for the UI to update if we expect it to have children.
    if (!(await toggle.isVisible()) && shouldExpand) {
      await this.page.waitForTimeout(500);
    }

    if (await toggle.isVisible()) {
      const isExpanded =
        (await toggle.getAttribute("data-expanded")) === "true";
      if (isExpanded !== shouldExpand) {
        await toggle.click();
        await this.waitForAppReady();
      }
    }
  }

  async completeTask(title: string): Promise<void> {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, { hasText: title })
      .first();

    const checkbox = taskRow.getByRole("checkbox");
    await checkbox.click();
  }

  async clearCompletedTasks(): Promise<void> {
    await this.switchToDoView();
    await this.page.getByRole("button", { name: "Refresh" }).click();
  }

  async editTaskTitle(title: string, newTitle: string): Promise<void> {
    await this.openTaskEditor(title);
    const modal = this.page.getByRole("dialog", { name: "Edit Task" });
    // Focus might be stolen by Dialog focus trap
    await modal.getByRole("textbox", { name: "Title" }).fill(newTitle);
    await this.waitForPersistence(async () => {
      await modal.getByRole("button", { name: "Save Changes" }).click();
    });
    await expect(modal).not.toBeVisible();
  }

  async deleteTask(title: string, expectedDescendants?: number): Promise<void> {
    await this.openTaskEditor(title);
    const modal = this.page.getByRole("dialog", { name: "Edit Task" });

    // Setup dialog handler for cascade confirm
    this.page.once("dialog", (dialog) => {
      if (expectedDescendants !== undefined) {
        expect(dialog.message()).toContain(
          `${expectedDescendants} descendants`,
        );
      }
      dialog.accept();
    });

    await modal.getByRole("button", { name: "Delete" }).click();
    await expect(modal).not.toBeVisible();
  }

  async createRoutineTask(
    title: string,
    config: {
      frequency: string;
      interval: number;
      leadTimeVal: number;
      leadTimeUnit: string;
    },
  ): Promise<void> {
    await this.createTask(title);
    await this.openTaskEditor(title);

    // Select "Routinely" first to reveal repetition fields
    await this.page.selectOption("#schedule-type-select", "Routinely");

    // Map frequency values to labels
    const freqLabels: Record<string, string> = {
      minutes: "Minutes",
      hours: "Hours",
      daily: "Daily",
    } as const;
    const optionName = freqLabels[config.frequency] || "Daily";

    // Select the option
    await this.page.selectOption("#repetition-frequency-select", optionName);

    // Fill Interval "Every X units"
    await this.page
      .locator("#repetition-interval-input")
      .fill(config.interval.toString());

    // Fill Lead Time
    await this.page
      .locator("#lead-time-scalar-input")
      .fill(config.leadTimeVal.toString());

    // "Unit" Select for Lead Time
    await this.page.selectOption("#lead-time-unit-select", config.leadTimeUnit);

    // Save
    await this.page.getByRole("button", { name: "Save Changes" }).click();
    await expect(
      this.page.getByRole("dialog", { name: "Edit Task" }),
    ).not.toBeVisible();
  }

  async refreshDoList(): Promise<void> {
    await this.page.getByRole("button", { name: "Refresh" }).click();
  }

  async advanceTime(minutes: number): Promise<void> {
    // This requires that the page clock is installed and controllable.
    // We assume the test runner or fixture handles install(), or we try to fastForward.
    await this.page.clock.fastForward(minutes * 60 * 1000);
  }

  async setDueDate(dateString: string): Promise<void> {
    // Assumes the Task Editor modal is already open
    const input = this.page.getByTestId("date-input");
    await input.fill(dateString);
    await expect(input).toHaveValue(dateString);
    await input.blur();
  }

  async setLeadTime(value: number, unit: string): Promise<void> {
    // Assumes the Task Editor modal is already open
    await this.page.locator("#lead-time-scalar-input").fill(value.toString());
    await this.page.selectOption("#lead-time-unit-select", { label: unit });
  }

  async setTaskDueDate(dateString: string): Promise<void> {
    // Ensure Schedule Type is set to "Due Date" so the field is visible
    await this.page.selectOption("#schedule-type-select", {
      label: "Due Date",
    });
    await this.setDueDate(dateString);
  }

  async setTaskLeadTime(value: number, unit: string): Promise<void> {
    await this.setLeadTime(value, unit);
  }

  async createTaskWithDueDate(
    title: string,
    config: { dueDate: string; leadTimeVal: number; leadTimeUnit: string },
  ): Promise<void> {
    await this.createTask(title);
    await this.openTaskEditor(title);
    await this.setDueDate(config.dueDate);
    await this.setLeadTime(config.leadTimeVal, config.leadTimeUnit);
    await this.page.getByRole("button", { name: "Save Changes" }).click();
    await expect(
      this.page.getByRole("dialog", { name: "Edit Task" }),
    ).not.toBeVisible();
  }

  async setSequential(
    title: string,
    shouldBeSequential: boolean,
  ): Promise<void> {
    await this.openTaskEditor(title);
    const modal = this.page.getByRole("dialog", { name: "Edit Task" });
    const toggle = modal.getByLabel("Sequential Project");

    const isChecked = await toggle.isChecked();
    if (isChecked !== shouldBeSequential) {
      // For Mantine switches, clicking the label text is more reliable
      const label = modal
        .locator("label")
        .filter({ hasText: "Sequential Project" });
      await label.dispatchEvent("click", { bubbles: true });
    }

    const saveButton = modal.getByRole("button", { name: "Save Changes" });
    await saveButton.scrollIntoViewIfNeeded();
    await saveButton.click({ force: true });
    await expect(modal).not.toBeVisible();
  }

  // --- Verification Helpers ---

  async verifyTaskVisible(title: string): Promise<void> {
    await expect(
      this.page.getByText(title, { exact: true }).first(),
    ).toBeVisible();
  }

  async verifyTaskHidden(title: string): Promise<void> {
    await expect(
      this.page.getByText(title, { exact: true }).first(),
    ).toBeHidden();
  }

  async verifyTaskCompleted(title: string, timeout = 5000): Promise<void> {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, { hasText: title })
      .first();
    const titleText = taskRow.getByText(title).first();

    await expect(taskRow).toBeVisible({ timeout });
    await expect(titleText).toHaveCSS("text-decoration-line", "line-through", {
      timeout,
    });
  }

  async verifyFocusedByLabel(label: string): Promise<void> {
    await expect(this.page.getByLabel(label)).toBeFocused();
  }

  // --- Navigation ---

  async switchToPlanView(): Promise<void> {
    await this.waitForAppReady();
    await this.page
      .locator("nav, footer, .navbar")
      .getByText("Plan")
      .last()
      .click();
    await this.waitForAppReady();
  }

  async switchToDoView(): Promise<void> {
    await this.waitForAppReady();
    await this.page
      .locator("nav, footer, .navbar")
      .getByText("Do")
      .last()
      .click();
    await this.waitForAppReady();
  }

  // --- Mobile Helpers ---

  async mobileDrillDown(title: string): Promise<void> {
    const taskRow = this.page
      .locator(`[data-testid="task-item"]`, { hasText: title })
      .first();
    await taskRow.getByLabel("Drill down").click();
  }

  async mobileNavigateUpLevel(): Promise<void> {
    await this.page.getByLabel("Up Level").click();
  }

  async mobileVerifyViewTitle(title: string): Promise<void> {
    // In mobile drill-down, the title might be the breadcrumb button
    await expect(this.page.getByRole("button", { name: title })).toBeVisible();
  }

  async mobileVerifyMobileBottomBar(): Promise<void> {
    await expect(this.page.getByLabel("Add Task at Top")).toBeVisible();
    await expect(this.page.getByLabel("Up Level")).toBeVisible();
  }

  // --- Move Picker Helpers ---

  async openMovePicker(title: string): Promise<void> {
    // Open task editor then click Move button
    await this.openTaskEditor(title);
    await this.clickMoveButton();
    await expect(
      this.page.getByRole("dialog", { name: /^Move "/ }),
    ).toBeVisible();
  }

  async moveTaskTo(targetTitle: string): Promise<void> {
    const picker = this.page.getByRole("dialog", { name: /^Move "/ });
    await picker.getByText(targetTitle, { exact: true }).click();
  }

  async verifyMovePickerExcludes(title: string): Promise<void> {
    // Verify a task is NOT visible as a valid move target (cycle prevention)
    const picker = this.page.getByRole("dialog", { name: /^Move "/ });
    const target = picker.getByText(title, { exact: true });
    await expect(target).not.toBeVisible();
  }

  // --- Plan View Specific ---

  async findInPlan(title: string): Promise<void> {
    await this.openTaskEditor(title);
    const modal = this.page.getByRole("dialog", { name: "Edit Task" });
    await modal.getByRole("button", { name: "Find in Plan" }).click();
    await expect(modal).not.toBeVisible();
  }

  // --- Lifecycle / Setup ---

  async setupClock(): Promise<void> {
    try {
      await this.page.clock.install();
    } catch (e: unknown) {
      if (!(e instanceof Error && e.message?.includes("already installed"))) {
        throw e;
      }
    }
  }

  async clearAndReload(): Promise<void> {
    // Clear localStorage first to ensure clean state
    await this.page.goto("/");
    await this.page.evaluate(() => {
      localStorage.clear();
    });

    // Reload the app
    await this.page.goto("/plan");
    await this.waitForAppReady();
    // Ensure the app is loaded by waiting for the Plan heading
    await expect(
      this.page.getByRole("heading", { name: "Plan" }),
    ).toBeVisible();
  }

  async primeWithSampleData(): Promise<void> {
    // Navigate to seed URL which triggers internal seeding logic
    await this.page.goto("/plan?seed=true");
    await this.waitForAppReady();
    // Ensure the app is loaded by waiting for the Plan heading
    await expect(
      this.page.getByRole("heading", { name: "Plan" }),
    ).toBeVisible();
  }

  // --- Document Management ---

  async getCurrentDocumentId(): Promise<string | undefined> {
    // Open Settings modal
    await this.page.getByTestId("settings-button").click();

    // Find the modal
    const modal = this.page.getByRole("dialog", { name: "Settings" });
    await expect(modal).toBeVisible();

    // Get the ID from the hidden span using data-testid
    const id = await modal.getByTestId("full-document-id").textContent();

    // Close the modal
    await modal.getByTestId("close-settings").click();
    await expect(modal).not.toBeVisible();

    return id?.trim() || undefined;
  }

  async createNewDocument(): Promise<void> {
    // Open Settings modal
    await this.page.getByTestId("settings-button").click();

    // Find the modal
    const modal = this.page.getByRole("dialog", { name: "Settings" });
    await expect(modal).toBeVisible();

    // Click "New Document"
    await this.waitForPersistence(async () => {
      await modal.getByTestId("new-document-button").click();
    });

    // Modal remains open or closes depending on implementation,
    // but the app should reload. Let's close modal.
    await modal.getByTestId("close-settings").click();
    await expect(modal).not.toBeVisible();
    await this.waitForAppReady();
  }

  async switchToDocument(id: string): Promise<void> {
    // Open Settings modal
    await this.page.getByTestId("settings-button").click();

    // Find the modal
    const modal = this.page.getByRole("dialog", { name: "Settings" });
    await expect(modal).toBeVisible();

    // Click "Enter ID" to reveal input
    const toggleBtn = modal.getByTestId("toggle-enter-id-button");
    if ((await toggleBtn.textContent()) === "Enter ID") {
      await toggleBtn.click();
    }

    // Fill the ID
    const input = modal.getByTestId("document-id-input");
    await input.fill(id);

    // Click "Load Document"
    await modal.getByTestId("load-document-button").click();

    // Wait for the app to be ready.
    await modal.getByTestId("close-settings").click();
    await expect(modal).not.toBeVisible();
    await this.waitForAppReady();
  }

  // --- Clock Control ---

  async setClock(now: Date): Promise<void> {
    await this.setupClock();
    await this.page.clock.setFixedTime(now);
  }

  // --- UI Helpers ---

  async closeEditor(): Promise<void> {
    const modal = this.page.getByRole("dialog", { name: "Edit Task" });
    if (await modal.isVisible()) {
      await modal.getByRole("button", { name: "Save Changes" }).click();
      await expect(modal).not.toBeVisible();
    }
  }

  // --- Verification ---

  async verifyTaskUrgency(taskTitle: string, urgency: string): Promise<void> {
    const row = this.page
      .locator(`[data-testid="task-item"]`, { hasText: taskTitle })
      .first();
    const breadcrumb = this.page
      .locator(".mantine-Breadcrumbs-root button", { hasText: taskTitle })
      .first();

    const badge = row.locator(
      `[data-testid="urgency-badge"][data-urgency="${urgency}"]`,
    );
    const breadcrumbBadge = breadcrumb.locator(
      `[data-testid="urgency-badge"][data-urgency="${urgency}"]`,
    );

    if (urgency.toLowerCase() === "none") {
      await expect(badge.or(breadcrumbBadge)).toBeHidden();
    } else {
      await expect(badge.or(breadcrumbBadge)).toBeVisible();
    }
  }

  async verifyNoDueDateIndicator(taskTitle: string): Promise<void> {
    const row = this.page
      .locator(`[data-testid="task-item"]`, { hasText: taskTitle })
      .first();
    const breadcrumb = this.page
      .locator(".mantine-Breadcrumbs-root button", { hasText: taskTitle })
      .first();

    // Badges have data-testid="urgency-badge"
    await expect(
      row.locator('[data-testid="urgency-badge"]'),
    ).not.toBeVisible();
    await expect(
      breadcrumb.locator('[data-testid="urgency-badge"]'),
    ).not.toBeVisible();
  }

  async verifyDueDateText(
    taskTitle: string,
    expectedText: string,
  ): Promise<void> {
    const row = this.page
      .locator(`[data-testid="task-item"]`, { hasText: taskTitle })
      .first();
    const breadcrumb = this.page
      .locator(".mantine-Breadcrumbs-root button", { hasText: taskTitle })
      .first();

    const rowText = row.getByText(expectedText, { exact: true });
    const breadcrumbText = breadcrumb.getByText(expectedText, { exact: true });

    await expect(rowText.or(breadcrumbText)).toBeVisible();
  }

  async verifyDueDateTextContains(
    taskTitle: string,
    part: string,
  ): Promise<void> {
    const row = this.page
      .locator(`[data-testid="task-item"]`, { hasText: taskTitle })
      .first();
    const breadcrumb = this.page
      .locator(".mantine-Breadcrumbs-root button", { hasText: taskTitle })
      .first();

    const rowText = row.getByText(part, { exact: false });
    const breadcrumbText = breadcrumb.getByText(part, { exact: false });

    await expect(rowText.or(breadcrumbText)).toBeVisible();
  }

  // --- Sync Settings ---

  async openSyncSettings(): Promise<void> {
    const indicator = this.page.getByTestId("sync-status-button");
    await indicator.click();
    await expect(this.page.getByText("Sync Settings")).toBeVisible();
  }

  async closeSyncSettings(): Promise<void> {
    const indicator = this.page.getByTestId("sync-status-button");
    await indicator.click();
    await expect(this.page.getByText("Sync Settings")).toBeHidden();
  }

  async setSyncServerUrl(url: string): Promise<void> {
    const input = this.page.getByTestId("sync-server-url-input");
    await input.fill(url);
  }

  async saveSyncSettings(): Promise<void> {
    await this.page.getByRole("button", { name: "Save & Reconnect" }).click();
    // Saving reloads the page
    await this.waitForAppReady();
  }

  async verifySyncServerUrl(url: string): Promise<void> {
    const input = this.page.getByTestId("sync-server-url-input");
    await expect(input).toHaveValue(url);
  }

  async downloadDocument(): Promise<string> {
    // Open Settings modal
    await this.page.getByTestId("settings-button").click();

    // Find the modal
    const modal = this.page.getByRole("dialog", { name: "Settings" });
    await expect(modal).toBeVisible();

    // Setup download listener
    const downloadPromise = this.page.waitForEvent("download");

    await modal.getByTestId("download-document-button").click();

    const download = await downloadPromise;
    // Close settings
    await modal.getByTestId("close-settings").click();

    // Get path or safe fallback
    let path = await download.path();
    if (!path) {
      const fs = await import("node:fs"); // Dynamic import to avoid top-level node dep issues if any
      // Use a consistent temp path strategy
      // Playwright usually saves to a temp dir
      const tempDir = (await fs.promises.stat("/tmp").catch(() => null))
        ? "/tmp"
        : ".";
      const tempPath = `${tempDir}/tasklens_download_${Date.now()}.automerge`;
      await download.saveAs(tempPath);
      path = tempPath;
    }

    return path;
  }

  async uploadDocument(filePath: string): Promise<void> {
    await this.waitForAppReady();
    // Add a small delay to ensure hydration/state stability after reload
    await this.page.waitForTimeout(500);

    const modal = this.page.getByRole("dialog", { name: "Settings" });

    for (let i = 0; i < 3; i++) {
      if (await modal.isVisible()) break;

      const btn = this.page.getByTestId("settings-button");
      await btn.waitFor({ state: "visible" });
      await btn.click();

      try {
        await expect(modal).toBeVisible({ timeout: 2000 });
      } catch (_e) {
        console.log(
          `Attempt ${i + 1} to open settings modal failed, retrying...`,
        );
      }
    }

    // Prepare file input - Correct Key!
    const fileInput = modal.locator(
      'input[type="file"][data-testid="document-upload-input"]',
    );
    await expect(fileInput).toBeAttached(); // Input is hidden but attached

    // Upload file
    await this.waitForPersistence(async () => {
      await fileInput.setInputFiles(filePath);
    });

    // Close settings
    await modal.getByTestId("close-settings").click();
    await expect(modal).not.toBeVisible();
    await this.waitForAppReady();
  }

  async getDetailedDocumentUrl(): Promise<string> {
    return await this.page.evaluate(() => {
      // The key defined in crates/tasklens-store/src/storage.rs is "tasklens_active_doc_id"
      const raw = localStorage.getItem("tasklens_active_doc_id");
      if (!raw) return "";
      try {
        return JSON.parse(raw);
      } catch {
        return raw;
      }
    });
  }

  async goto(path = "/"): Promise<void> {
    await this.page.goto(path);
  }

  async evaluate<T>(fn: () => T): Promise<T> {
    return await this.page.evaluate(fn);
  }
}
