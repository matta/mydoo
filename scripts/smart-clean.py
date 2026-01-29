#!/usr/bin/env -S uv run
# /// script
# dependencies = []
# ///

import subprocess
import sys
import re
import os
import shutil

# --- Configuration ---

HIGH_CONFIDENCE_PATTERNS = [
    re.compile(r"node_modules"),
    re.compile(r"dist/"),
    re.compile(r"\.turbo/"),
    re.compile(r"coverage/"),
    re.compile(r"playwright-report/"),
    re.compile(r"test-results/"),
    re.compile(r"\.tsbuildinfo"),
    re.compile(r"\.husky/_"),
    re.compile(r"src/generated"),
]

GIT_CLEAN_ARGS = [
    "clean",
    "-fdX",
    "-n",
    "-e", "!.proto",
    "-e", "!.env",
    "-e", "!*.local",
    "-e", "!.vscode",
    "-e", "!.gemini",
    "-e", "!.specify",
]

def ask(query):
    try:
        response = input(f"{query} [y/N] ").strip().lower()
        return response == "y" or response == "yes"
    except EOFError:
        return False

def main():
    print("Analyzing repository for ignored files...")
    
    result = subprocess.run(
        ["git"] + GIT_CLEAN_ARGS, 
        capture_output=True, 
        text=True, 
        check=False
    )
    
    if result.returncode != 0:
        print("Failed to run git clean:", result.stderr, file=sys.stderr)
        sys.exit(1)
        
    raw_output = result.stdout.strip()
    
    if not raw_output:
        print("Repository is already clean.")
        return

    paths = []
    for line in raw_output.split("\n"):
        cleaned = line.replace("Would remove ", "").strip()
        if cleaned:
            paths.append(cleaned)
            
    high_confidence = []
    other = []
    
    for p in paths:
        if any(pat.search(p) for pat in HIGH_CONFIDENCE_PATTERNS):
            high_confidence.append(p)
        else:
            other.append(p)
            
    # Process High Confidence
    if high_confidence:
        print(f"\nFound {len(high_confidence)} 'High Confidence' items (node_modules, dist, logs, etc.).")
        examples = ", ".join(high_confidence[:3])
        if len(high_confidence) > 3:
            examples += "..."
        print(f"Examples: {examples}")
        
        if ask("Delete all high confidence items?"):
            print("Deleting...")
            for p in high_confidence:
                try:
                    if os.path.isdir(p):
                        shutil.rmtree(p)
                    else:
                        os.remove(p)
                except Exception as e:
                    print(f"Failed to delete {p}: {e}", file=sys.stderr)
            print("✅ High confidence items deleted.")
        else:
            print("Skipped.")
    else:
        print("\nNo high confidence items found.")
        
    # Process Others
    if other:
        print(f"\nFound {len(other)} 'Other' items.")
        
        limit = 20
        if len(other) < limit:
            print("Items:")
            for p in other:
                print(f" - {p}")
        else:
            print("First 10 items:")
            for p in other[:10]:
                print(f" - {p}")
            print(f"... and {len(other) - 10} more.")
            
        if ask("Delete these items?"):
            print("Deleting...")
            for p in other:
                try:
                    if os.path.isdir(p):
                        shutil.rmtree(p)
                    else:
                        os.remove(p)
                except Exception as e:
                    print(f"Failed to delete {p}: {e}", file=sys.stderr)
            print("✅ Other items deleted.")
        else:
            print("Skipped.")
    else:
        print("\nNo other ignored items found.")
        
    print("\nClean check complete.")

if __name__ == "__main__":
    main()
