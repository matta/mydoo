Feature: Due Dates

  Scenario: Task with distant due date is hidden from Do list
    Given the user creates a task "Future Task" with due date "3 days" from now and lead time "1 day"
    Then the task "Future Task" should be hidden in the Do list

  Scenario: Task within lead time window is visible in Do list
    Given the user creates a task "Immediate Task" with due date "1 day" from now and lead time "1 day"
    Then the task "Immediate Task" should be visible in the Do list
