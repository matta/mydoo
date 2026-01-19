import { expect, test } from "./fixtures";

test("alice and bob should be independent", async ({ alice, bob }) => {
  // 1. Navigate both to home
  await alice.plan.goto();
  await bob.plan.goto();

  // 2. Set distinct storage values
  await alice.page.evaluate(() => localStorage.setItem("user", "Alice"));
  await bob.page.evaluate(() => localStorage.setItem("user", "Bob"));

  // 3. Verify independence
  const aliceUser = await alice.page.evaluate(() =>
    localStorage.getItem("user"),
  );
  const bobUser = await bob.page.evaluate(() => localStorage.getItem("user"));

  expect(aliceUser).toBe("Alice");
  expect(bobUser).toBe("Bob");
});
