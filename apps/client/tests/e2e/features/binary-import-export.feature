@binary
Feature: Binary Import/Export

  Background:
    Given I have a workspace seeded with sample data
    And I create a task "My Important Task"

  Scenario: Export and Import Binary Document
    When I export the document as binary
    And the user creates a new document
    Then the document ID changes
    When I import the binary document
    And I switch to Plan view
    Then I should see "My Important Task" visible
    And the document URL should be preserved
