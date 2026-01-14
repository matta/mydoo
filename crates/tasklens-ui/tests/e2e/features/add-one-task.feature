Feature: Add One Task

    @add-one-task
    Scenario: Add one task
        Given I start with a clean workspace
        When I switch to Do view
        And I create a task "Buy Groceries" in Do view
        Then I should see "Buy Groceries" visible
