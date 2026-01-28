import { test } from "../fixtures";

test.describe("Due Date Indicators and Inheritance", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.cleanWorkspace();
    await I.Given.currentTimeIs("2024-06-01T12:00:00Z");
  });

  test("Overdue task shows overdue status", async ({ I }) => {
    await I.When.createTask("Overdue Task");
    await I.When.setDueDate("Overdue Task", "2024-05-31");
    await I.Then.taskHasUrgency("Overdue Task", "overdue");
  });

  test("Due today task shows urgent status", async ({ I }) => {
    await I.When.createTask("Urgent Task");
    await I.When.setDueDate("Urgent Task", "2024-06-01");
    await I.Then.taskHasUrgency("Urgent Task", "urgent");
  });

  test("Due soon task shows active status", async ({ I }) => {
    await I.When.createTask("Active Task");
    await I.When.setDueDate("Active Task", "2024-06-04");
    await I.When.setLeadTime("Active Task", "7 days");
    await I.Then.taskHasUrgency("Active Task", "active");
  });

  test("Due far future task shows no urgency status", async ({ I }) => {
    await I.When.createTask("Future Task");
    await I.When.setDueDate("Future Task", "2024-06-11");
    await I.When.setLeadTime("Future Task", "7 days");
    await I.Then.taskHasUrgency("Future Task", "none");
  });

  test("Due in lead time window task shows upcoming status", async ({ I }) => {
    await I.When.createTask("Upcoming Task");
    await I.When.setDueDate("Upcoming Task", "2024-06-09");
    await I.When.setLeadTime("Upcoming Task", "7 days");
    await I.Then.taskHasUrgency("Upcoming Task", "upcoming");
  });

  test("Child tasks inherit due dates from ancestors", async ({ I }) => {
    // Parent due tomorrow
    await I.When.createTask("Parent Task");
    await I.When.setDueDate("Parent Task", "2024-06-02");
    await I.When.setLeadTime("Parent Task", "7 days");

    // Child with no date
    await I.When.addsChildTask("Child Task", "Parent Task");
    await I.Then.taskHasUrgency("Child Task", "urgent");
    await I.Then.taskIsDue("Child Task", "Tomorrow");

    // Grandchild with no date
    await I.When.addsChildTask("Grandchild Task", "Child Task");
    await I.Then.taskHasUrgency("Grandchild Task", "urgent");
    await I.Then.taskIsDue("Grandchild Task", "Tomorrow");
  });

  test("Child tasks override inherited dates", async ({ I }) => {
    await I.When.createTask("Parent Task");
    await I.When.setDueDate("Parent Task", "2024-06-02");
    await I.When.setLeadTime("Parent Task", "30 days");

    await I.When.addsChildTask("Child Task", "Parent Task");
    await I.When.setDueDate("Child Task", "2024-06-05");

    await I.Then.taskIsDue("Parent Task", "Tomorrow");
  });

  test("Child tasks inherit far future due dates", async ({ I }) => {
    await I.When.createTask("Parent Task");
    await I.When.setDueDate("Parent Task", "2124-12-31");
    await I.When.addsChildTask("Child Task", "Parent Task");

    await I.Then.taskIsDue("Parent Task", "Dec 31, 2124");
    await I.Then.taskIsDue("Child Task", "Dec 31, 2124");
  });
});
