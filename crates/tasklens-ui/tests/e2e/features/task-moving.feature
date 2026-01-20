@task-moving
Feature: Task Moving
    As a user
    I want to move tasks to different parents
    So that I can reorganize my plans

    Background:
        Given I have a workspace seeded with sample data

    Scenario: Move Task to Another Parent
        Given I have a task "Move Root"
        And I have a task "Move Target"
        And I have a task "Move Child" as a child of "Move Root"

        When I move "Move Child" to "Move Target"
        And I expand "Move Target"
        Then I should see "Move Child" visible

    Scenario: Prevents Moving Task to Own Descendant (Cycle Detection)
        Given I have a task "Cycle Parent"
        And I have a task "Cycle Child" as a child of "Cycle Parent"
        And I have a task "Cycle Grandchild" as a child of "Cycle Child"

        When I open the move picker for "Cycle Parent"
        Then I should see "Cycle Child" disabled or hidden in move picker
        And I should see "Cycle Grandchild" disabled or hidden in move picker
