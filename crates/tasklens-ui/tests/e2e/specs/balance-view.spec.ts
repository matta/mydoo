import { test } from "../fixtures";

test.describe("Balance View", () => {
  test("Shows empty state when no root goals exist", async ({ I }) => {
    await I.When.switchToBalanceView();
    await I.Then.balanceViewIsEmpty();
  });

  test("Displays balance items for root goals excluding Inbox", async ({
    I,
  }) => {
    // Create root-level goals
    await I.When.switchToPlanView();
    await I.When.createTask("Health");
    await I.When.createTask("Career");
    await I.When.createTask("Inbox");

    // Navigate to Balance
    await I.When.switchToBalanceView();

    // Verify only non-Inbox goals appear
    await I.Then.balanceItemIsVisible("Health");
    await I.Then.balanceItemIsVisible("Career");
    await I.Then.balanceItemCount(2);
  });

  test("Adjusting desired credits changes task priority in Do view", async ({
    I,
  }) => {
    // Setup: Create two root goals with actionable child tasks
    await I.When.switchToPlanView();
    await I.When.createTask("Health");
    await I.When.addsChild("Health", "Exercise");

    await I.When.switchToPlanView();
    await I.When.createTask("Career");
    await I.When.addsChild("Career", "Write Report");

    // Initial state: both goals have equal desired_credits (default 1.0)
    // Tasks should appear in outline order (Exercise before Write Report)
    await I.When.switchToDoView();
    await I.Then.taskIsVisible("Exercise");
    await I.Then.taskIsVisible("Write Report");

    // Now boost Career's desired credits significantly in Balance view
    // This increases Career's FeedbackFactor, making its tasks higher priority
    await I.When.switchToBalanceView();
    await I.Then.balanceItemIsVisible("Health");
    await I.Then.balanceItemIsVisible("Career");

    // Set Career's desired credits much higher than Health's
    // Health default: 1.0, Career: set to 9.0
    // This makes Career's TargetPercent = 9/(1+9) = 90%
    // With no credits earned yet, Career is "starving" and gets boosted
    await I.When.adjustsDesiredCredits("Career", 9.0);

    // Verify Career is now marked as starving (wants 90% but has ~50%)
    await I.Then.balanceItemIsStarving("Career");

    // Check Do view: Career's task should now appear before Health's task
    // because Career's FeedbackFactor is now higher
    await I.Then.taskAppearsBeforeInDoList("Write Report", "Exercise");
  });

  test("Completing tasks affects balance status", async ({ I }) => {
    // Setup: Two goals with child tasks
    await I.When.switchToPlanView();
    await I.When.createTask("Health");
    await I.When.addsChild("Health", "Morning Run");

    await I.When.switchToPlanView();
    await I.When.createTask("Career");
    await I.When.addsChild("Career", "Team Meeting");

    // Set equal desired credits
    await I.When.switchToBalanceView();
    await I.When.adjustsDesiredCredits("Health", 5.0);
    await I.When.adjustsDesiredCredits("Career", 5.0);

    // Initially both are "starving" because no credits have been earned yet
    // (actual_percent = 0, target_percent = 0.5, so actual < target)
    await I.Then.balanceItemIsStarving("Health");
    await I.Then.balanceItemIsStarving("Career");

    // Complete Health task to earn credits for Health
    await I.When.switchToDoView();
    await I.When.completesTask("Morning Run");
    await I.When.refreshesDoList();

    // Check Balance: After completing Health's task:
    // - Health now has 100% of actual credits (only credits earned so far)
    // - Career still has 0% actual credits
    // - Health: target=50%, actual=100% -> Balanced (actual >= target)
    // - Career: target=50%, actual=0% -> Starving (actual < target)
    await I.When.switchToBalanceView();
    await I.Then.balanceItemIsStarving("Career");
    await I.Then.balanceItemIsBalanced("Health");
  });
});
