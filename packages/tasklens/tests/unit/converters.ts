import type {
  Step as FeatureStep,
  TunnelAlgorithmFeatureSchema,
} from '../../src/generated/feature';
import type {
  Step as LegacyStep,
  TunnelAlgorithmTestCaseSchema,
} from '../../src/generated/test-case';

type Variables = Record<string, string | number | boolean | null | undefined>;

/**
 * Recursively interpolates template strings like `${varName}` with variable values.
 */
export function interpolate<T>(template: T, variables: Variables): T {
  if (typeof template === 'string') {
    return template.replace(/\$\{([^}]+)\}/g, (_, key: string) => {
      const val = variables[key];
      if (val === undefined) return `\${${key}}`;
      return String(val);
    }) as T;
  }
  if (Array.isArray(template)) {
    return template.map(item => interpolate(item, variables)) as T;
  }
  if (template !== null && typeof template === 'object') {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(template)) {
      result[key] = interpolate(value, variables);
    }
    return result as T;
  }
  return template;
}

/**
 * Recursively casts string representations to their proper types.
 * Handles 'true'/'false' → boolean, 'null' → undefined, numeric strings → number.
 */
export function castTypes<T>(obj: T): T {
  if (typeof obj === 'string') {
    if (obj === 'true') return true as T;
    if (obj === 'false') return false as T;
    // Return undefined instead of null per project conventions
    if (obj === 'null') return undefined as T;
    if (/^-?\d+(\.\d+)?$/.test(obj)) return Number(obj) as T;
    return obj as T;
  }
  if (Array.isArray(obj)) {
    return obj.map(castTypes) as T;
  }
  if (obj !== null && typeof obj === 'object') {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = castTypes(value);
    }
    return result as T;
  }
  return obj;
}

function cleanUndefined<T>(obj: T): T {
  if (Array.isArray(obj)) {
    return obj.map(cleanUndefined) as T;
  }
  if (obj !== null && typeof obj === 'object') {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      if (value !== undefined) {
        result[key] = cleanUndefined(value);
      }
    }
    return result as T;
  }
  return obj;
}

/**
 * Converts a legacy test case format to the new Feature schema format.
 *
 * Note: Uses type assertion at the return level because the legacy and feature
 * schemas have structurally compatible but not identical types due to
 * exactOptionalPropertyTypes strictness in tsconfig.
 */
export function convertLegacyToFeature(
  legacy: TunnelAlgorithmTestCaseSchema,
): TunnelAlgorithmFeatureSchema {
  // Build as plain object, then assert to target type
  const feature = {
    feature: legacy.name,
    description: legacy.description,
    background: legacy.initial_state,
    scenarios: [
      {
        name: 'Legacy Scenario',
        steps: legacy.steps.map(step => ({
          view_filter: step.view_filter,
          when: step.mutation,
          description: step.name,
          // biome-ignore lint/suspicious/noThenProperty: Schema defined property
          then: {
            expected_order: step.expected_order,
            expected_props: step.expected_props,
          },
        })),
      },
    ],
  };
  return cleanUndefined(feature) as TunnelAlgorithmFeatureSchema;
}

interface LegacyStepWithMutation extends LegacyStep {
  mutation?: LegacyStep['mutation'] & {
    task_updates?: Record<string, unknown>[];
  };
}

/**
 * Converts a Feature schema format to legacy test case format(s).
 * Each scenario with examples expands into multiple test cases.
 */
export function convertFeatureToLegacy(
  feature: TunnelAlgorithmFeatureSchema,
): TunnelAlgorithmTestCaseSchema[] {
  const testCases: TunnelAlgorithmTestCaseSchema[] = [];

  for (const scenario of feature.scenarios) {
    const examples = scenario.examples || [{}];

    for (const [exampleIndex, example] of examples.entries()) {
      // 1. Interpolate & Cast
      const hydratedBackground = castTypes(
        interpolate(feature.background || {}, example),
      );
      const hydratedScenarioSteps = castTypes(
        interpolate(scenario.steps, example),
      ) as FeatureStep[];

      // 2. Map Steps
      const legacySteps: LegacyStepWithMutation[] = hydratedScenarioSteps.map(
        (step, idx) => {
          const mutation: Record<string, unknown> = {...(step.when || {})};

          if (step.given?.tasks) {
            const existingUpdates = (mutation.task_updates as unknown[]) || [];
            mutation.task_updates = [...existingUpdates, ...step.given.tasks];
          }

          return {
            name: step.description || `Step ${idx + 1}`,
            view_filter: step.view_filter,
            mutation:
              Object.keys(mutation).length > 0
                ? (mutation as LegacyStep['mutation'])
                : undefined,
            expected_order: step.then?.expected_order as string[] | undefined,
            expected_props: step.then?.expected_props,
          } as LegacyStepWithMutation;
        },
      );

      let testName = `${feature.feature}: ${scenario.name}`;
      if (scenario.name === 'Legacy Scenario') {
        testName = feature.feature;
      } else if (
        examples.length > 1 ||
        (scenario.examples && scenario.examples.length > 0)
      ) {
        // Append example index only if it was an explicitly parameterized scenario
        // For round-trip of single legacy, scenarios won't have examples
        testName = `${feature.feature}: ${scenario.name} (Ex ${exampleIndex + 1})`;
      }

      testCases.push(
        cleanUndefined({
          name: testName,
          description: feature.description || '',
          initial_state:
            hydratedBackground as TunnelAlgorithmTestCaseSchema['initial_state'],
          steps: legacySteps as LegacyStep[],
        }),
      );
    }
  }

  return testCases;
}
