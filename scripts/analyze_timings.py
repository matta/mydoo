#!/usr/bin/env -S uv run
# /// script
# dependencies = ["python-dateutil", "tabulate"]
# ///
"""
Analyzes Playwright test logs for performance bottlenecks.
Parses logs containing "TEST STARTED/ENDED" and "Started/Finished step" markers
(produced by step-reporter.ts) to calculate durations, histograms, and identify slow steps.
"""

import re
import sys
from datetime import datetime
from dateutil import parser
from tabulate import tabulate
import statistics
from collections import defaultdict

LOG_FILE = "timings.log"

# Regex patterns
TEST_START_RE = re.compile(r"\[(.*?)\] --- TEST STARTED: (.*?) ---")
TEST_END_RE = re.compile(r"\[(.*?)\] --- TEST ENDED: (.*?) \((.*?)\) ---")
STEP_START_RE = re.compile(r"\[(.*?)\] Started step: (.*)")
STEP_END_RE = re.compile(r"\[(.*?)\] Finished step: (.*)")

def parse_log_file(filename):
    completed_tests = []
    all_steps = []

    current_test = None
    step_stack = [] # (name, start_time)

    with open(filename, 'r', encoding='utf-8', errors='ignore') as f:
        for line_num, line in enumerate(f):
            line = line.strip()

            # Test Start
            m_test_start = TEST_START_RE.search(line)
            if m_test_start:
                ts, name = m_test_start.groups()
                try:
                    dt = parser.isoparse(ts)
                    current_test = {
                        "name": name,
                        "start": dt,
                        "steps": [],
                        "line": line_num
                    }
                    step_stack = [] # Reset step stack on new test
                except Exception:
                    pass
                continue

            # Test End
            m_test_end = TEST_END_RE.search(line)
            if m_test_end:
                ts, name, status = m_test_end.groups()
                try:
                    dt = parser.isoparse(ts)
                    if current_test and current_test["name"] == name:
                        current_test["end"] = dt
                        current_test["duration"] = (dt - current_test["start"]).total_seconds()
                        current_test["status"] = status
                        completed_tests.append(current_test)
                        current_test = None
                except Exception:
                    pass
                continue

            # Step Start
            m_step_start = STEP_START_RE.search(line)
            if m_step_start:
                ts, name = m_step_start.groups()
                try:
                    dt = parser.isoparse(ts)
                    step_stack.append({"name": name, "start": dt})
                except Exception:
                    pass
                continue

            # Step End
            m_step_end = STEP_END_RE.search(line)
            if m_step_end:
                ts, name = m_step_end.groups()
                try:
                    dt = parser.isoparse(ts)

                    # Pop from stack matching name
                    match_idx = -1
                    for i in range(len(step_stack) - 1, -1, -1):
                        if step_stack[i]["name"] == name:
                            match_idx = i
                            break

                    if match_idx != -1:
                        step_data = step_stack.pop(match_idx)
                        duration = (dt - step_data["start"]).total_seconds()

                        full_step = {
                            "name": name,
                            "duration": duration,
                            "test": current_test["name"] if current_test else "UNKNOWN",
                            "start": step_data["start"]
                        }
                        all_steps.append(full_step)
                        if current_test:
                            current_test["steps"].append(full_step)
                except Exception:
                    pass
                continue

    return completed_tests, all_steps

def analyze_durations(items):
    if not items:
        return {}
    durations = [x["duration"] for x in items]
    return {
        "count": len(durations),
        "total": sum(durations),
        "min": min(durations),
        "max": max(durations),
        "avg": statistics.mean(durations),
        "median": statistics.median(durations),
        "p95": sorted(durations)[int(len(durations) * 0.95)] if len(durations) > 1 else durations[0]
    }

def print_histogram(data, title="Duration Distribution", bins=20):
    if not data:
        return

    min_val, max_val = min(data), max(data)
    if min_val == max_val:
        return

    bin_width = (max_val - min_val) / bins
    buckets = [0] * bins

    for x in data:
        idx = min(int((x - min_val) / bin_width), bins - 1)
        buckets[idx] += 1

    print(f"\n{title} ({len(data)} items):")
    max_count = max(buckets)
    scale = 40.0 / max_count if max_count > 0 else 1

    for i in range(bins):
        low = min_val + (i * bin_width)
        high = low + bin_width
        count = buckets[i]
        bar = "#" * int(count * scale)
        if count > 0:
            print(f"{low:6.2f}s - {high:6.2f}s | {count:3d} | {bar}")

def print_distribution(completed_tests, all_steps):
    print("\n" + "="*80)
    print(f"ANALYSIS REPORT: {len(completed_tests)} Tests, {len(all_steps)} Steps")
    print("="*80)

    # 1. Test Durations
    test_stats = analyze_durations(completed_tests)
    print(f"\nTEST SUMMARY ({test_stats.get('count',0)} tests):")
    print(f"  Total Time: {test_stats.get('total', 0):.2f}s")
    print(f"  Avg: {test_stats.get('avg', 0):.2f}s | Median: {test_stats.get('median', 0):.2f}s | P95: {test_stats.get('p95', 0):.2f}s | Max: {test_stats.get('max', 0):.2f}s")

    test_durations = [t["duration"] for t in completed_tests]
    print_histogram(test_durations, "Test Durations")

    # Top 15 Slowest Tests
    print("\nTOP 15 SLOWEST TESTS:")
    sorted_tests = sorted(completed_tests, key=lambda x: x["duration"], reverse=True)[:15]
    test_table = [[t["name"], f"{t['duration']:.2f}s", t["status"]] for t in sorted_tests]
    print(tabulate(test_table, headers=["Test Name", "Duration", "Status"]))

    # 2. Step Analysis
    # Group by name
    steps_by_name = defaultdict(list)
    for s in all_steps:
        steps_by_name[s["name"]].append(s["duration"])

    step_aggregates = []
    for name, durs in steps_by_name.items():
        step_aggregates.append({
            "name": name,
            "count": len(durs),
            "avg": statistics.mean(durs),
            "median": statistics.median(durs),
            "max": max(durs),
            "total": sum(durs)
        })

    # Top slow STEP TYPES (by average)
    print("\nSLOWEST STEP TYPES (by Average Duration, min 2 occurrences):")
    slow_step_types = sorted([s for s in step_aggregates if s["count"] > 1], key=lambda x: x["avg"], reverse=True)[:15]
    step_type_table = [[s["name"], s["count"], f"{s['avg']:.3f}s", f"{s['median']:.3f}s", f"{s['max']:.3f}s"] for s in slow_step_types]
    print(tabulate(step_type_table, headers=["Step Name", "Count", "Avg", "Median", "Max"]))

    # Top slow STEP TYPES (by Total Impact)
    print("\nHIGHEST IMPACT STEP TYPES (by Total Duration):")
    impact_step_types = sorted(step_aggregates, key=lambda x: x["total"], reverse=True)[:10]
    impact_table = [[s["name"], s["count"], f"{s['total']:.2f}s", f"{s['avg']:.3f}s"] for s in impact_step_types]
    print(tabulate(impact_table, headers=["Step Name", "Count", "Total Time", "Avg Time"]))

    # 3. Individual Slow Steps (Outliers)
    print("\nTOP 30 SLOWEST INDIVIDUAL STEP EXECUTIONS:")
    sorted_steps = sorted(all_steps, key=lambda x: x["duration"], reverse=True)[:30]
    slow_step_table = [[s["name"], s["test"], f"{s['duration']:.3f}s"] for s in sorted_steps]
    print(tabulate(slow_step_table, headers=["Step Name", "Test Context", "Duration"]))

if __name__ == "__main__":
    if len(sys.argv) > 1:
        LOG_FILE = sys.argv[1]

    print(f"Reading {LOG_FILE}...")
    try:
        completed_tests, all_steps = parse_log_file(LOG_FILE)
        if not completed_tests and not all_steps:
            print("No tests or steps found in log. Check formatting.")
        else:
            print_distribution(completed_tests, all_steps)
    except FileNotFoundError:
        print(f"File {LOG_FILE} not found.")
