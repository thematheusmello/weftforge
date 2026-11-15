"""Invoke unchanged numerical analysis through the exact current-metadata adapter."""
import argparse
from pathlib import Path
import time

import consumer16 as c
import resources as r
import signatures

ROOT = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--deadline", type=float, required=True)
    args = parser.parse_args()
    r.deadline_check(args.deadline)
    c.REPO = Path(c.read(ROOT / "READONLY-BINDING.json")["repo"])
    c.OUTPUT_DIRECTORIES = r.OUTPUT_DIRECTORIES
    c.check_metadata = signatures.check_current_metadata
    result = c.analyze_saved("base", ROOT / "runs/base", deadline=args.deadline)
    result.update(source_commit=c.SOURCE, experiment="Harbor-A16x32-001",
                  consumer_source="consumer16.py; only settings()/target_step() changed",
                  metadata_adapter="signatures.check_current_metadata; full exact signature",
                  analysis_completed_monotonic_seconds=time.monotonic())
    r.deadline_check(args.deadline)
    c.write_report(ROOT / "runs/base/analysis16.json", result)
    print({key: result.get(key) for key in ("status", "strict_delivery_passed", "failed_frames_by_screen", "errors")})
    if result["status"] != "analyzed_complete":
        return 1
    return 0 if result["strict_delivery_passed"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
