Feature: Task Creation Defaults
    As a user
    I want new tasks to have sensible default values
    So that I save time on data entry

    Background:
        Given I have a workspace seeded with sample data
        And I switch to Plan view

    Scenario: New tasks have correct default values
        When I open the Create Task modal
        Then I should see "Importance: 1.00"
        And I should see Lead Time "8" "Hours"

    Scenario: Child tasks inherit defaults correctly
        Given I have a task "Root Task for Defaults"
        When I add a child to "Root Task for Defaults"
        Then I should see "Importance: 1.00"
        And I should see Lead Time "8" "Hours"
