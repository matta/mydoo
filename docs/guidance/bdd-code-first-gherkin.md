# Engineering Standard: Code-First Gherkin (Strict Step-Object Pattern)

## 1. Concept Overview

The **Strict Step-Object Pattern** replaces physical `.feature` files and
Gherkin parsers with strictly typed TypeScript fixtures. It retains the
**rigor** and **semantics** of BDD (Behavior Driven Development) while
leveraging the **compile-time safety** and **refactoring tools** of the native
language.

In this pattern, "Steps" are not regex-matched strings; they are class methods
organized by Gherkin phase (`Given`, `When`, `Then`).

### Comparison

| Feature             | Classic Gherkin (Cucumber/Playwright-BDD)            | Code-First Gherkin (Step-Object)              |
| :------------------ | :--------------------------------------------------- | :-------------------------------------------- |
| **Source of Truth** | `.feature` text file                                 | `.spec.ts` TypeScript file                    |
| **Step Matching**   | Regex / String matching at runtime                   | Static method calls at compile time           |
| **Parameters**      | Extracted from strings (weakly typed)                | Passed as function arguments (strongly typed) |
| **Navigation**      | Requires VS Code plugins                             | Native "Go to Definition" (F12)               |
| **Maintenance**     | Renaming requires updating feature + step definition | Renaming is a single symbol refactor          |

---

## 2. Style Guide & Conventions

### 2.1. The Spec File Structure

Test files (`.spec.ts`) must read like user stories. They should contain
**zero** Playwright implementation details (no `page.locator`, no `expect`
assertions directly in the test body).

- **`test.describe`**: Represents the **Feature**.
- **`test`**: Represents the **Scenario**.
- **`I`**: The simplified actor fixture that exposes the Gherkin API.

**Example:**

```typescript
test.describe("Due Date Indicators", () => {
  test("Overdue task shows overdue status", async ({ I }) => {
    // Context
    await I.Given.cleanWorkspace();
    await I.Given.currentTimeIs("2024-06-01T12:00:00Z");

    // Action
    await I.When.createTask("Overdue Task");
    await I.When.setDueDate("Overdue Task", "2024-05-31");

    // Outcome
    await I.Then.taskHasUrgency("Overdue Task", "overdue");
  });
});
```

### 2.2. Naming Conventions

Step methods must be namespaced under `Given`, `When`, or `Then` objects within
the main actor.

- **`Given.*`**: Sets up state. Must be idempotent where possible.
  - _Bad:_ `I.Given.login()`
  - _Good:_ `I.Given.userIsLoggedIn()`
- **`When.*`**: Performs an action. Use active verbs.
  - _Bad:_ `I.When.clickSave()` (Implementation detail)
  - _Good:_ `I.When.saveChanges()` (User intent)
- **`Then.*`**: Asserts state. Must contain assertions.
  - _Bad:_ `I.Then.checkElement()`
  - _Good:_ `I.Then.taskShouldBeVisible()`

### 2.3. Atomicity

Each step method should map to one logical user action or assertion. Avoid "God
Steps" that do everything, but also avoid "Click Steps" that are too granular.

- _Too Granular:_ `When.clickInput()`, `When.typeText()`, `When.blur()`
- _Just Right:_ `When.fillTaskDetails(title, date)`

---

## 3. Implementation Reference

To adopt this pattern, we extend the Playwright test fixture to include an `I`
object. This object wraps your existing Page Objects (like `PlanPage`).

### 3.1. The Actor Fixture (`fixtures.ts`)

```typescript
import { test as base, expect } from "@playwright/test";
import { PlanPage } from "./pages/plan-page";
import { Steps } from "./steps/steps-library";

// 1. Define the Fixture Type
type BddFixtures = {
  plan: PlanPage;
  I: Steps;
};

// 2. Extend the Test
export const test = base.extend<BddFixtures>({
  plan: async ({ page }, use) => {
    await use(new PlanPage(page));
  },

  // The "I" object aggregates all Gherkin semantics
  I: async ({ plan, page }, use) => {
    await use(new Steps(plan, page));
  },
});
```

### 3.2. The Steps Library (`steps/steps-library.ts`)

This class acts as the "Translation Layer" between Gherkin semantics and your
Page Objects (`PlanPage`).

```typescript
import { expect, type Page } from "@playwright/test";
import { PlanPage } from "../pages/plan-page";

export class Steps {
  constructor(
    private plan: PlanPage,
    private page: Page,
  ) {}

  // Namespaced "Given" methods
  public Given = {
    cleanWorkspace: async () => {
      await this.plan.setupClock();
      await this.page.goto("/");
      await this.page.evaluate(() => localStorage.clear());
      await this.page.reload();
      await this.plan.setupClock();
    },

    currentTimeIs: async (isoDate: string) => {
      await this.plan.setClock(new Date(isoDate));
    },

    documentExists: async (name: string) => {
      // Implementation using this.plan...
    },
  };

  // Namespaced "When" methods
  public When = {
    createTask: async (title: string) => {
      await this.plan.switchToPlanView();
      await this.plan.createTask(title);
    },

    setDueDate: async (taskTitle: string, dateStr: string) => {
      await this.plan.openTaskEditor(taskTitle);
      await this.plan.setTaskDueDate(dateStr);
      await this.plan.closeEditor();
    },

    completesTask: async (title: string) => {
      await this.plan.switchToDoView();
      await this.plan.completeTask(title);
    },
  };

  // Namespaced "Then" methods
  public Then = {
    taskHasUrgency: async (title: string, urgency: string) => {
      await this.plan.verifyTaskUrgency(title, urgency);
    },

    taskIsVisible: async (title: string) => {
      await this.plan.verifyTaskVisible(title);
    },

    documentIdChanges: async (oldId: string) => {
      const newId = await this.plan.getCurrentDocumentId();
      expect(newId).not.toBe(oldId);
    },
  };
}
```

---

## 4. Migration Example

Here is how the existing `sequential-projects.feature` transforms into
`sequential-projects.spec.ts`.

**Original Gherkin:**

```gherkin
Scenario: Sequential tasks are blocked until previous sibling is done
    Given the user creates a task "Project Alpha"
    And the user marks the task "Project Alpha" as sequential
    And the user adds a child "Step 1" to "Project Alpha"
    Then the task "Step 1" should be visible in the Do list
```

**New Code-First Spec:**

```typescript
import { test } from "../fixtures";

test.describe("Sequential Projects", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.cleanWorkspace();
  });

  test("Sequential tasks are blocked until previous sibling is done", async ({
    I,
  }) => {
    // Given
    await I.Given.taskExists("Project Alpha");
    await I.Given.taskIsSequential("Project Alpha");
    await I.Given.taskHasChild("Project Alpha", "Step 1");
    await I.Given.taskHasChild("Project Alpha", "Step 2");

    // Then (Initial State)
    await I.Then.taskIsVisibleInDoList("Step 1");
    await I.Then.taskIsHiddenInDoList("Step 2");

    // When (Action)
    await I.When.completesTask("Step 1");
    await I.When.refreshesDoList();

    // Then (Final State)
    await I.Then.taskIsVisibleInDoList("Step 2");
  });
});
```

## 5. Conversion Roadmap

The following legacy `.feature` files in `crates/tasklens-ui/tests/e2e/features`
are slated for conversion to code-first specs.

### Conversion Steps

1.  **Create Spec**: Create a new file in `crates/tasklens-ui/tests/e2e/specs/`
    named `<feature-name>.spec.ts`.
2.  **Implement Steps**: Translate Gherkin scenarios into `test()` calls using
    the `I` actor fixture.
3.  **Delete Feature**: Once the spec is passing, delete the `.feature` file.
4.  **Cleanup**: If a step was unique to that feature and is no longer needed in
    `all.steps.ts`, it can eventually be removed from the legacy system.

### Checklist

- [ ] `binary-import-export.feature` → `specs/binary-import-export.spec.ts`
- [ ] `document-switching.feature` → `specs/document-switching.spec.ts`
- [ ] `due-dates.feature` → `specs/due-dates.spec.ts`
- [ ] `mobile-journeys.feature` → `specs/mobile-journeys.spec.ts`
- [ ] `plan-management.feature` → `specs/plan-management.spec.ts`
- [ ] `routine-tasks.feature` → `specs/routine-tasks.spec.ts`
- [ ] `sequential-projects.feature` → `specs/sequential-projects.spec.ts`
- [ ] `smoke.feature` → `specs/smoke.spec.ts`
- [ ] `task-creation.feature` → `specs/task-creation.spec.ts`
- [ ] `task-lifecycle.feature` → `specs/task-lifecycle.spec.ts`
- [ ] `task-moving.feature` → `specs/task-moving.spec.ts`
