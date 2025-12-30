Feature: Document Switching

    Scenario: User creates a new document
        Given the user is on a document
        When the user creates a new document
        Then the document ID changes
        And the new document is empty
