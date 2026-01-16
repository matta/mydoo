@due-dates
Feature: Due Date Indicators and Inheritance

    Background:
        Given I have a clean workspace
        Given the current time is "2024-06-01T12:00:00Z"

    Scenario: Overdue task shows overdue status
        When I create a task "Overdue Task"
        And I set the due date of "Overdue Task" to "2024-05-31"
        Then the task "Overdue Task" should have urgency "overdue"

    Scenario: Due today task shows urgent status
        When I create a task "Urgent Task"
        And I set the due date of "Urgent Task" to "2024-06-01"
        Then the task "Urgent Task" should have urgency "urgent"

    Scenario: Due soon task shows active status
        When I create a task "Active Task"
        And I set the due date of "Active Task" to "2024-06-04"
        And I set the lead time of "Active Task" to "7 days"
        Then the task "Active Task" should have urgency "active"

    Scenario: Due far future task shows no urgency status
        When I create a task "Future Task"
        And I set the due date of "Future Task" to "2024-06-11"
        And I set the lead time of "Future Task" to "7 days"
        Then the task "Future Task" should have urgency "none"

    Scenario: Due in lead time window task shows upcoming status
        When I create a task "Upcoming Task"
        And I set the due date of "Upcoming Task" to "2024-06-09"
        And I set the lead time of "Upcoming Task" to "7 days"
        Then the task "Upcoming Task" should have urgency "upcoming"

    Scenario: Child tasks inherit due dates from ancestors

        # Parent due tomorrow
        When I create a task "Parent Task"
        And I set the due date of "Parent Task" to "2024-06-02"
        And I set the lead time of "Parent Task" to "7 days"

        # Child with no date
        And I add a child task "Child Task" to "Parent Task"
        Then the task "Child Task" should have urgency "urgent"
        And the task "Child Task" should be due "Tomorrow"

        # Grandchild with no date
        When I add a child task "Grandchild Task" to "Child Task"
        Then the task "Grandchild Task" should have urgency "urgent"
        And the task "Grandchild Task" should be due "Tomorrow"

    Scenario: Child tasks override inherited dates

        When I create a task "Parent Task"
        And I set the due date of "Parent Task" to "2024-06-02"
        And I set the lead time of "Parent Task" to "30 days"

        When I add a child task "Child Task" to "Parent Task"
        And I set the due date of "Child Task" to "2024-06-05"

        Then the task "Parent Task" should be due "Tomorrow"

    Scenario: Child tasks inherit far future due dates
        When I create a task "Parent Task"
        And I set the due date of "Parent Task" to "2124-12-31"
        And I add a child task "Child Task" to "Parent Task"

        Then the task "Parent Task" should be due "Dec 31, 2124"
        And the task "Child Task" should be due "Dec 31, 2124"
