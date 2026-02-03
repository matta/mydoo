# Tunnel Algorithm Specification: Credit Tracking and Task Prioritization

## 1. Scope and Purpose

This document specifies the core business logic for the Tunnel application's
dynamic prioritization engine. Its primary purpose is to define how the system
maintains equilibrium between a user's stated goals ("Desired Credits") and
their actual historical activity ("Credits").

The algorithm prevents burnout and ensures balanced progress by:

1.  **Tracking Effort**: Ensures users see progress and avoid burnout by
    balancing work across life areas.
2.  **Decaying History**: Prioritizes recent activity, keeping the system
    responsive to current needs.
3.  **Dynamic Scoring**: Surfaces neglected areas via feedback factors to
    maintain equilibrium.
4.  **Relative Importance**: Ensures fair prioritization within hierarchies.
5.  **Attribution on Deletion**: Preserves historical effort to maintain
    accuracy.

This specification serves as the authoritative source of truth for the
implementation of scoring, sorting, and visibility logic.

## 2. Overview

The Tunnel application dynamically prioritizes a flat list of actionable tasks
derived from a hierarchical project tree. The sorting logic compares "Desired
Credits" (user goals) against "Credits" (historical data) to identify
under-serviced areas. This document specifies the algorithms for credit
attribution, exponential decay, score calculation, and temporal constraints.

## 3. Data Model Definitions

The system relies on strict schema definitions to ensure data integrity across
synchronization boundaries and during algorithm execution.

### 3.1 Task Record

A `Task` represents a unit of work or a container of work.

#### 3.1.1 Stored Properties

| Field               | Type         | Definition                                                                                                  |
| :------------------ | :----------- | :---------------------------------------------------------------------------------------------------------- |
| `TaskID`            | `Integer`    | Unique Identifier (Primary Key).                                                                            |
| `Title`             | `String`     | Display name or title of the task. (Metadata)                                                               |
| `ParentID`          | `Integer?`   | **(Ref/Foreign Key)** Parent Task ID. Null indicates a Root Goal.                                           |
| `PlaceID`           | `Integer?`   | **(Ref/Foreign Key)** Link to the `Place` this task belongs to. Inherits from Parent if Null.               |
| `Status`            | `Enum`       | Current state ({ Pending, Done, Deleted }). Used to derive `IsPending`.                                     |
| `Importance`        | `Float`      | (0.0–1.0) Importance relative to parent. Set by user.                                                       |
| `CreditIncrement`   | `Float`      | Value added to history upon completion. Recommended range 0.0-2.0 (See Section 3.6). Must be >= 0.0.        |
| `Sequential`        | `Boolean`    | If true, blocks sibling tasks until this one is done.                                                       |
| `DesiredCredits`    | `Float`      | Target allocation value (only valid for Root Goals). Must be >= 0.0.                                        |
| `Schedule`          | `Enum`       | One of: `'Once'`, `'Routinely'`, `'DueDate'`, `'Calendar'`. Default `'Once'`.                               |
| `Due`               | `Timestamp?` | The effective due date (computed or explicit). Resolution fallback if `Once`.                               |
| `LeadTime`          | `Duration`   | (Default 8h / 28,800,000ms) Time before Due when task becomes Ready/Visible.                                |
| `Period`            | `Duration?`  | (Routinely only) Recurrence interval. Default 24h.                                                          |
| `LastDone`          | `Timestamp?` | (Routinely only) Timestamp of last completion.                                                              |
| `Credits`           | `Float`      | Current total of _decayed_ credit history for this task/subtree _as of `CreditsTimestamp`_. Must be >= 0.0. |
| `PriorityTimestamp` | `Timestamp`  | Timestamp used for stability in sorting or priority dampening.                                              |
| `CreditsTimestamp`  | `Timestamp`  | Timestamp of last credit modification ($t_0$ for decay calculations).                                       |

### 3.1.3 Scheduling Semantics

The system uses a flat schema where `Schedule` acts as the discriminator
controlling the behavior of the other temporal fields.

| Type          | Behavior                                                               | Key Fields Used              |
| :------------ | :--------------------------------------------------------------------- | :--------------------------- |
| **Once**      | Single occurrence. Inherits `Due` from parent if undefined (Fallback). | `Due` (optional), `LeadTime` |
| **Routinely** | Recycles in-place. `Due` = `LastDone` + `Period`.                      | `Period`, `LastDone`         |
| **DueDate**   | Explicit deadline. Does not repeat.                                    | `Due` (required), `LeadTime` |
| **Calendar**  | (Deferred) Linked to external appointment list.                        | N/A                          |

#### 3.1.2 Computed/Runtime Properties

**`Task.IsContainer`** (`Boolean`) _Source: Metadata_ : Structure Flag. True if
the task has children.

**`Task.IsPending`** (`Boolean`) _Source: State_ : Completion Status. True if
`Task.Status != Done`.

**`Task.IsReady`** (`Boolean`) _Source: Time_ : Start Constraint. True if
`CurrentTime >= Task.DueDate - (2 * Task.LeadTime)`.

**`Task.NormalizedImportance`** (`Float`) _Source: Tree_ : Normalized Tree
Weight. The fraction of the Root Task's total weight allocated to this task.
_Every Root Task starts with a `NormalizedImportance` of `1.0`. This value flows
down the tree, divided among siblings based on their `Importance`._ Formula:
`Task.Importance / Sum(Siblings.Importance) * Parent.NormalizedImportance`.

**`Task.EffectiveCredits`** (`Float`) _Source: Algorithm_ : Decayed History. The
current value of past credits after applying time decay. Formula:
`Task.Credits * (0.5 ^ (TimeDelta / HalfLife))` Where
`TimeDelta = CurrentTime - Task.CreditsTimestamp`.

**`Task.Visibility`** (`Boolean`) _Source: Context_ : View Filter. Final
visibility state combining Place (Open), Time (Ready), and Blocking. Formula:
`True` if (Open AND Task.IsReady AND FilterMatch), else `False`.

**`Task.Priority`** (`Float`) _Source: Algorithm_ : Final Sort Score. The
ultimate sorting value derived from all passes. Formula:
`(Task.Visibility ? 1.0 : 0.0) * Task.NormalizedImportance * FeedbackFactor * LeadTimeFactor`.

### 3.2 Place Record

A `Place` represents a Context (Location + Time) constraint.

#### 3.2.1 Data Structure (Variant)

`Place` is a **Variant Type** (or Sum Type), existing in one of two forms.
Implementations may model this as a single record with a discriminator flag
(e.g. `is_anywhere`) for simplicity.

1.  **Anywhere**: The universal, built-in context.
    - **Properties**: `ID` (Unique Identifier, Implementation Defined).
    - **Behavior**: Always Open, Included in all filters.

2.  **Specific Place**: A concrete user-defined context.
    - **Properties**:
      - `ID` (Unique Identifier).
      - `Hours` (Schedule/Union: Bitmask, "always_open", "always_closed", or
        Schedule Dict).
      - `IncludedPlaces` (List<ID>).

> **Implementation Note**: The ID for "Anywhere" is implementation-defined and
> NOT required to be 0, provided it is unique.

#### 3.2.2 Special Places: "Anywhere" (Variant Case 1)

"Anywhere" is a built-in Place constant.

> **Note**: The ID `"Anywhere"` is strictly **RESERVED**. User-defined places
> MUST NOT use this ID.

- **Definition**: A universal location that always exists.
- **Characteristics**:
  - `Hours`: Always Open (24/7).
  - **Universal Inclusion**: Tasks assigned to "Anywhere" appear in **ALL**
    specific Place filters (e.g. filtering by "Home" shows both "Home" tasks and
    "Anywhere" tasks).

#### 3.2.3 View Filters: "All Places"

"All Places" is a UI View Filter, not a stored Place.

- **Definition**: A mode instructing the system to ignore specific Place
  filtering.
- **Characteristics**:
  - Displays tasks from **every** place simultaneously.
  - **Respects Time**: Does **NOT** override `IsOpen` checks. Closed places
    remain hidden unless an explicit "Include Closed" override is active.

### 3.3 Relationships and Edge Cases

- **Task -> Place**: Many-to-One. A Task belongs to exactly one Place (inherits
  from parent if undefined).
- **Ambiguous Definitions**: Properties depending on Context (like `Priority`)
  are placed on the Task Record.
- **Computed Properties**: Runtime-only properties are explicitly marked.

### 3.4 State Definitions & Visibility Logic

To ensure the `Priority` formula is deterministic, we define four distinct
boolean input states. These combine to determine the final `Visibility` boolean.

| State         | Definition | Logic                                                                               |
| :------------ | :--------- | :---------------------------------------------------------------------------------- |
| **IsPending** | Completion | `Status != Done` (and not Deleted).                                                 |
| **IsReady**   | Temporal   | `CurrentTime >= (DueDate - 2 * LeadTime)`.                                          |
| **IsOpen**    | Contextual | `Place.Hours.Contains(CurrentTime)`.                                                |
| **IsBlocked** | Sequential | True if Parent is Sequential AND a preceding sibling (in Outline Order) is Pending. |

**Relationship to Visibility**: The `Visibility` computed property (Boolean) is
the logical conjunction of these states and the active View Filter.

- `Visibility = True` if (`IsPending` AND `IsReady` AND `IsOpen` AND
  `!IsBlocked` AND `FilterMatch`).
- `Visibility = False` otherwise.

> **Note**: This Boolean value acts as a gatekeeper in the `Priority` formula.

### 3.5 Task Store Functional Interface (Pseudocode)

The algorithm assumes a Store providing these primitive operations:

1.  **Read Operations**:
    - `get_task(id) -> Task`
    - `get_children(id) -> List<Task>` (Ordered by Priority). **MUST Return ALL
      children**, regardless of visibility/status, so the algorithm can filter
      them.
    - `get_ancestors(id) -> List<Task>` (For recursive property resolution)
    - `get_place(id) -> Place`

2.  **Write Operations**:
    - `create_task(parent_id, props) -> Task` (Inherits Credits/Place)
    - `update_task(id, diff) -> Task` (Triggers Score Recalc)
    - `complete_task(id)` (Triggers Attribution & Decay)

3.  **Calculation API**:
    - `recalculate_scores(view_filter)`: Runs the 7-Pass Algorithm.
    - `get_todo_list(context) -> List<Task>`: Returns filtered, sorted views.

### 3.6 Standard CreditIncrement Values

Implementations should allow users to select `CreditIncrement` values within the
range `[0.0, 1.0]`. The conceptual mapping distributes these values such that
"Average" occupies the central one-third of the spectrum.

| Effort Label      | Range Notation (Exact) | Decimal Approximation |
| :---------------- | :--------------------- | :-------------------- |
| **None**          | `{0.0}`                | `0.00`                |
| **Below Average** | `(0.0, 1/3)`           | `(0.00, 0.33)`        |
| **Average**       | `[1/3, 2/3]`           | `[0.33, 0.67]`        |
| **Above Average** | `(2/3, 1.0)`           | `(0.67, 1.00)`        |
| **Maximum**       | `{1.0}`                | `1.00`                |

### 4.1 Inherited Properties

Properties are initialized using one of three strategies:

| Property          | Strategy             | Details                                              |
| :---------------- | :------------------- | :--------------------------------------------------- |
| `Importance`      | **Fixed Default**    | Always `0.5`. Never inherited.                       |
| `PlaceID`         | **Copy-on-Create**   | Copy parent's value. Root default: `ANYWHERE`.       |
| `CreditIncrement` | **Copy-on-Create**   | Copy parent's value. Root default: `0.5`.            |
| `Due`             | **Runtime Fallback** | Only when `Schedule='Once'`. Resolves at query time. |

### 4.2 Logic

#### 4.2.1 Initialization (Creation Time)

**Goal**: Determine initial state for task properties.

When a new task $T$ is created within a parent $P$:

**Copy-on-Create Properties** (`PlaceID`, `CreditIncrement`):

1.  Read the parent's effective value (if parent undefined, walk up the
    hierarchy).
2.  Store that value directly on the new task $T$.
3.  **Effect**: The task owns its value. Future changes to ancestors do NOT
    propagate.

**Runtime Fallback Properties** (`Due` when `Schedule='Once'`):

1.  Store `undefined` on the new task $T$.
2.  At query time, the system walks up the tree to find an ancestor with a
    `Due`.
3.  **Effect**: Changes to ancestor schedules ARE reflected in descendants.

**Fixed Defaults** (`Importance`):

1.  Always initialize to `0.5`.
2.  **Effect**: Independent of hierarchy.

#### 4.2.2 Resolution (Runtime)

**Goal**: Determine the effective value for calculation.

When the algorithm requires the value of a property for Task $T$:

1.  **Check Local**: If $T$ has a defined value, use it.
2.  **Recursive Fallback**: If $T$'s value is `undefined`, check if the property
    participates in **Recursive Fallback** (e.g., `Due`, `LeadTime` for `Once`
    schedules).
    - **If Participating**: Check $T$'s Parent. Repeat this step up the
      hierarchy.
    - **If Not Participating** (e.g. `Importance`, `PlaceID`,
      `CreditIncrement`): Stop and immediately proceed to Step 3 (System
      Default).
3.  **Root Default**: If the root is reached and the value is still `undefined`,
    use the System Default.

| Property          | System Default            |
| :---------------- | :------------------------ |
| `PlaceID`         | `Anywhere`                |
| `CreditIncrement` | `0.5`                     |
| `Schedule`        | `'Once'`                  |
| `Due`             | `undefined` (No Due Date) |
| `LeadTime`        | `28800000` (8 Hours)      |
| `Importance`      | `1.0`                     |

## 5. Credit Data Management

### 5.1 Credit Attribution (Task Completion)

Completion of a task generates credit which is attributed directly to the
completed task. Ancestors reflect this historical effort through runtime
aggregation of `EffectiveCredits`.

**Trigger:** User marks Task C as complete.

**Procedure:**

1.  **Bring History to Present:** Before adding new credit, apply pending decay
    to the existing `Credits` of Task C. This ensures the current balance
    correctly reflects the passage of time (decay) up to the present moment
    before new credit is added.

    **Decay Formula:**
    `EffectiveCredits = Credits * (0.5 ^ ((CurrentTime - CreditsTimestamp) / HalfLife))`
    _(Default HalfLife = 7 Days)_

2.  **Accrue Credit:** Add the task's `CreditIncrement` to the decayed `Credits`
    of Task C. `C.Credits = C.Credits + C.CreditIncrement`

3.  **Checkpoint Time:** Update `CreditsTimestamp` to `CurrentSystemTime` for
    Task C.

4.  **Ancestor Aggregation (Runtime):** Ancestors do NOT store propagated
    credits. Instead, their `EffectiveCredits` property aggregates the
    `EffectiveCredits` of all descendants during the prioritization cycle.

5.  **Recurring Tasks:** Recurring tasks maintain a persistent `Credits`
    balance. Completing an instance adds to this running total; it does not
    reset `Credits` to zero.

## 6. Algorithm: Lifecycle

The algorithm operates in a discrete event loop triggered by user actions or
periodic refreshes.

### 6.1 The Update Cycle

1.  **Event**: User action (Complete Task) or Timer Tick.
2.  **Increment**: if Completion, add `CreditIncrement` to `Credits` and update
    `CreditsTimestamp`.
3.  **Decay**: Calculate `EffectiveCredits` based on `TimeDelta`.
4.  **Recalculate**: Execute the 7-Pass Scoring update.

## 7. Algorithm: Score Calculation (7-Pass)

The "Update" operation recalculates the `Priority` for all active tasks. It
ensures Deterministic results independent of execution order.

**Constraints:**

- **Maximum Hierarchy Depth**: To prevent stack overflows and performance
  degradation, the hierarchy is limited to a maximum depth of **20**. Attempts
  to process deeper trees will result in an error.

### Pass 1: Contextual Visibility

Filter tasks by Physical Context and Time.

- **Resolution**: `EffectivePlace = Task.PlaceID ?? Anywhere`. (See
  **[4. Property Inheritance Model](#4-property-inheritance-model)**).
- **Hours Check**: `IsOpen = EffectivePlace.Hours.Contains(CurrentTime)` (Note:
  "Anywhere" is always Open).
- **Place Match**:
  - If `Filter == "All Places"`: Match = True.
  - If `EffectivePlace == "Anywhere"`: Match = True (Universal).
  - Else:
    `Match = (EffectivePlace == Filter) OR (Filter.IncludedPlaces.Contains(EffectivePlace))`.
- **Result**: `Visibility = True` if (`IsOpen` AND `Match`), else `False`.

### Pass 2: Schedule Inheritance

Resolve actionable timeframe.

- **Inheritance**: If `Task.Schedule.DueDate` is `undefined`, resolve via
  Recursive Fallback. (See
  **[4. Property Inheritance Model](#4-property-inheritance-model)**).
- **Constraint**: Inheritance is a **fallback**. If the child has an explicit
  `DueDate`, it MUST NOT be overwritten by the parent.
- **Recurrence**: Helper calculation for `NextDue` dates.

### Pass 3: Deviation Feedback (The "Thermostat")

Calculate how far each Root Goal is from its target allocation. **Constants**:
`k=2.0` (Sensitivity), `epsilon=0.001` (Div/0 Protection).

1.  **Sum**:
    - `TotalDesired = Sum(Root.DesiredCredits)`
    - `TotalActual = Sum(Root.EffectiveCredits)`
2.  **Ratio**:
    - `TargetPercent = DesiredCredits / TotalDesired`
    - `ActualPercent = EffectiveCredits / TotalActual`
    - `DeviationRatio = TargetPercent / max(ActualPercent, epsilon)`
3.  **Factor**:
    - `FeedbackFactor = DeviationRatio ^ k`
    - **Note**: `DeviationRatio` is capped at 1000.0 to prevent integer
      overflows or effectively infinite priorities when `ActualPercent` is near
      zero. The logic ensures `ActualPercent` is never lower than `1 / 1000` for
      calculation purposes.

### Pass 4: Weight Normalization

Propagate importance down the tree.

- **Root**: `NormalizedImportance = 1.0` (Root Goals compete via Feedback, not
  Weight).
- **Child**:
  - `SumSiblings = Sum(Sibling.Importance)`
  - `NormalizedImportance = (Importance / SumSiblings) * Parent.NormalizedImportance`

### Pass 5: Lead Time Ramp

Calculate urgency based on deadlines.

- `TimeRemaining = DueDate - CurrentTime`
- **Ramp Function**:
  - If `TimeRemaining > 2 * LeadTime`: `LeadTimeFactor = 0.0`
  - Else:
    `LeadTimeFactor = Clamp(0, 1, (2 * LeadTime - TimeRemaining) / LeadTime)`

### Pass 6: Final Priority

Combine all factors into a sortable float.

```math
Priority = (Visibility ? 1.0 : 0.0) * NormalizedImportance * Root.FeedbackFactor * LeadTimeFactor
```

### Pass 7: Container Visibility

Recursively hide any Container Task that has at least one visible descendant
(child, grandchild, etc.).

- **Goal**: Ensure the Todo List remains uncluttered, showing only actionable
  leaves or empty containers.
- **Logic**:
  - For each Task T where `IsContainer = True`:
    - If `Any(Descendants(T).Visibility == True)`:
      - Set `T.Visibility = False`
      - Set `T.Priority = 0.0` (Effectively hidden)

**Noise Gate**: Tasks with a final `Priority <= 0.001` are effectively "noise"
and are excluded from the final sorted results (The Todo List), even if their
`Visibility` property is technically `True`. This filtering happens at the view
layer (sorting), preserving the calculated low priority for debugging purposes.

**Sorting Order**:

1.  **Primary Key**: `Priority` (Descending).
2.  **Secondary Key (Tie-Breaker)**: `DFS Pre-Order Index` (Outline Order).

> **Sequential Blocking Note**: If `Sequential=True`, the algorithm effectively
> treats non-first siblings as `Visibility=0` (hidden) during Pass 1 or Pass 6,
> filtering them out of the Final Priority list.

## 8. Verification (Scenarios & Tests)

This section defines the "Source of Truth" scenarios for validating the
algorithm (See SC-001..SC-004).

### 8.1 Scenario: The Thermostat (Dynamic Feedback)

**Goal**: Verify that neglected areas strictly rise in priority. **Setup**: Two
Roots: work (`Target=50%`) and life (`Target=50%`). `k=2.0`.

| Step             | State Changes                            | Deviation Calc                 | Feedback Factor                               | Result                   |
| :--------------- | :--------------------------------------- | :----------------------------- | :-------------------------------------------- | :----------------------- |
| **1. Balanced**  | `credits_work=100`<br>`credits_life=100` | `Total=200`<br>`Act%=0.5`      | `0.5 / 0.5 = 1.0`<br>`1.0^2 = 1.0`            | **Equal Score**          |
| **2. Imbalance** | `credits_work=150`<br>`credits_life=100` | `Total=250`<br>`LifeAct%=0.4`  | `LifeDev = 0.5/0.4 = 1.25`<br>`1.25^2 = 1.56` | **Life Boosted** (1.56x) |
| **3. Extreme**   | `credits_work=900`<br>`credits_life=100` | `Total=1000`<br>`LifeAct%=0.1` | `LifeDev = 0.5/0.1 = 5.0`<br>`5.0^2 = 25.0`   | **Emergency** (25x)      |

### 8.2 Scenario: Lead Time Ramp

**Goal**: Verify urgency curve. `LeadTime=7d`.

| Time Remaining | Formula Input     | LeadTimeFactor | Logic                   |
| :------------- | :---------------- | :------------- | :---------------------- |
| **20 Days**    | `20 > 14`         | **0.0**        | Too early (Hidden)      |
| **14 Days**    | `14 == 14`        | **0.0**        | Just appearing          |
| **10.5 Days**  | `(14 - 10.5) / 7` | **0.5**        | Halfway ramped          |
| **7 Days**     | `(14 - 7) / 7`    | **1.0**        | Due Date approach (Max) |
| **0 Days**     | `(14 - 0) / 7`    | **2.0 -> 1.0** | Overdue (Clamped Max)   |

### 8.3 Scenario: Decay Mechanics

**Goal**: Verify half-life reduction. `HalfLife=7d`, `Credits=100`.

| Time Delta   | Formula         | EffectiveCredits |
| :----------- | :-------------- | :--------------- |
| **0 Days**   | `100 * 0.5^0`   | **100.0**        |
| **7 Days**   | `100 * 0.5^1`   | **50.0**         |
| **14 Days**  | `100 * 0.5^2`   | **25.0**         |
| **3.5 Days** | `100 * 0.5^0.5` | **70.71**        |

### 8.4 Scenario: Weight Propagation

**Goal**: Verify parent weight influence on children.

| Parent NormalizedImportance | Child Importance | Num Siblings (Equal Imp) | Child NormalizedImportance |
| :-------------------------- | :--------------- | :----------------------- | :------------------------- |
| **1.0**                     | 1.0              | 1                        | `1.0 / 1.0 * 1.0 = 1.0`    |
| **0.5**                     | 1.0              | 1                        | `1.0 / 1.0 * 0.5 = 0.5`    |
| **1.0**                     | 0.2              | 5 (Sum=1.0)              | `0.2 / 1.0 * 1.0 = 0.2`    |

## Appendix A: Design Rationale

1.  **Feedback Ratio**: Calculated as `(Target% / Actual%)` rather than raw
    values to avoid scaling issues (Large projects would dominate small ones
    without % normalization).
2.  **Container Visibility**: We hide the Project when it has actionable steps
    to reduce clutter. However, if steps are hidden (sleeping), we show the
    Project to prevent "Black Holes".
3.  **Non-Recursive Places**: Defines `IncludedPlaces` as strictly one-level
    deep (A includes B). Complexity trade-off for performance.

## Appendix B: Success Criteria

- **SC-001**: "Panic" tasks (Due < Lead Time) consistently rank in top 5.
- **SC-002**: Neglected Life Areas (Ratio > 2.0) consistently surface at least 1
  task to top 10.
- **SC-003**: System calculates 10,000 node tree priority in < 200ms (Desktop).
