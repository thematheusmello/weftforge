#!/usr/bin/env python3
"""Copy opaque original primary/sidecar bytes; do not parse live construction."""
import argparse
import json
import os
import sys
from pathlib import Path

sys.dont_write_bytecode = True
import grade_saved as G


def write_new(path, raw):
    # Narrow byte-copy helper retained from the old immutable retain_saved.py.
    with path.open('xb') as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())
    path.chmod(0o444)
    G.require(path.read_bytes() == raw, 'raw snapshot copy differs')


def retain(run, case, label, save_record, capture_record, retain_record, evidence_kind='native_saved_bytes'):
    spec = G.CASES[case]
    index = spec['labels'].index(label)
    G.require(evidence_kind in ('native_saved_bytes', 'synthetic_selfcheck'), 'wrong evidence kind')
    G.require(int(save_record) < int(capture_record) < int(retain_record), 'unordered save/capture/copy')
    G.paired(run, save_record)
    G.require(G.paired(run, capture_record)[0]['method'] == 'getAXStateAndScreenshot', 'wrong clean capture')
    G.require(G.paired(run, retain_record, False)['method'] == 'fs.raw_snapshot' and
              not (run / 'trace' / (retain_record + '-after.json')).exists(), 'raw copy needs current FS operation')
    primary = run / spec['primary']
    raw = G.read(primary)
    directory = run / 'snapshots'
    directory.mkdir(exist_ok=True)
    write_new(directory / (label + '.weftforge'), raw)
    inventory = {}
    for suffix in G.SIDECARS:
        source = Path(str(primary) + suffix)
        G.require(not source.is_symlink(), 'native sidecar is a symlink')
        if source.exists():
            sidecar = G.read(source)
            write_new(directory / (label + suffix), sidecar)
            G.require(G.read(source) == sidecar, 'sidecar changed during raw copy')
            inventory[suffix] = {'present': True, 'bytes': len(sidecar), 'sha256': G.sha(sidecar)}
        else:
            inventory[suffix] = {'present': False, 'bytes': None, 'sha256': None}
    G.require(G.read(primary) == raw, 'primary changed during raw copy')
    G.require(all(Path(str(primary) + suffix).exists() == entry['present'] for suffix, entry in inventory.items()),
              'sidecar presence changed during raw copy')
    receipt = {'schema': 'weftforge.straight-cut.raw-copy.v1', 'evidence_kind': evidence_kind,
               'case': case, 'label': label, 'revision': spec['revisions'][index],
               'primary_relative_path': spec['primary'], 'primary_sha256': G.sha(raw),
               'snapshot_sha256': G.sha(raw), 'bytes': len(raw), 'native_save_record': save_record,
               'clean_capture_record': capture_record, 'raw_copy_record': retain_record,
               'previous_saved_label': spec['labels'][index - 1] if index else None,
               'sidecars': inventory, 'construction_graded': False, 'raw_bytes_unedited': True}
    write_new(directory / (label + '.receipt.json'), (json.dumps(receipt, indent=2) + '\n').encode())
    return receipt


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--case', choices=G.CASES, required=True)
    parser.add_argument('--label', required=True)
    parser.add_argument('--save-record', required=True)
    parser.add_argument('--capture-record', required=True)
    parser.add_argument('--retain-record', required=True)
    args = parser.parse_args()
    function = retain_export if args.case == 'HOLE-H2' and args.label in ('hole-r6.obj', 'hole-r6.svg', 'hole-r6.dxf') else retain
    print(json.dumps(function(args.run.resolve(), args.case, args.label, args.save_record,
                           args.capture_record, args.retain_record)))


def retain_export(run, case, label, action_record, capture_record, retain_record):
    G.require(int(action_record) < int(capture_record) < int(retain_record), 'unordered export action/capture/copy')
    G.paired(run, action_record)
    G.require(G.paired(run, capture_record)[0]['method'] == 'getAXStateAndScreenshot' and
              G.paired(run, retain_record, False)['method'] == 'fs.raw_snapshot' and
              not (run / 'trace' / (retain_record + '-after.json')).exists(), 'export raw-copy source differs')
    primary = run / 'exports' / label
    raw = G.read(primary)
    directory = run / 'exports-retained'; directory.mkdir(exist_ok=True)
    write_new(directory / label, raw)
    inventory = {}
    for suffix in G.SIDECARS:
        path = Path(str(primary) + suffix)
        G.require(not path.is_symlink(), 'export sidecar is a symlink')
        if path.exists():
            sidecar = G.read(path); write_new(directory / (label + suffix), sidecar)
            G.require(G.read(path) == sidecar, 'export sidecar changed while copying')
            inventory[suffix] = {'present': True, 'bytes': len(sidecar), 'sha256': G.sha(sidecar)}
        else:
            inventory[suffix] = {'present': False, 'bytes': None, 'sha256': None}
    G.require(G.read(primary) == raw, 'export changed while copying')
    G.require(all(Path(str(primary) + suffix).exists() == entry['present'] for suffix, entry in inventory.items()), 'export sidecar presence changed while copying')
    receipt = {'schema': 'weftforge.straight-hole.export-copy.v1', 'case': case, 'label': label, 'revision': 6,
               'sha256': G.sha(raw), 'bytes': len(raw), 'ui_action_record': action_record,
               'clean_capture_record': capture_record, 'raw_copy_record': retain_record, 'sidecars': inventory,
               'intentional_refusal': label.endswith('.dxf'), 'construction_graded': False}
    write_new(directory / (label + '.receipt.json'), (json.dumps(receipt, indent=2) + '\n').encode())
    return receipt


if __name__ == '__main__':
    main()
