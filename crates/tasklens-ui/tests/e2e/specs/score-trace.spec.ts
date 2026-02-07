import { test } from "../fixtures";

/** Verifies the Do view score trace workflow. */
test.describe("Score Trace", () => {
  test("Do list score links to score trace breakdown", async ({ I }) => {
    await I.Given.currentTimeIs("2024-06-01T12:00:00Z");
    await I.When.createTask("Trace Task");
    await I.When.setDueDate("Trace Task", "2024-06-11");
    await I.When.setLeadTime("Trace Task", "7 days");
    await I.When.switchToDoView();
    await I.Then.doTaskShowsScore("Trace Task");
    await I.When.opensScoreTraceForTask("Trace Task");
    await I.Then.scoreTraceShowsBreakdown("Trace Task");
    await I.Then.scoreTraceShowsLeadTimeStage("Ramping");
  });
});
