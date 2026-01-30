import { test } from "../fixtures";

test.describe("Field Persistence", () => {
  test("Importance and Effort sliders persist on save", async ({ I }) => {
    const taskTitle = "Slider Persistence Task";

    await I.When.switchToPlanView();
    await I.When.createTask(taskTitle);

    // Action
    await I.When.setsImportance(taskTitle, 0.8);
    await I.When.setsEffort(taskTitle, 0.2);

    // Outcome
    await I.Then.importanceShouldBe(taskTitle, "0.8");
    await I.Then.effortShouldBe(taskTitle, "0.2");
  });

  test("Task notes persist on save", async ({ I }) => {
    const taskTitle = "Notes Persistence Task";
    const taskNotes = "This is a test note that should persist.";

    await I.When.switchToPlanView();
    await I.When.createTask(taskTitle);

    // Action
    await I.When.setsNotes(taskTitle, taskNotes);

    // Outcome
    await I.Then.notesShouldBe(taskTitle, taskNotes);
  });
});
