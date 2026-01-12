import { UrgencyStatus } from "@mydoo/tasklens";
import { createBdd } from "playwright-bdd";
import { formatDateAsISO } from "../../../src/test/utils/date-formatter";
import {
  durationToMs,
  parseDuration,
} from "../../../src/test/utils/duration-parser";
import { expect, test } from "../fixtures";

const { Given, When, Then } = createBdd(test);

// --- Common Steps ---

Given(
  "the user launches the app with a clean slate",
  async ({ page, plan }) => {
    await plan.setupClock();
    await page.goto("/");
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await plan.setupClock();
  },
);

Given("the user creates a task {string}", async ({ plan }, title: string) => {
  await plan.createTask(title);
});

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
    await plan.setupClock();
    await plan.primeWithSampleData();

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

When("I create a task {string}", async ({ plan }, title) => {
  // Navigate to Plan view if not there
  await plan.switchToPlanView();
  await plan.createTask(title);
});

When(
  "I add a child task {string} to {string}",
  async ({ plan }, childTitle, parentTitle) => {
    await plan.switchToPlanView();
    await plan.openTaskEditor(parentTitle);
    await plan.addChild(childTitle);
    // Close editor to return to list? Or stay?
    // addChild usually stays in editor or returns.
    // Assuming we need to close to see the list update or verify indicators.
    // Let's ensure we close it.
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

Given("I have a clean workspace", async ({ page, plan }) => {
  await plan.setupClock();
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await plan.setupClock();
});

Then(
  "the task {string} should be due {string}",
  async ({ plan }, taskTitle, dateText) => {
    // Use verifyDueDateTextContains to be robust against "Jun 5" vs "Jun 5, 2024" if needed,
    // but the scenario says "should be due 'Tomorrow'".
    // If the dateText is short ("Tomorrow", "Yesterday"), exact match is better.
    // If it is "Jun 5", exact match might be "Jun 5".
    // Let's us contains for robustness if it's a date string.
    if (["Tomorrow", "Yesterday", "Today"].includes(dateText)) {
      await plan.verifyDueDateText(taskTitle, dateText);
    } else {
      await plan.verifyDueDateTextContains(taskTitle, dateText);
    }
  },
);
