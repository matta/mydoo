# Algorithm Test Suite

This directory contains a language-agnostic test suite for the Tunnel
prioritization algorithm. It includes a JSON schema for defining test cases and
a set of YAML fixtures that cover various aspects of the algorithm.

## Purpose

The goal of this test suite is to ensure that any implementation of the Tunnel
algorithm behaves consistently and correctly according to the specification
defined in [ALGORITHM.md](./ALGORITHM.md). By running these tests, implementers
can verify conformance to the standard.

## Structure

The test suite is organized as follows:

- `specs/compliance/schemas/`: Contains the JSON schema for test cases.
- `specs/compliance/fixtures/`: Contains YAML files defining individual test scenarios.

## Schema

The test cases are defined using a JSON schema located at
`specs/compliance/schemas/test_case.schema.json`. This schema defines the structure of a
"Scenario", which consists of:

- `initial_state`: The state of the system before any steps are executed.
- `steps`: A sequence of operations (e.g., ticking time) and assertions
  (verifying task states).

Implemetations should parse these YAML files (validating against the schema) and
execute the steps sequentially.

## Fixtures

The following fixtures are provided to cover different features of the
algorithm:

| Fixture                           | Description                                                                |
| :-------------------------------- | :------------------------------------------------------------------------- |
| `balancing.yaml`                  | Tests the balancing logic for limiting active tasks.                       |
| `boost_importance.yaml`           | Verifies the effect of importance boosts on task prioritization.           |
| `boost_lead_time.yaml`            | Tests how lead time boosts affect task ordering.                           |
| `complex_mutation.yaml`           | Covers complex scenarios involving state mutations during execution.       |
| `decay.yaml`                      | Tests the decay of accumulated task value over time.                       |
| `lead_time.yaml`                  | Verifies standard lead time calculations.                                  |
| `lead_time_edge_cases.yaml`       | Tests edge cases for lead time, such as zero or infinite bounds.           |
| `min_threshold.yaml`              | Tests minimum value thresholds for task activation.                        |
| `sequential_flow.yaml`            | Verifies sequential task dependencies and blocking behavior.               |
| `sorting.yaml`                    | Tests the primary sorting logic for the queue.                             |
| `thermostat.yaml`                 | Tests feedback mechanisms that adjust system parameters.                   |
| `tree_order.yaml`                 | Verifies that tree structure order is respected when priorities are equal. |
| `tree_order_id_conflict.yaml`     | Detailed test for conflicts between ID sorting and tree order.             |
| `visibility_place_filtering.yaml` | Tests filtering of tasks based on visibility and place contexts.           |
| `weight.yaml`                     | Verifies the application of weights to task value.                         |
| `zero_feedback.yaml`              | Tests behavior when feedback loops result in zero adjustments.             |

## Usage

To use this suite for conformance testing:

1.  **Load Schema**: Load `specs/compliance/schemas/test_case.schema.json` to understand
    the data model.
2.  **Load Fixtures**: Parse each `.yaml` file in `specs/compliance/fixtures/`.
3.  **Execute**: For each scenario:
    - Initialize your algorithm implementation with the `initial_state`.
    - Iterate through `steps`.
    - For `tick` steps, advance the algorithm's clock.
    - For `verify` steps, assert that your implementation's internal state
      matches the expected values for the specified tasks.

## Test Runner Requirements

To ensure consistent execution of the test suite across different
implementations, test runners **MUST** adhere to the following invariants and
behaviors, which are enforced by the reference implementation's test harness.

### 1. Initialization Defaults

When parsing the `initial_state` and task definitions, the following default
values **MUST** be applied if specific fields are missing in the schema:

- **Task `lead_time`**: Defaults to `604800.0` seconds (1 week).
- **Task `importance`**: Defaults to `1.0`.
- **Task `is_sequential`**: Defaults to `False`.
- **Task `desired_credits`**: Defaults to `1.0`.
- **Task State `status`**: Defaults to `"Pending"`.
- **Task State `credits`**: Defaults to `0.0`.
- **Task State `credits_timestamp`**:
  - If `credits` are provided but `credits_timestamp` is missing, it **MUST**
    default to the `initial_state.current_time`.
  - If `credits` are not provided, it is irrelevant (implementation dependent,
    but conceptually `None` or `0`).
- **Global `timezone_offset`**: Defaults to `0` seconds (UTC) if not specified
  in `initial_state`.

### 2. Validation Rules

The test runner **MUST** enforce the following validation checks before or
during execution:

- **Timezone Awareness**: The `current_time` and all timestamps **MUST** be
  treated as timezone-aware. If an input timestamp is naive, it should be
  rejected or explicitly assumed to be UTC (the reference implementation rejects
  naive timestamps).
- **Referential Integrity**:
  - Any `place_id` referenced by a task (in `view_filter`) **MUST** exist in the
    `initial_state.places` map.
  - Any `view_filter` referenced in a generic `Step` **MUST** be either
    `"All Places"`, `"Anywhere"`, or a valid ID from `initial_state.places`.
  - The `initial_state.places` list **MUST NOT** contain a place with the ID
    `"Anywhere"`. This ID is reserved for the system-defined universal context.

### 3. Execution & Mutation

- **Time Advancement**: When a mutation specifies `advance_time_seconds`, the
  internal clock **MUST** be advanced by that amount _before_ processing any
  other updates in that mutation step.
- **Credit Updates**:
  - When `update_credits` is applied, the task's `credits_timestamp` **MUST** be
    updated to the _new_ `current_time` (after any time advancement).
- **Spec Mutations**:
  - If `due_date`, `importance`, or `desired_credits` are updated in a mutation,
    the existing values for other fields in the spec **MUST** be preserved
    (partial update).

### 4. Verification Standards

- **Floating Point Tolerance**: When verifying numeric fields (`score`,
  `effective_credits`, `normalized_importance`), comparisons **MUST** use a
  tolerance of **0.001** (e.g., `abs(actual - expected) < 0.001`). This accounts
  for minor differences in floating-point arithmetic across languages.
- **Boolean Exactness**: Boolean checks (`is_blocked`, `is_visible`, `is_ready`,
  `is_open`) **MUST** match exactly.

## Contributing

When adding new features to the algorithm, please add a corresponding test
fixture here to ensure it is covered by the standard test suite.
