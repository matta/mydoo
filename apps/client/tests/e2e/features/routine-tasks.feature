@skip-mobile
Feature: Routine Tasks

  Scenario: Routine task reappears after lead time
    Given the user creates a routine task "Water Plants" repeating every "1 days" with lead time "12 hours"
    Then the task "Water Plants" should be visible in the Do list
    When the user completes the task "Water Plants" from the Do list
    Then the task "Water Plants" should be marked as completed in the Do list
    When the user refreshes the Do list
    Then the task "Water Plants" should be hidden in the Do list
    When the user waits "14 hours"
    And the user refreshes the Do list
    Then the task "Water Plants" should be visible in the Do list
