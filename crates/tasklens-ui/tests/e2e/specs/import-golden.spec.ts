import * as path from "path";
import { test } from "../fixtures";

test.describe("Binary Doc Import", () => {
  test.beforeEach(async ({ I }) => {
    await I.Given.cleanWorkspace();
  });

  test("Import Golden File", async ({ I }) => {
    // Determine the absolute path to the golden file
    // The original feature used: "../tasklens-store/tests/data/golden.automerge"
    // We need to resolve this relative to the project root or use a known path structure.
    // Assuming the test runs from module root, we can try to resolve it.
    // Better yet, let's assume we can resolve it relative to this file or just pass a relative path that the step handles.
    // The step library implementation of `uploadsDocument` should handle path resolution if needed,
    // but typically Playwright's setInputFiles handles relative paths or we pass absolute.
    // Let's pass a relative path that works from the repo root if possible, or construct an absolute one.
    // Since we don't easily know the absolute repo root at runtime without some config,
    // let's assume we are running from `crates/tasklens-ui`.
    // Then `../tasklens-store/tests/data/golden.automerge` is correct relative to CWD.
    const goldenFilePath = path.resolve(
      "..",
      "tasklens-store",
      "tests",
      "data",
      "golden.automerge",
    );

    // When I upload the document
    await I.When.uploadsDocument(goldenFilePath);

    // And I expand the task "Household"
    await I.When.expandsTask("Household");

    // Then I see the task "Install the mailbox"
    await I.Then.taskIsVisible("Install the mailbox");
  });
});
