@migration-pending
Feature: Sequential Projects

    Background:
        Given the user launches the app with a clean slate

    Scenario: Sequential tasks are blocked until previous sibling is done
        Given the user creates a task "Project Alpha"
        And the user marks the task "Project Alpha" as sequential
        And the user adds a child "Step 1" to "Project Alpha"
        And the user adds a child "Step 2" to "Project Alpha"
        Then the task "Step 1" should be visible in the Do list
        And the task "Step 2" should be hidden in the Do list
        When the user completes the task "Step 1" from the Do list
        And the user refreshes the Do list
        Then the task "Step 2" should be visible in the Do list
