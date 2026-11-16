#!/usr/bin/env python3
"""Retain byte-identical primary/sidecar copies after an observed native Save."""
import argparse
import hashlib
import json
import os
from pathlib import Path

import grade_saved as G

ROOT = Path(__file__).resolve().parent


def write_new(path, raw):
    with path.open('xb') as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())
    path.chmod(0o444)
    G.require(path.read_bytes() == raw, 'raw snapshot copy differs')


def retain(run, case, label, save_record, capture_record, evidence_kind):
    models = G.parsed(G.read(ROOT / 'EXPECTATIONS.json'))
    model = models['states'][case][label]
    primary_relative = models['primary_paths'][case][label]
    primary = run / primary_relative
    for record in (save_record, capture_record):
        before = G.parsed(G.read(run / 'trace' / (record + '-before.json')))
        after = G.parsed(G.read(run / 'trace' / (record + '-after.json')))
        G.require(before['id'] == after['id'] == record and after['status'] == 'returned',
                  'source Save/capture did not return')
    raw = G.read(primary)
    digest = hashlib.sha256(raw).hexdigest()
    directory = run / 'snapshots'
    directory.mkdir(exist_ok=True)
    target = directory / (label + '.weftforge')
    write_new(target, raw)
    # Native construction is deliberately not parsed/graded during the input lease.
    G.require(hashlib.sha256(G.read(primary)).hexdigest() == digest, 'primary changed during copy')
    sidecars = []
    recovery_sha = None
    for suffix in ('.recovery', '.autosave', '.autosave.recovery'):
        source = Path(str(primary) + suffix)
        if not source.exists():
            continue
        sidecar_raw = G.read(source)
        name = label + suffix
        write_new(directory / name, sidecar_raw)
        sidecars.append({'suffix': suffix, 'snapshot': 'snapshots/' + name,
                         'bytes': len(sidecar_raw), 'sha256': G.sha(sidecar_raw)})
        if suffix == '.recovery':
            recovery_sha = G.sha(sidecar_raw)
    receipt = {'evidence_kind': evidence_kind, 'case': case, 'label': label,
               'revision': model['project']['revision'],
               'primary_relative_path': primary_relative,
               'primary_sha256': digest, 'snapshot_sha256': digest, 'bytes': len(raw),
               'native_save_record': save_record, 'clean_capture_record': capture_record,
               'previous_saved_label': models['previous_saved'][case][label],
               'recovery_sha256': recovery_sha, 'sidecars': sidecars,
               'construction_graded': False, 'raw_bytes_unedited': True}
    write_new(directory / (label + '.receipt.json'), (json.dumps(receipt, indent=2) + '\n').encode())
    return receipt


def selfcheck(output):
    directory = ROOT / 'SYNTHETIC-SNAPSHOT-001'
    directory.mkdir()
    (directory / 'work').mkdir()
    (directory / 'trace').mkdir()
    raw = b'Original synthetic bytes including whitespace:\n -0.0  0.470e0\n'
    (directory / 'work' / 'A.weftforge').write_bytes(raw)
    for identity in ('0001','0002'):
        (directory / 'trace' / (identity + '-before.json')).write_text(json.dumps({'id': identity}))
        (directory / 'trace' / (identity + '-after.json')).write_text(json.dumps({'id': identity, 'status': 'returned'}))
    first = retain(directory, 'A', 'A00-empty', '0001', '0002', 'synthetic_selfcheck')
    G.require((directory / 'snapshots/A00-empty.weftforge').read_bytes() == raw and
              not (directory / 'snapshots/A00-empty.weftforge').stat().st_mode & 0o222,
              'synthetic primary bytes/readonly check')
    replacement = b'Different synthetic current-primary bytes\n'
    (directory / 'work/A.weftforge').write_bytes(replacement)
    (directory / 'work/A.weftforge.recovery').write_bytes(raw)
    second = retain(directory, 'A', 'A01-square', '0001', '0002', 'synthetic_selfcheck')
    G.require((directory / 'snapshots/A01-square.recovery').read_bytes() == raw and
              second['previous_saved_label'] == 'A00-empty', 'synthetic exact old-primary recovery')
    try:
        retain(directory, 'A', 'A01-square', '0001', '0002', 'synthetic_selfcheck')
    except FileExistsError:
        pass
    else:
        raise AssertionError('duplicate snapshot overwrite was accepted')
    result = {'status': 'passed_synthetic_copy_only', 'native_executed': False,
              'raw_primary_and_recovery_copied_exactly': True, 'readonly_snapshots': True,
              'no_json_regeneration_or_live_construction_grading': True,
              'duplicate_snapshot_overwrite_refused': True,
              'source_snapshot_sha256': first['snapshot_sha256']}
    with output.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['copy', 'selfcheck'])
    parser.add_argument('--run', type=Path)
    parser.add_argument('--case', choices=list('ABCDE'))
    parser.add_argument('--label')
    parser.add_argument('--save-record')
    parser.add_argument('--capture-record')
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    if args.mode == 'selfcheck':
        G.require(args.report is not None and not args.report.exists(), 'fresh selfcheck report required')
        result = selfcheck(args.report)
    else:
        G.require(all((args.run,args.case,args.label,args.save_record,args.capture_record)), 'copy requires run/case/label/save/capture records')
        result = retain(args.run.resolve(), args.case, args.label, args.save_record,
                        args.capture_record, 'native_saved_bytes')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
