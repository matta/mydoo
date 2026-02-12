import { test } from "../fixtures";

test.use({ db: "sample" });

test.describe("Task Search", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.seededWithSampleData();
  });

  test("Search panel opens and closes", async ({ I }) => {
    // When
    await I.When.opensSearch();

    // Then
    await I.Then.searchPanelIsOpen();

    // When
    await I.When.closesSearch();

    // Then
    await I.Then.searchPanelIsClosed();
  });

  test("Search input is focused when panel opens", async ({ I }) => {
    // When
    await I.When.opensSearch();

    // Then
    await I.Then.searchInputIsFocused();
  });

  test("Search finds tasks by title", async ({ I }) => {
    // When
    await I.When.searchesForTask("Project Alpha");

    // Then
    await I.Then.searchResultIsVisible("Project Alpha");
  });

  test("Search is case-insensitive", async ({ I }) => {
    // When
    await I.When.searchesForTask("project alpha");

    // Then
    await I.Then.searchResultIsVisible("Project Alpha");
  });

  test("Search shows no results for non-matching query", async ({ I }) => {
    // When
    await I.When.searchesForTask("xyznonexistent");

    // Then
    await I.Then.noSearchResults();
  });

  test("Search shows breadcrumb path for nested tasks", async ({ I }) => {
    // When
    await I.When.searchesForTask("Research Requirements");

    // Then
    await I.Then.searchResultIsVisible("Research Requirements");
    await I.Then.searchResultShowsPath(
      "Research Requirements",
      "Project Alpha",
    );
  });

  test("Clicking search result navigates to Plan with task focused", async ({
    I,
  }) => {
    // Given
    await I.When.switchToDoView();

    // When
    await I.When.searchesForTask("Research Requirements");
    await I.When.clicksSearchResult("Research Requirements");

    // Then
    await I.Then.shouldBeInPlanView();
    await I.Then.taskIsVisible("Research Requirements");
  });

  test("Search finds completed tasks", async ({ I }) => {
    // Given
    await I.When.completesTask("Buy Groceries");

    // When
    await I.When.searchesForTask("Buy Groceries");

    // Then
    await I.Then.searchResultIsVisible("Buy Groceries");
  });

  test("Search panel closes after selecting a result", async ({ I }) => {
    // When
    await I.When.searchesForTask("Project Alpha");
    await I.When.clicksSearchResult("Project Alpha");

    // Then
    await I.Then.searchPanelIsClosed();
  });

  test("Escape key closes search panel", async ({ I, page }) => {
    // When
    await I.When.opensSearch();
    await I.Then.searchInputIsFocused();
    await page.keyboard.press("Escape");

    // Then
    await I.Then.searchPanelIsClosed();
  });

  test("Escape key closes search panel even if input is not focused", async ({
    I,
    page,
  }) => {
    // When
    await I.When.opensSearch();
    // Blur the input to ensure it's not focused
    await page.getByTestId("search-input").blur();
    await page.keyboard.press("Escape");

    // Then
    await I.Then.searchPanelIsClosed();
  });
});
