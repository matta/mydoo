# Tunnel Algorithm Extension: Staleness Boost & Autofocus

**Status:** Draft, unimplemented

## 1. Problem Statement

The core Tunnel Algorithm prioritizes tasks based on "Life Balance" (Root Goals)
and "Relative Importance" (Tree Structure). While this ensures high-level
alignment with user goals, it suffers from two specific deficits in long-running
lists:

1.  **Sedimentation (Starvation):** Low-importance sub-tasks can sink to the
    bottom of the list and remain there indefinitely, effectively hiding from
    the user.
2.  **Lack of Circulation:** There is no mechanism to force a review of old
    tasks. The user is never prompted to decide whether to "Do, Delete, or
    Delay" these neglected items.

## 2. Goals & Constraints

The solution must achieve the following:

1.  **Autofocus Effect:** Neglected tasks must slowly rise in priority until
    they intrude into the active view, forcing a user decision.
2.  **Conservation of Energy:** The mechanism must **not** alter the `Credits`
    or `CreditsTimestamp` fields. The "Effort Pie Chart" must remain a strictly
    factual ledger of work performed (Conservation of Energy).
3.  **New Task Stability:** Newly created tasks must enter the list neutrally,
    without artificial boosting or suppression.
4.  **Safe Circulation:** The user must be able to "Snooze" a stale task
    (sending it back to the bottom) without corrupting the decay math used for
    the Life Balance calculation.
5.  **Snooze Safety:** The "Snooze" action must not affect the `Credits` or
    `CreditsTimestamp` fields.
6.  **Position-Agnostic Credits**: Tasks must be able to move within the outline
    without affecting the `Credits` or `CreditsTimestamp` fields, or
    dramatically altering the Life Balance calculation.

## 3. Data Model Extensions

To decouple "Circulation" (Workflow) from "Accountability" (Credit Decay), we
introduce a dedicated timestamp field.

### 3.1 New Field: `Task.LastReviewTimestamp`

**Type:** `Timestamp`

**Definition:** The last time the user explicitly interacted with this task in a
way that implies "awareness."

**Initialization:** Set to `CurrentTime` upon Task Creation.

**Update Triggers:**

1. **Completion:** When a task is marked Done.
2. **Snooze/Review:** When the user explicitly delays a stale task via the UI.
3. **Modification:** (Optional) When the user edits the task title or notes.

**Constraint:** Updating this field **MUST NOT** affect `Task.Credits` or
`Task.CreditsTimestamp`.

## 4. User Configuration Inputs

These parameters control the aggression of the Autofocus mechanism.

### 4.1 `StalenessSaturationDays` ($T_{saturation}$)

- **Definition:** The duration of neglect required for a task to reach its
  maximum priority boost.
- **Type:** `Duration` (Days).
- **Range:** `1.0` to `365.0`.
- **Default:** `30.0` Days.

### 4.2 `StalenessMaxBoost` ($K_{max}$)

- **Definition:** The maximum multiplier applied to a task's priority when it is
  fully saturated (stale).
- **Type:** `Float`.
- **Range:** `1.0` (Off) to `10.0` (Aggressive).
- **Default:** `5.0`.
  - _Rationale:_ A factor of 5.0 is typically required to allow a "Low
    Importance" task (e.g., Normalized Importance 0.2) to overtake a "Standard
    Importance" task (1.0).

## 5. Algorithm Logic Updates

We introduce a new term, `StalenessFactor`, into **Pass 6 (Final Priority)**.

### 5.1 Definitions

- **$T_{now}$**: The current system time.
- **$T_{last}$**: The value of `Task.LastReviewTimestamp`.
- **$\Delta t$ (Staleness Duration)**: The elapsed time since the task was last
  reviewed. $$\Delta t = \max(0, T_{now} - T_{last})$$

### 5.2 The Staleness Formula

We calculate a linear ramp that starts at 1.0 (neutral) and climbs to $K_{max}$
over the course of $T_{saturation}$, capping at $K_{max}$.

$$
\text{StalenessFactor} = 1.0 + \left( \min\left(1.0, \frac{\Delta t}{T_{saturation}}\right) \times (K_{max} - 1.0) \right)
$$

### 5.3 Integration into Pass 6

The Final Priority formula is updated to multiply the existing factors by the
new StalenessFactor.

$$
\text{Priority} = (\text{Visibility} ? 1.0 : 0.0) \times \text{NormalizedImportance} \times \text{Root.FeedbackFactor} \times \text{LeadTimeFactor} \times \mathbf{StalenessFactor}
$$

## 6. Workflow Interaction: The "Snooze"

To support the "Autofocus" workflow, the UI must provide a specific action for
handling stale tasks that have bubbled to the top but will not be done
immediately.

**Operation:** `SnoozeTask(TaskID)`

1.  **Update:** Set `Task(ID).LastReviewTimestamp = CurrentSystemTime`.
2.  **No-Op:** Do **NOT** modify `Credits`, `CreditsTimestamp`, or `Status`.
3.  **Result:**
    - $\Delta t$ becomes `0`.
    - `StalenessFactor` becomes `1.0`.
    - The task drops to its natural position in the list based solely on
      Importance and Life Balance.
    - The decay of the user's "Effort History" (Credits) continues
      uninterrupted.
