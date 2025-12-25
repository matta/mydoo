#!/bin/sh

# Colors
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Checking filenames for kebab-case compliance..."

# Files to ignore (Grandfathered list)
# We use a grep pattern file for this to handle the list cleanly
cat <<EOF > .grandfathered_files
./docs/design/architecture_variances.md
./docs/plan/PHASE_3_CORE_INTERACTIONS.md
./docs/plan/PHASE_3.5_TASK_DEFAULTS.md
./docs/plan/PHASE_1_INFRA.md
./docs/plan/PHASE_2_DO_VIEW.md
./docs/plan/PHASE_5_PLAN_VIEW.md
./docs/plan/MASTER_PLAN.md
./docs/plan/PLAN_PHASE_5_STEP_6.md
./docs/plan/PHASE_4_TASK_DETAILS.md
./docs/plan/PLAN_PHASE_5_STEP_7.md
./TODO.md
./packages/tasklens/specs/compliance/schemas/test_case.d.ts
./packages/tasklens/specs/compliance/schemas/test_case.schema.json
./packages/tasklens/specs/compliance/fixtures/zero_feedback.yaml
./packages/tasklens/specs/compliance/fixtures/tree_order.yaml
./packages/tasklens/specs/compliance/fixtures/boost_lead_time.yaml
./packages/tasklens/specs/compliance/fixtures/visibility_place_filtering.yaml
./packages/tasklens/specs/compliance/fixtures/lead_time_edge_cases.yaml
./packages/tasklens/specs/compliance/fixtures/sequential_flow.yaml
./packages/tasklens/specs/compliance/fixtures/lead_time.yaml
./packages/tasklens/specs/compliance/fixtures/tree_order_id_conflict.yaml
./packages/tasklens/specs/compliance/fixtures/min_threshold.yaml
./packages/tasklens/specs/compliance/fixtures/complex_mutation.yaml
./packages/tasklens/specs/compliance/fixtures/boost_importance.yaml
./packages/tasklens/tests/unit/deleteTaskCascade.test.ts
./packages/tasklens/tests/unit/schemaValidation.test.ts
./packages/tasklens/tests/unit/createTaskDefaults.test.ts
./packages/tasklens/tests/unit/updateTask.test.ts
./packages/tasklens/src/README.md
./packages/tasklens/src/domain/pass5Leadtime.ts
./packages/tasklens/src/domain/pass2Schedule.ts
./packages/tasklens/src/domain/pass6Priority.ts
./packages/tasklens/src/domain/pass3Thermostat.ts
./packages/tasklens/src/domain/pass1Visibility.ts
./packages/tasklens/src/domain/pass4Weights.ts
./packages/tasklens/src/domain/pass7Container.ts
./GEMINI.md
./AGENTS.md
./ROLLING_CONTEXT.md
./apps/client/tests/e2e/priorityList.spec.ts
./apps/client/README.md
./apps/client/src/viewmodel/ui/useBreadcrumbs.test.tsx
./apps/client/src/viewmodel/ui/useNavigationState.tsx
./apps/client/src/viewmodel/ui/useBreadcrumbs.ts
./apps/client/src/viewmodel/ui/useNavigationState.test.tsx
./apps/client/src/viewmodel/useTodoList.ts
./apps/client/src/viewmodel/UseDocument.test.tsx
./apps/client/src/viewmodel/projections/usePriorityList.ts
./apps/client/src/viewmodel/projections/useTaskTree.test.tsx
./apps/client/src/viewmodel/projections/useTaskDetails.ts
./apps/client/src/viewmodel/projections/useTaskDetails.test.tsx
./apps/client/src/viewmodel/projections/UsePriorityList.test.tsx
./apps/client/src/viewmodel/projections/useTaskTree.ts
./apps/client/src/viewmodel/useTaskActions.ts
./apps/client/src/viewmodel/useBreadcrumbs.ts
./apps/client/src/viewmodel/useDocument.ts
./apps/client/src/viewmodel/containers/DoViewContainer.tsx
./apps/client/src/viewmodel/containers/TaskEditorContainer.tsx
./apps/client/src/viewmodel/intents/UseTaskIntents_Move.test.tsx
./apps/client/src/viewmodel/intents/UseTaskIntents.test.tsx
./apps/client/src/viewmodel/intents/useSystemIntents.ts
./apps/client/src/viewmodel/intents/UseSystemIntents.test.tsx
./apps/client/src/viewmodel/intents/useTaskIntents.ts
./apps/client/src/viewmodel/useTaskTree.ts
./apps/client/src/App.tsx
./apps/client/src/Main.tsx
./apps/client/src/components/TodoItem.tsx
./apps/client/src/components/shell/AppShellContainer.tsx
./apps/client/src/components/TodoApp.tsx
./apps/client/src/components/composites/PriorityTaskList.tsx
./apps/client/src/components/Breadcrumbs.tsx
./apps/client/src/components/modals/DeleteConfirmModal.tsx
./apps/client/src/components/modals/TaskEditorModal.test.tsx
./apps/client/src/components/modals/TaskEditorModal.tsx
./apps/client/src/components/modals/DeleteConfirmModal.test.tsx
./apps/client/src/components/primitives/QuickAddInput.test.tsx
./apps/client/src/components/primitives/TaskRow.tsx
./apps/client/src/components/primitives/TaskRow.test.tsx
./apps/client/src/components/primitives/QuickAddInput.tsx
./apps/client/src/components/primitives/TaskOutlineItem.css
./apps/client/src/components/primitives/TaskOutlineItem.tsx
./apps/client/src/components/primitives/TaskOutlineItem.test.tsx
./apps/client/src/components/views/plan/OutlineTree.tsx
./apps/client/src/components/views/plan/PlanViewContainer.tsx
./apps/client/src/components/views/plan/OutlineTree.test.tsx
./apps/client/src/components/views/plan/PlanViewContainer.test.tsx
./apps/client/src/components/InlineInput.tsx
./apps/client/src/components/TodoList.tsx
./apps/client/src/hooks/RepoProvider.tsx
./apps/client/src/lib/todoUtils.ts
./apps/client/src/dev/SeedData.tsx
EOF

# Find files that violate the convention
# Exclude: .git, node_modules, dist, .turbo, coverage, grandfathered files
# Exclude: Extensions handled by Biome (ts, tsx, js, jsx, cjs, mjs, json, jsonc)
VIOLATIONS=$(find . -type f \( \
    -name "*.md" -o \
    -name "*.sh" -o \
    -name "*.yaml" -o \
    -name "*.yml" -o \
    -name "*.toml" -o \
    -name "*.config.js" -o \
    -name "*.config.ts" \
  \) \
  -not -path "./.git/*" \
  -not -path "*/.git/*" \
  -not -path "./node_modules/*" \
  -not -path "*/node_modules/*" \
  -not -path "./dist/*" \
  -not -path "*/dist/*" \
  -not -path "./dev-dist/*" \
  -not -path "*/dev-dist/*" \
  -not -path "./build/*" \
  -not -path "*/build/*" \
  -not -path "./coverage/*" \
  -not -path "*/coverage/*" \
  -not -path "./.turbo/*" \
  -not -path "*/.turbo/*" \
  -not -path "./.husky/*" \
  -not -name ".grandfathered_files" \
  -not -name "check-filenames.sh" \
  -not -name "LICENSE" \
  -not -name "README.md" \
  -not -name "Dockerfile" \
  -not -name "Makefile" \
  | grep -E "/[^/]*[A-Z_][^/]*$" \
  | grep -vFf .grandfathered_files)

# Cleanup
rm .grandfathered_files

if [ -n "$VIOLATIONS" ]; then
  echo "${RED}❌ Error: The following files do not follow the kebab-case naming convention:${NC}"
  echo "$VIOLATIONS"
  echo "${RED}Please rename them to kebab-case (e.g., my-file.ts).${NC}"
  exit 1
else
  echo "✅ All filenames are compliant."
  exit 0
fi
