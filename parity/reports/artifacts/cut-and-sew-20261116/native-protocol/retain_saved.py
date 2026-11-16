#!/usr/bin/env python3
"""Opaque retention only; reuse the frozen Cut byte-copy helper with new cases."""
import argparse
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
import grade_saved as G

FROZEN_PATH = G.REPO / '.local/straight-cut-native-plan-v2-20261116/retain_saved.py'
assert hashlib.sha256(FROZEN_PATH.read_bytes()).hexdigest() == '5fd28138d6ce42991ad3576a5ecfec2a4f8cbe70a0b98be32ebb37f809a12888'
SPEC = importlib.util.spec_from_file_location('frozen_opaque_retainer', FROZEN_PATH)
FROZEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FROZEN)
FROZEN.G = G
retain = FROZEN.retain


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--case', choices=G.CASES, required=True)
    parser.add_argument('--label', required=True)
    parser.add_argument('--save-record', required=True)
    parser.add_argument('--capture-record', required=True)
    parser.add_argument('--retain-record', required=True)
    args = parser.parse_args()
    G.require(args.run.is_absolute() and args.run == args.run.resolve(), 'absolute canonical owned run required')
    print(json.dumps(retain(args.run, args.case, args.label, args.save_record, args.capture_record, args.retain_record)))


if __name__ == '__main__':
    main()
