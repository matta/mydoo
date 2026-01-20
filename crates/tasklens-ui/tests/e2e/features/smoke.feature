@smoke
Feature: Smoke Test

    Scenario: User launches the app
        Given I start with a clean workspace
        Then the page title should contain "TaskLens"
