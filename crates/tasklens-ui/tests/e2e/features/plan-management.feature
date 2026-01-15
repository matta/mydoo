@plan-management
Feature: Plan Management
    As a user
    I want to manage my task hierarchy in Plan view
    So that I can structure my projects effectively

    Background:
        Given I have a workspace seeded with sample data

    Scenario: Render task hierarchy
        When I switch to Plan view
        Then I should see "Project Alpha" visible
        And I should see "Buy Groceries" visible
        And I should not see "Research Requirements" visible

        When I expand "Project Alpha"
        Then I should see "Research Requirements" visible
        And I should see "Design UI Mocks" visible

    @migration-pending
    Scenario: Find in Plan from Do view
        When I switch to Do view
        Then I should see "Research Requirements" visible

        When I find "Research Requirements" in Plan
        Then I should be in Plan view
        And I should see "Project Alpha" visible
        And I should see "Research Requirements" visible
        And I should see "Design UI Mocks" visible

    Scenario: Edit task properties and persist
        When I create a task "Task to Edit"
        And I rename "Task to Edit" to "Edited Task Title"
        Then I should see "Edited Task Title" visible
        And I should not see "Task to Edit" visible

        When I reload the page
        And I switch to Plan view
        Then I should see "Edited Task Title" visible

    Scenario: Delete task with cascade
        Given I have a task "Parent Task" with child "Child Task"
        When I delete "Parent Task"
        Then I should not see "Parent Task" visible
        And I should not see "Child Task" visible

    @plan-management @migration-pending
    Scenario: Persist data across reloads
        When I create a task "Persistent Task"
        And I reload the page
        Then I should see "Persistent Task" in the Plan view
        Then I should see "Persistent Task" visible
