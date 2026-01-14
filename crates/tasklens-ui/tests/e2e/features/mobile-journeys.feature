@migration-pending
Feature: Mobile Journeys
    As a mobile user
    I want to navigate and manage tasks on my phone
    So that I can be productive on the go

    Background:
        Given I am on a mobile device
        And I have a workspace seeded with sample data
        And I switch to Plan view

    @mobile
    Scenario: Mobile Smoke Test
        Then I should see the mobile bottom bar

    @mobile
    Scenario: Add Child via Drill Down
        When I drill down into "Deep Work Project"
        Then the view title should be "Deep Work Project"

        When I create a task "Drill Child"
        Then I should see "Drill Child" visible

        When I navigate up a level
        Then I should see the mobile bottom bar

    @mobile
    Scenario: Deep drill-down navigation
        When I drill down into "Deep Work Project"
        And I drill down into "Module A"
        And I drill down into "Component X"
        Then I should see "Deep Work Project" in breadcrumbs
        And I should see "Module A" in breadcrumbs
        And I should see "Component X" in breadcrumbs
