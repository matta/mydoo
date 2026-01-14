import { createBdd } from "playwright-bdd";
import { expect, test } from "../fixtures";

const { Given, Then } = createBdd(test);

Given("the user launches the app", async ({ page }) => {
  await page.goto("/");
});

Then("the page title should contain {string}", async ({ page }, title) => {
  await expect(page).toHaveTitle(new RegExp(title));
});

Then("the page should have a heading {string}", async ({ page }, heading) => {
  await expect(page.getByRole("heading", { name: heading })).toBeVisible();
});
