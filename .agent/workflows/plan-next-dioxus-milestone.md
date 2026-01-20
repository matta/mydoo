---
description: Plan the next rust_migration milestone
---

You are an expert Rust, Dioxus, Typescript, and React engineer planning the next stage of a migration of this repository from React to Dioxus. Read rust_migration.md and identify the next milestone that has not yet been completed. Consider section of the doc to be your implementation plan. Do not use an ephemeral implementation plan markdown doc. Instead, flesh out the milestone in rust_migration.md itself. Plan in sufficient detail that a lesser agent will be able to complete the tasks without doing research. Refresh your context with @AGENTS.md and any other relevant context from the docs directory. Of particular note: the prd.md doc.

STRICT REQUIREMENT: the milestone must be focused and minimal, targeting one achievable goal that is, ideally, indivisible into smaller work items.

COMPLETION GATE:
 - the plan is expressed as an outline
 - the plan uses markdown checkboxes `[ ]` for ALL actionable steps.
 - ❌ NEGATIVE CONSTRAINT: Do NOT use standard bullet points (`-`) for implementation details.
 - ✅ CORRECT FORMAT: `- [ ] Create file foo.rs`
 - ❌ WRONG FORMAT:   `- Create file foo.rs`
 - DO NOT IMPLEMENT THE PLAN (Do not write code files, only update the plan doc)