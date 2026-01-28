import type {
  Reporter,
  TestCase,
  TestResult,
  TestStep,
} from "@playwright/test/reporter";

/**
 * StepReporter logs test progress including step start/end times.
 * These logs can be analyzed using `scripts/analyze_timings.py` to identify
 * slow tests and individual step bottlenecks.
 */
class StepReporter implements Reporter {
  onTestBegin(test: TestCase, _result: TestResult) {
    console.log(
      `\n[${new Date().toISOString()}] --- TEST STARTED: ${test.title} ---`,
    );
  }

  onTestEnd(test: TestCase, result: TestResult) {
    console.log(
      `[${new Date().toISOString()}] --- TEST ENDED: ${test.title} (${result.status.toUpperCase()}) ---\n`,
    );
  }

  onStepBegin(_test: TestCase, _result: TestResult, step: TestStep) {
    // Filter for only "test.step" to avoid internal Playwright steps (like 'Before Hooks')
    // Remove the if-condition to see absolutely everything (fixture setup, etc.)
    // biome-ignore lint/correctness/noConstantCondition: debugging
    if (step.category === "test.step" || true) {
      console.log(`[${new Date().toISOString()}] Started step: ${step.title}`);
    }
  }

  onStepEnd(_test: TestCase, _result: TestResult, step: TestStep) {
    // biome-ignore lint/correctness/noConstantCondition: debugging
    if (step.category === "test.step" || true) {
      console.log(`[${new Date().toISOString()}] Finished step: ${step.title}`);
    }
  }

  printsToStdio() {
    return true;
  }
}

export default StepReporter;
