@smoke
Feature: Smoke Test

    Scenario: User launches the app
        Given the user launches the app
        Then the page title should contain "TaskLens"
        And the page should have a heading "TaskLens"
