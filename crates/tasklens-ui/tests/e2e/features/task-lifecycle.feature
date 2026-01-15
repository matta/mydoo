Feature: Task Lifecycle
    As a user
    I want to manage my tasks through their entire lifecycle
    So that I can organize my work and track progress

    Background:
        Given I start with a clean workspace

    Scenario: Full Desktop Journey
        When I see the welcome screen
        And I switch to Plan view
        And I create the first task "Desktop Root"
        Then I should see "Desktop Root" visible

        When I rename "Desktop Root" to "Desktop Root Renamed"
        Then I should see "Desktop Root Renamed" visible
        And I should not see "Desktop Root" visible

        When I add a sibling "Desktop Sibling" to "Desktop Root Renamed"
        Then I should see "Desktop Root Renamed" visible
        And I should see "Desktop Sibling" visible

        When I add a child "Desktop Child" to "Desktop Root Renamed"
        Then I should see "Desktop Child" visible

    Scenario: Basic Create and Complete
        When I create a task "New E2E Task"
        Then I should see "New E2E Task" visible

        When I complete the task "New E2E Task"
        Then I should see "New E2E Task" marked as completed

        When I clear completed tasks
        Then I should not see "New E2E Task" visible

    Scenario: Completed tasks stay visible until refresh
        When I create a task "Remediation Task" in Do view
        Then I should see "Remediation Task" visible

        When I complete the task "Remediation Task"
        Then I should see "Remediation Task" visible

        When I refresh the Do list
        Then I should not see "Remediation Task" visible
