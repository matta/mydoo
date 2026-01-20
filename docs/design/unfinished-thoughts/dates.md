There are actually three different dates involved in task management, but most
apps only give you fields for one or two.

- Constraint (Due Date): When must this happen? (e.g., "Tax day is April 15").
- Availability (Start/Defer Date): When can I start this? (e.g., "The form isn't
  released until Feb 1").
- Scheduling (Do Date): When will I do this? (e.g., "I plan to do my taxes on
  Sunday").

The design of date handling in todo apps is notoriously difficult because it
tries to collapse three distinct concepts into one or two database fields.

You are correct that the tension lies between **clarity** (modeling reality
accurately) and **fiddliness** (maintenance overhead).

Here is the breakdown of how **Things 3** handles this compared to your **Life
Balance** mental model, followed by a survey of how other influential apps
handle these distinct "time types." The design of date handling in todo apps is
notoriously difficult because it tries to collapse three distinct concepts into
one or two database fields.

You are correct that the tension lies between **clarity** (modeling reality
accurately) and **fiddliness** (maintenance overhead).

Here is the breakdown of how **Things 3** handles this compared to your **Life
Balance** mental model, followed by a survey of how other influential apps
handle these distinct "time types."

### 1. Confirming Behavior: Things 3

You are correct in your assessment of Things 3. It decouples the "Do Date" from
the "Due Date" entirely.

- **When (Start Date):** This dictates **visibility**. If you set a "When" date
  for next Friday, the task acts as if it does not exist (it is hidden in the
  "Upcoming" list) until next Friday morning, at which point it pops into your
  "Today" list.
- **Deadline (Due Date):** This dictates **urgency**. It places a flag on the
  item and a countdown (e.g., "3 days left").
- **The Interaction:**
  - **Scheduled + Deadline:** You can set a task to appear this Friday (When)
    but be due next Monday (Deadline). It will be hidden until Friday, then
    appear in Today with a "3 days left" badge.
  - **Scheduled Only:** You can set a task for Friday without a deadline. It
    appears Friday. If you don't do it Friday, it rolls over to Saturday's
    "Today" list automatically.
  - **Deadline Only:** You can set a deadline for next Monday _without_ a start
    date. The task sits in your "Anytime" list immediately (visible now), with a
    countdown badge.

**The Key Difference:** In _Life Balance_, the "Start Date" is calculated for
you (`Due Date - Lead Time`). In _Things 3_, you must manually calculate and
enter both dates if you want that "lead time" behavior.

### 2. The Core Design Problem

The reason you find this "fiddly" is that there are actually **three** different
dates involved in task management, but most apps only give you fields for one or
two.

1.  **Constraint (Due Date):** When _must_ this happen? (e.g., "Tax day is April
    15").
2.  **Availability (Start/Defer Date):** When _can_ I start this? (e.g., "The
    form isn't released until Feb 1").
3.  **Scheduling (Do Date):** When _will_ I do this? (e.g., "I plan to do my
    taxes on Sunday").

_Life Balance_ merged #1 and #3 using a "Lead Time" variable. Most modern apps
force you to manually manage #2 and #1, and use #2 as a proxy for #3.

### 3. Survey of Approaches in Other Apps

Here is how other influential apps handle this "Scheduled vs. Deadline"
conflict.

#### OmniFocus (The "Fiddly but Precise" Approach)

OmniFocus is the direct competitor to Things and the closest to the "Life
Balance" complexity, but with manual controls.

- **Fields:** It explicitly has distinct fields for **Defer Date** (Start) and
  **Due Date** (Deadline).
- **Behavior:** A task with a Defer Date is "Unavailable." It is grayed out and
  hidden from most views. On that date, it becomes "Available."
- **Pros:** Extremely precise. You can hide things until you can actually act on
  them.
- **Cons:** High maintenance. If you defer a task to Friday (planning to do it
  then) but get sick, the task becomes "Available" on Saturday and sits there.
  You have to manually re-defer it if you want to "schedule" it for a different
  day. It does not "roll over" as a scheduled intention; it simply changes state
  from "Hidden" to "Visible."

#### Todoist (The "Ambiguous Single Date" Approach)

Todoist is the most popular current app, and it historically avoided this
complexity by having only one date field.

- **Fields:** Traditionally just **Date**.
- **Behavior:** This field is ambiguous. If you set it to Friday, is it _due_
  Friday or are you _doing_ it Friday? The app treats it as a "Do Date"
  (scheduling). If you miss it, the text turns red (implying it was a deadline),
  but the app encourages you to "Reschedule" it to tomorrow.
- **Recent Change:** They recently added more distinction, but the core design
  relies on the user mentally deciding if a date is a hard deadline or a soft
  plan.
- **Cons:** This leads to "red date anxiety," where your list is full of overdue
  items that aren't actually deadlines, they were just plans you missed.

#### Sorted³ (The "Hyper-Scheduling" Approach)

This is a newer, influential design that takes "Scheduling" to the extreme.

- **Fields:** It assumes **everything** should be scheduled.
- **Behavior:** It has an "Auto-Schedule" feature. If you have a list of tasks
  with durations, it will fit them into the open slots in your calendar.
- **Pros:** Solves the "when will I do this" problem aggressively.
- **Cons:** Extremely fragile. One interrupted meeting breaks the whole day's
  schedule.

#### Amazing Marvin (The "Meta-Configurable" Approach)

Marvin is a power-user app that lets you toggle features on and off.

- **Behavior:** It has a specific "Do Date" vs. "Due Date" feature you can
  enable.
- **Strategy:** It explicitly allows you to plan "I will do this on Friday" (Do
  Date) separate from "This dies on Sunday" (Due Date).
- **Pros:** Matches the mental model perfectly.
- **Cons:** The UI complexity is very high.

### Summary of the Design Space

| App              | Start / Defer Date                 | Due Date                    | Interaction                                                                         |
| :--------------- | :--------------------------------- | :-------------------------- | :---------------------------------------------------------------------------------- |
| **Life Balance** | **Calculated** (Due - Lead Time)   | **Manual**                  | Best for low maintenance. "Lead Time" is a set-and-forget property.                 |
| **Things 3**     | **Manual** (Controls Visibility)   | **Manual** (Controls Badge) | Elegant but manual. You must set two dates to emulate "lead time."                  |
| **OmniFocus**    | **Manual** (Controls Availability) | **Manual**                  | Very strict. Deferred items don't "schedule" the day, they just become "available." |
| **Todoist**      | **Ambiguous** (Usually "Do Date")  | **Ambiguous**               | Fast, but prone to "fake overdue" items.                                            |

**The "Fiddly" Verdict:** The friction you feel in Things 3 (and OmniFocus) is
that they require you to be a manual scheduler. _Life Balance_ was unique
because it treated "Lead Time" as a static attribute of the task (e.g., "I
always need 3 days for this type of task"), whereas Things requires you to
decide that _every single time_ you schedule an instance of the task.

If you miss the "Lead Time" automation, you might find **Amazing Marvin**
interesting, as it is one of the few modern apps that explicitly models "Do
Dates" separate from "Due Dates" without hiding the task entirely. You are
correct in your assessment of Things 3. It decouples the "Do Date" from the "Due
Date" entirely.

- **When (Start Date):** This dictates **visibility**. If you set a "When" date
  for next Friday, the task acts as if it does not exist (it is hidden in the
  "Upcoming" list) until next Friday morning, at which point it pops into your
  "Today" list.
- **Deadline (Due Date):** This dictates **urgency**. It places a flag on the
  item and a countdown (e.g., "3 days left").
- **The Interaction:**
  - **Scheduled + Deadline:** You can set a task to appear this Friday (When)
    but be due next Monday (Deadline). It will be hidden until Friday, then
    appear in Today with a "3 days left" badge.
  - **Scheduled Only:** You can set
