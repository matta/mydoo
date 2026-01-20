@e2e
Feature: Document Binary Import/Export
  As a user
  I want to export my document as a binary file and import it back
  So that I can backup and restore my data while preserving identity

  Background:
    Given I am on the home page
    And I have created a new document

  Scenario: Export and Import Preserves Document Identity
    Given I have a task "Medieval Quest" in the "Plan" view
    When I click the "Download" button
    And I wait for the file to download
    And I clear the application state
    And I upload the downloaded document
    Then I should see the task "Medieval Quest"
    And the current document ID should remain the same
    And the document URL should use the "automerge:" schema
