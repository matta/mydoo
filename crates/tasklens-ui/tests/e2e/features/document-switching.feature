Feature: Document Switching

    Background:
        Given the user launches the app with a clean slate

    Scenario: User creates a new document
        Given the user is on a document
        When the user creates a new document
        Then the document ID changes
        And the new document is empty

    Scenario: User switches to an existing document by ID
        Given a document "A" with task "Task in A"
        And a document "B" with task "Task in B"
        When the user switches to document "A" by its ID
        Then the document ID should be the ID of "A"
        And the task "Task in A" should be visible
        When the user switches to document "B" by its ID
        Then the document ID should be the ID of "B"
        And the task "Task in B" should be visible
