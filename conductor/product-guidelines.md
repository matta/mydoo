# Product Guidelines: MyDoo

## Voice and Tone

- **Functional & Direct:** Communication within the application and its
  documentation should be clear, concise, and focused on utility. Use direct
  language to describe actions and states (e.g., "Task completed," "Priority
  updated").
- **Professional & Utility-Focused:** Avoid unnecessary fluff or overly casual
  language. The goal is to provide information quickly and efficiently to the
  user.

## Visual Identity & UX

- **Toolkit-Driven Design:** Utilize the default styles and components provided
  by the underlying UI toolkit (currently Mantine, with a potential move to
  MUI). Maintain consistency with the toolkit's design language.
- **Minimalist Leanings:** While following toolkit defaults, aim for a
  minimalist aesthetic that prioritizes readability and data clarity. Reduce
  cognitive load by avoiding cluttered interfaces.
- **Platform-Specific Navigation:**
  - **Mobile:** Prioritize single-handed operation with bottom-tab navigation
    and a drill-down approach for hierarchical views (Plan view) to maintain
    focus on smaller screens.
  - **Desktop:** Utilize the available screen real estate for a comprehensive
    view, such as a split-pane layout (Plan | Do), avoiding the drill-down
    constraint.
  - **Both:** Treat both mobile and desktop experiences as critically important
    and first-class citizens.

## Error Handling & Feedback

- **User-Centric & Actionable:** Error messages should be clear and
  non-technical. Explain what happened and, more importantly, suggest a clear
  next step for the user (e.g., "Sync failed. Please check your internet
  connection and try again.").
- **Immediate Feedback:** Provide clear visual confirmation for user actions
  (e.g., a task row flashing yellow upon creation or a checkmark appearing when
  completed).

## Philosophy Integration

- **Subtle Strategic Balance Indicators:** Reflect the core philosophy through
  clear visual data, such as the actual vs. target effort bars in the Balance
  view. Let the data speak for itself rather than providing explicit
  philosophical commentary within the UI.
- **Trust the Algorithm:** The application should empower the user by presenting
  a prioritized list based on the Prioritization Algorithm, reinforcing
  "Autofocus" and "Goal Alignment" principles through the ordering of tasks.

## Testing Philosophy

- **Test-Driven Development (TDD):** Adopt a TDD approach where tests are
  written before implementation to drive design and ensure correctness.
- **Full E2E Coverage:** Maintain comprehensive End-to-End (E2E) test coverage
  for all critical user stories.
- **Tooling:** Utilize `playwright-bdd` (Playwright with BDD/Gherkin syntax) to
  express and verify these user stories in a clear, readable format.
