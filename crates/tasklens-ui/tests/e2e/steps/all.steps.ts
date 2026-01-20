import { UrgencyStatus } from "@mydoo/tasklens";
import { createBdd } from "playwright-bdd";
import { expect, test } from "../fixtures";
import { formatDateAsISO } from "../utils/date-formatter";
import { durationToMs, parseDuration } from "../utils/duration-parser";

const { Given, When, Then } = createBdd(test);

// --- Common / Setup Steps ---

Given(
  "the user launches the app with a clean slate",
  async ({ page, plan }) => {
    await plan.setupClock();
    await page.goto("/");
    await page.evaluate(() => {
      localStorage.clear();
      return new Promise<void>((resolve, reject) => {
        const req = indexedDB.deleteDatabase("tasklens_db");
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
      });
    });
    await page.reload();
    await plan.setupClock();
    await plan.waitForAppReady();
  },
);

Given("I have a clean workspace", async ({ page, plan }) => {
  await page.goto("/");
  // Wait for the app-provided reset API to be attached
  await page.waitForFunction(
    () =>
      typeof (window as unknown as { tasklensReset?: unknown })
        .tasklensReset === "function",
    {
      timeout: 15000,
    },
  );
  await page.evaluate(async () => {
    interface TaskLensWindow extends Window {
      tasklensReset?: () => Promise<void>;
    }
    const win = window as unknown as TaskLensWindow;
    if (win.tasklensReset) {
      await win.tasklensReset();
    } else {
      throw new Error(
        "tasklensReset not found on window. Ensure the app is running in test mode with WASM enabled.",
      );
    }
  });

  // Since the API no longer reloads internally, we do it here explicitly
  // to start from a fresh, non-checking state with no DB connection.
  await page.goto("/");
  await page.waitForURL("**/", { waitUntil: "networkidle" });
  await plan.setupClock();
  await plan.waitForAppReady();
});

// Added for binary-import-export.feature
Given("I am on the home page", async ({ page, plan }) => {
  // Equivalent to starting clean or just navigating
  await page.goto("/");
  await plan.waitForAppReady();
});

Given("I have created a new document", async ({ plan, documentContext }) => {
  await plan.createNewDocument();
  const id = await plan.getCurrentDocumentId();
  if (!id) throw new Error("Failed to get document ID");
  documentContext.documents.set("original", id);
});

Given(
  "I have a task {string} in the {string} view",
  async ({ plan }, taskTitle, viewName) => {
    if (viewName === "Plan") {
      await plan.switchToPlanView();
    } else {
      await plan.switchToDoView();
    }
    await plan.createTask(taskTitle);
  },
);

Then("I should see the task {string}", async ({ plan }, title) => {
  await plan.verifyTaskVisible(title);
});

Given("I start with a clean workspace", async ({ page, plan }) => {
  await plan.setupClock();
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await plan.setupClock();
  await plan.waitForAppReady();
});

Given("I have a workspace seeded with sample data", async ({ plan }) => {
  await plan.primeWithSampleData();
});

Given("I am on a mobile device", async () => {
  // Viewport is set by the project config (@mobile tag -> bdd-mobile project)
  // We can verify if needed, but usually redundant.
});

// --- Navigation Steps ---

When("I see the welcome screen", async ({ page }) => {
  // Assuming welcome screen just means the app is loaded initially
  await expect(page.locator("#main")).toBeVisible();
});

When("I switch to Plan view", async ({ plan }) => {
  await plan.switchToPlanView();
});

When("I switch to Do view", async ({ plan }) => {
  await plan.switchToDoView();
});

When("I reload the page", async ({ page }) => {
  // Wait for IndexedDB persistence to flush
  await page.waitForTimeout(1500);
  await page.reload();
});

When("I refresh the page", async ({ page }) => {
  await page.waitForTimeout(1500);
  await page.reload();
});

// --- Task Lifecycle Steps ---

When("I create the first task {string}", async ({ plan }, title) => {
  await plan.addFirstTask(title);
});

When("I create a root task {string}", async ({ plan }, title) => {
  await plan.addFirstTask(title);
});

When("I create a task {string}", async ({ plan }, title) => {
  await plan.createTask(title);
});

Given("the user creates a task {string}", async ({ plan }, title: string) => {
  await plan.createTask(title);
});

Then("I should see {string} visible", async ({ plan }, title) => {
  await plan.verifyTaskVisible(title);
});

Then("I should not see {string} visible", async ({ plan }, title) => {
  await plan.verifyTaskHidden(title);
});

When("I rename {string} to {string}", async ({ plan }, oldTitle, newTitle) => {
  await plan.editTaskTitle(oldTitle, newTitle);
});

When(
  "I add a sibling {string} to {string}",
  async ({ plan }, sibling, target) => {
    await plan.addSibling(target, sibling);
  },
);

When("I add a child {string} to {string}", async ({ plan }, child, parent) => {
  await plan.openTaskEditor(parent);
  await plan.addChild(child);
});

When("I complete the task {string}", async ({ plan }, title) => {
  await plan.completeTask(title);
});

Then("I should see {string} marked as completed", async ({ plan }, title) => {
  await plan.verifyTaskCompleted(title);
});

When("I clear completed tasks", async ({ plan }) => {
  await plan.clearCompletedTasks();
});

When("I refresh the Do list", async ({ plan }) => {
  await plan.refreshDoList();
});

When("I create a task {string} in Do view", async ({ plan }, title) => {
  await plan.createTaskInDoView(title);
});

// --- Plan Management Steps ---

When("I expand {string}", async ({ plan }, title) => {
  await plan.toggleExpand(title, true);
});

When("I collapse {string}", async ({ plan }, title) => {
  await plan.toggleExpand(title, false);
});

When("I find {string} in Plan", async ({ plan }, title) => {
  await plan.findInPlan(title);
});

Then("I should see {string} in the Plan view", async ({ plan }, title) => {
  await plan.switchToPlanView();
  await plan.verifyTaskVisible(title);
});

Then("I should be in Plan view", async ({ page }) => {
  // Verify Plan view is active by checking for the "Plan" heading
  await expect(page.getByRole("heading", { name: "Plan" })).toBeVisible();
});

Given(
  "I have a task {string} with child {string}",
  async ({ plan }, parent, child) => {
    await plan.createTask(parent);
    await plan.openTaskEditor(parent);
    await plan.addChild(child);
  },
);

When("I delete {string}", async ({ plan }, title) => {
  await plan.deleteTask(title);
});

// --- Task Moving Steps ---

Given("I have a task {string}", async ({ plan }, title) => {
  await plan.createTask(title);
});

Given(
  "I have a task {string} as a child of {string}",
  async ({ plan }, child, parent) => {
    await plan.openTaskEditor(parent);
    await plan.addChild(child);
  },
);

When("I move {string} to {string}", async ({ plan }, child, target) => {
  await plan.openMovePicker(child);
  await plan.moveTaskTo(target);
});

When("I open the move picker for {string}", async ({ plan }, title) => {
  await plan.openMovePicker(title);
});

Then(
  "I should see {string} disabled or hidden in move picker",
  async ({ plan }, title) => {
    await plan.verifyMovePickerExcludes(title);
  },
);

// --- Task Creation Defaults Steps ---

When("I open the Create Task modal", async ({ page }) => {
  const addTop = page.getByLabel("Add Task at Top");
  if (await addTop.isVisible()) {
    await addTop.click();
  } else {
    // Fallback: Add sibling to first task if "Add Task at Top" isn't visible
    // This handles cases where data is already seeded (Add First Task is gone)
    // and we are on desktop where 'Add Task at Top' might not be available or different.
    // Actually, if we are in Plan view with data, we should use 'Add Sibling' via menu
    // OR find if there is a header button for adding tasks.
    // But for "Task Defaults" test, we just need ANY Create Task modal.

    // Try finding a task and adding a sibling
    const firstTask = page.locator(`[data-testid="task-item"]`).first();
    if (await firstTask.isVisible()) {
      await firstTask.hover();
      await firstTask.getByTestId("task-menu-trigger").click();
      await page.getByRole("menuitem", { name: "Add Sibling" }).click();
    } else {
      await page.getByRole("button", { name: "Add First Task" }).click();
    }
  }
});

Then("I should see {string}", async ({ page }, text) => {
  try {
    await expect(page.getByText(text).first()).toBeVisible({ timeout: 2000 });
  } catch (e) {
    console.log(`DEBUG: Failed to find text "${text}"`);
    throw e;
  }
});

Then(
  "I should see Lead Time {string} {string}",
  async ({ page }, val, unit) => {
    try {
      await expect(page.locator("#lead-time-scalar-input")).toHaveValue(val, {
        timeout: 2000,
      });
      await expect(page.locator("#lead-time-unit-select")).toHaveValue(unit, {
        timeout: 2000,
      });
    } catch (e) {
      console.log(`DEBUG: Failed Lead Time assertion. Expected ${val} ${unit}`);
      const actualVal = await page
        .locator("#lead-time-scalar-input")
        .inputValue();
      const actualUnit = await page
        .locator("#lead-time-unit-select")
        .inputValue();
      console.log(`DEBUG: Actual values: ${actualVal} ${actualUnit}`);
      throw e;
    }
  },
);

Then("I should see the {string} selector", async ({ page }, label) => {
  // Logic: Find a nearby select or role="combobox"
  // Robust approach: Expect a visible element with appropriate role identifiable by the label
  await expect(page.getByLabel(label)).toBeVisible({ timeout: 2000 });
});

When("I add a child to {string}", async ({ plan, page }, title) => {
  await plan.openTaskEditor(title);
  const modal = page.getByRole("dialog", { name: "Edit Task" });
  await modal.getByRole("button", { name: "Add Child" }).click();
});

// --- Mobile Steps ---

Then("I should see the mobile bottom bar", async ({ plan }) => {
  await plan.mobileVerifyMobileBottomBar();
});

When("I drill down into {string}", async ({ plan }, title) => {
  await plan.mobileDrillDown(title);
});

Then("the view title should be {string}", async ({ plan }, title) => {
  await plan.mobileVerifyViewTitle(title);
});

When("I navigate up a level", async ({ plan }) => {
  await plan.mobileNavigateUpLevel();
});

Then("I should see {string} in breadcrumbs", async ({ page }, title) => {
  await expect(page.getByRole("button", { name: title })).toBeVisible();
});

// --- Existing / Other Steps (kept for compatibility) ---

// ... (Keeping existing Document Steps, Routine Steps, etc. that were in the file)

// --- Document Steps ---

Given("the user is on a document", async ({ plan, documentContext }) => {
  await plan.primeWithSampleData();
  const docUrl = await plan.getCurrentDocumentId();
  if (!docUrl) throw new Error("Failed to get document ID");
  documentContext.documents.set("original", docUrl);
});

When("the user creates a new document", async ({ plan }) => {
  await plan.createNewDocument();
});

Then("the document ID changes", async ({ plan, documentContext }) => {
  const newId = await plan.getCurrentDocumentId();
  expect(newId).not.toBe(documentContext.documents.get("original"));
});

When('I click the "Download" button', async ({ plan, documentContext }) => {
  const path = await plan.downloadDocument();
  documentContext.documents.set("downloaded_file_path", path);
});

When("I wait for the file to download", async () => {
  // Handled in clickDownloadButton via waitForEvent("download")
});

When("I clear the application state", async ({ plan }) => {
  // Instead of reloading (which causes flake/rebuild issues), we create a new document.
  // This effectively "clears" the current document from view and changes the ID.
  // When we upload the original, we verify that the ID switches back to the original.
  await plan.createNewDocument();
});

When("I upload the downloaded document", async ({ plan, documentContext }) => {
  const filePath = documentContext.documents.get("downloaded_file_path");
  if (!filePath) throw new Error("No downloaded file path found");
  await plan.uploadDocument(filePath);
});

Then(
  "the current document ID should remain the same",
  async ({ plan, documentContext }) => {
    const currentId = await plan.getCurrentDocumentId();
    const originalId = documentContext.documents.get("original");
    // Extract ID (remove tasklens: prefix if present in stored value,
    // though getCurrentDocumentId returns raw ID usually).
    // Let's assume strict equality for now.
    expect(currentId).toBe(originalId);
  },
);

Then(
  'the document URL should use the "automerge:" schema',
  async ({ plan }) => {
    const url = await plan.getDetailedDocumentUrl();
    expect(url).toMatch(/^automerge:/);
  },
);

Then("the new document is empty", async ({ plan }) => {
  await plan.switchToPlanView();
  await plan.verifyTaskHidden("Project Alpha");
});

Given(
  "a document {string} with task {string}",
  async ({ plan, documentContext }, name: string, task: string) => {
    if (documentContext.documents.size === 0) {
      await plan.primeWithSampleData();
    } else {
      await plan.createNewDocument();
    }

    await plan.createTask(task);
    const docUrl = await plan.getCurrentDocumentId();
    if (!docUrl) throw new Error("Failed to get document URL");
    documentContext.documents.set(name, docUrl);
  },
);

When(
  "the user switches to document {string} by its ID",
  async ({ plan, documentContext }, name: string) => {
    const docUrl = documentContext.documents.get(name);
    if (!docUrl) throw new Error(`Document ${name} not found in context`);
    await plan.switchToDocument(docUrl);
  },
);

Then(
  "the document ID should be the ID of {string}",
  async ({ plan, documentContext }, name: string) => {
    const expectedUrl = documentContext.documents.get(name);
    const actualUrl = await plan.getCurrentDocumentId();
    expect(actualUrl).toBe(expectedUrl);
  },
);

Then("the task {string} should be visible", async ({ plan }, task: string) => {
  await plan.switchToPlanView();
  await plan.verifyTaskVisible(task);
});

// --- Due Date Steps ---

Given(
  "the user creates a task {string} with due date {string} from now and lead time {string}",
  async ({ plan, page }, title, dueStr, leadTimeStr) => {
    await plan.setupClock();
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await plan.setupClock();

    await expect(
      page.locator("nav, footer").getByRole("button", { name: "Plan" }).last(),
    ).toBeVisible();

    const leadDuration = parseDuration(leadTimeStr);
    const now = new Date(await page.evaluate(() => Date.now()));
    const dueMs = durationToMs(dueStr);
    now.setTime(now.getTime() + dueMs);
    const dateString = formatDateAsISO(now);

    await plan.createTaskWithDueDate(title, {
      dueDate: dateString,
      leadTimeVal: leadDuration.value,
      leadTimeUnit: leadDuration.uiUnit,
    });
  },
);

// --- Routine Steps ---

Given(
  "the user creates a routine task {string} repeating every {string} with lead time {string}",
  async ({ plan }, title, repeatStr, leadTimeStr) => {
    await plan.primeWithSampleData();
    await plan.setupClock();

    const repeat = parseDuration(repeatStr);
    const lead = parseDuration(leadTimeStr);

    const freqMap: Record<string, string> = {
      Minutes: "minutes",
      Hours: "hours",
      Days: "daily",
    };
    const freqValue = freqMap[repeat.uiUnit];
    if (!freqValue) {
      throw new Error(
        `Unsupported repeat unit: "${repeat.uiUnit}". Expected: Minutes, Hours, or Days.`,
      );
    }

    await plan.createRoutineTask(title, {
      frequency: freqValue,
      interval: repeat.value,
      leadTimeVal: lead.value,
      leadTimeUnit: lead.uiUnit,
    });
  },
);

Then(
  "the task {string} should be visible in the Do list",
  async ({ plan }, title) => {
    await plan.switchToDoView();
    await plan.verifyTaskVisible(title);
  },
);

When(
  "the user completes the task {string} from the Do list",
  async ({ plan }, title) => {
    await plan.switchToDoView();
    await plan.completeTask(title);
  },
);

Then(
  "the task {string} should be marked as completed in the Do list",
  async ({ plan }, title) => {
    await plan.switchToDoView();
    await plan.verifyTaskCompleted(title);
  },
);

When("the user refreshes the Do list", async ({ plan }) => {
  await plan.switchToDoView();
  await plan.refreshDoList();
});

Then(
  "the task {string} should be hidden in the Do list",
  async ({ plan }, title) => {
    await plan.switchToDoView();
    await plan.verifyTaskHidden(title);
  },
);

When("the user waits {string}", async ({ plan }, durationStr) => {
  const parts = durationStr.split(" ");
  const val = parseInt(parts[0] || "0", 10);
  const unit = parts[1]?.toLowerCase() || "";

  let minutes = 0;
  if (unit.startsWith("minute")) minutes = val;
  else if (unit.startsWith("hour")) minutes = val * 60;
  else if (unit.startsWith("day")) minutes = val * 60 * 24;

  await plan.advanceTime(minutes);
});

// --- Sequential Projects Steps ---

Given(
  "the user marks the task {string} as sequential",
  async ({ plan }, title) => {
    await plan.setSequential(title, true);
  },
);

Given(
  "the user adds a child {string} to {string}",
  async ({ plan }, childTitle, parentTitle) => {
    await plan.switchToPlanView();
    await plan.openTaskEditor(parentTitle);
    await plan.addChild(childTitle);
  },
);

// --- Due Date Feature Steps ---

Given("the current time is {string}", async ({ plan }, isoTime) => {
  await plan.setClock(new Date(isoTime));
});

When(
  "I add a child task {string} to {string}",
  async ({ plan }, childTitle, parentTitle) => {
    await plan.switchToPlanView();
    await plan.openTaskEditor(parentTitle);
    await plan.addChild(childTitle);
    await plan.closeEditor();
  },
);

When(
  "I set the due date of {string} to {string}",
  async ({ plan }, taskTitle, dateStr) => {
    await plan.openTaskEditor(taskTitle);
    await plan.setTaskDueDate(dateStr); // YYYY-MM-DD
    await plan.closeEditor();
  },
);

When(
  "I set the lead time of {string} to {string}",
  async ({ plan }, taskTitle, leadTimeStr) => {
    await plan.openTaskEditor(taskTitle);
    const lead = parseDuration(leadTimeStr);
    await plan.setTaskLeadTime(lead.value, lead.uiUnit);
    await plan.closeEditor();
  },
);

const STATUS_MAP: Record<string, UrgencyStatus> = {
  overdue: UrgencyStatus.Overdue,
  urgent: UrgencyStatus.Urgent,
  active: UrgencyStatus.Active,
  upcoming: UrgencyStatus.Upcoming,
  none: UrgencyStatus.None,
};

// Use regex to avoid conflict with "should be visible"
Then(
  "the task {string} should have urgency {string}",
  async ({ plan }, taskTitle, status) => {
    const urgency = STATUS_MAP[status.toLowerCase()];
    if (!urgency) {
      throw new Error(`Unknown urgency status in step: "${status}"`);
    }
    await plan.verifyTaskUrgency(taskTitle, urgency);
  },
);

Then(
  "the task {string} should be due {string}",
  async ({ plan }, taskTitle, dateText) => {
    if (["Tomorrow", "Yesterday", "Today"].includes(dateText)) {
      await plan.verifyDueDateText(taskTitle, dateText);
    } else {
      await plan.verifyDueDateTextContains(taskTitle, dateText);
    }
  },
);
