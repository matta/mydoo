# Initial Concept

Personal prioritization engine for managing life tasks, implementing a custom
algorithm and local-first synchronization.

# Product Guide: MyDoo

## Product Vision

MyDoo is a local-first, synchronization-agnostic task management system designed
to eliminate "list rot." By dynamically promoting tasks based on a "Life
Balance" algorithm (Target vs. Actual effort) and "Autofocus" principles, it
ensures that neglected tasks are surfaced and the user's focus is aligned with
their goals. The device is the ultimate source of truth, emphasizing privacy and
availability.

## Target Audience

- **Primary:** The author (Matt), for managing personal life and projects.
- **Secondary:** (Future) Individuals seeking a privacy-focused,
  algorithm-driven task manager.

## Core Value Proposition

- **Dynamic Prioritization:** Utilizes a custom Prioritization Algorithm to
  calculate task priority based on importance, effort history, and staleness (Implemented in Rust).
- **Local-First Architecture:** Built on Automerge and IndexedDB, ensuring full
  offline capability. Synchronization is currently handled via a central server
  using the `samod` sync protocol, though the design allows for
  potential future decentralized strategies.
- **Flexible Scheduling:** Offers a robust system for handling both routine
  tasks (floating intervals based on completion) and strict scheduled events
  (due dates), adapting to the user's natural workflow.
- **Context-Aware Filtering:** Provides "Place"-based filtering (e.g., Home,
  Work, Anywhere) to ensure that the prioritized "Do" list only shows tasks
  relevant to the user's current environment.

## Key Features

- **Views:**
  - **Do:** A flat, priority-sorted list of actionable tasks. Includes a context
    filter to switch between different "Places." No manual sorting; the
    algorithm decides.
  - **Plan:** An indented tree view for hierarchical project management
    (infinite nesting).
  - **Balance:** A high-level view comparing "Desired %" vs. "Actual %" effort
    for top-level goals (e.g., Health, Career).
- **Task Lifecycle:**
  - **Creation:** Smart inheritance of importance, effort, and "Place"
    properties.
  - **Completion:** "Done" tasks remain visible until "Refreshed"
    (Acknowledged).
- **Scheduling:** Polymorphic scheduling engine supporting:
  - **Once:** Single occurrence.
  - **Routinely:** Floating intervals (e.g., "Every 3 days after last done").
  - **By Due Date:** Hard deadlines.
  - **By Calendar:** (Future) Linked to external events.
