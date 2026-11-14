#!/usr/bin/env python3
"""Fixed bounded native document-I/O evidence; exact typed bits, no fitting."""
import copy
from decimal import Decimal as D
import json
from pathlib import Path
from ieee_document_oracle import at, put, load, sha, require, nearest_bits, exact_decimal, rounded_native, compare_native, reject

ROOT = Path(__file__).parent
INPUT_MANIFEST_SHA = '3a3e8b6bf7cd76282eb18afef508650831154fbb752b95395777fca1ecc55c5f'
EXPECTED_SHA = '2f471d3ef127f3057b1ed5d7085b5fc3b8e66e52a87506415aa29ee705c1903d'
ORACLE_SHA = '9e46dae6ababcb0066b01ec9864d5230c76b4daecd5a45a4378b952f5760a892'
SAVED = [('v1-original.weftforge', 'v1-saved-v2.weftforge'),
         ('v1-original.weftforge', 'v1-reopened-v2.weftforge'),
         ('v1-extreme-stiffness.weftforge', 'v1-extreme-saved-v2.weftforge'),
         *[('v2-ieee-boundaries.weftforge', name) for name in ('v2-saved.weftforge', 'v2-after-focus.weftforge',
           'after-invalid-number.weftforge', 'after-invalid-schema.weftforge', 'before-quit.weftforge')]]
RECOVERED = ['before-quit.weftforge.autosave', 'quit-recovery-capture.weftforge',
             'recovered-saved.weftforge', 'recovered-reopened.weftforge']


def identical(names):
    baseline = (ROOT / names[0]).read_bytes()
    for name in names[1:]:
        require((ROOT / name).read_bytes() == baseline, 'Native bytes differ: ' + name)
    return {'files': names, 'bytes_identical': True, 'shared_sha256': sha(ROOT / names[0])}


def main():
    require(sha(ROOT / 'input-manifest.json') == INPUT_MANIFEST_SHA, 'Frozen input manifest changed')
    require(sha(ROOT / 'expected-bits.json') == EXPECTED_SHA, 'Frozen expected-bit supplement changed')
    require(sha(ROOT / 'ieee_document_oracle.py') == ORACLE_SHA, 'Frozen exact oracle changed')
    manifest, expected = load(ROOT / 'input-manifest.json'), load(ROOT / 'expected-bits.json')
    for name, info in manifest['inputs'].items():
        require(sha(ROOT / name) == info['sha256'], 'Frozen input changed: ' + name)
    declaration, process, restart = [load(ROOT / name) for name in ('declaration.json', 'process.json', 'restart-process.json')]
    require(declaration['input_manifest_sha256'] == INPUT_MANIFEST_SHA, 'Native declaration input binding')
    require(declaration['source_commit'] == '1528e3662380a4a74c3b70d3ef9b11f2c607af5f', 'Candidate source record')
    require(declaration['executable_sha256'] == process['executable_sha256'] == restart['executable_sha256'] ==
            'a16acde0fb010a34b1452881ca9865ac3a3ef58908ca50e2719177fb9ab3b838', 'Candidate executable records')
    checks = []
    for source, output in SAVED:
        checked = compare_native(load(ROOT / source), load(ROOT / output))
        require(checked['float_fields_bit_exact'] == len(expected['cases'][source]['fields']), 'Expected field coverage')
        checks.append({'input': source, 'output': output, 'output_sha256': sha(ROOT / output), **checked})
    pre, outcome = load(ROOT / 'quit-precondition.json'), load(ROOT / 'quit-result.json')
    literal = manifest['criteria']['recovery']['focused_grain_literal']
    require(pre['focused_uncommitted_grain_literal'] == literal == '0.12345678901234568', 'Predeclared focused grain')
    recovery_source = load(ROOT / 'v2-ieee-boundaries.weftforge')
    recovery_source['project']['revision'] += 1
    next(p for p in recovery_source['project']['pieces'] if p['id'] == 20)['grain_angle'] = D(literal)
    recovery_checks = []
    for name in RECOVERED:
        checked = compare_native(recovery_source, load(ROOT / name))
        recovery_checks.append({'output': name, 'output_sha256': sha(ROOT / name), **checked})
    primary_sha = sha(ROOT / 'before-quit.weftforge')
    require(primary_sha == pre['primary_sha256'] == outcome['primary_sha256'] ==
            '5003de76ad856ebe78d213241416c2e99dcdf1723b5e9f01d396f7996ee29290', 'Focused quit overwrote primary')
    require(sha(ROOT / RECOVERED[0]) == outcome['autosave_sha256'], 'Recorded recovery bytes')
    groups = [identical(['v1-saved-v2.weftforge', 'v1-reopened-v2.weftforge']),
              identical([output for source, output in SAVED if source == 'v2-ieee-boundaries.weftforge']), identical(RECOVERED)]
    primary_doc, recovered_doc = load(ROOT / 'before-quit.weftforge'), load(ROOT / RECOVERED[0])
    e = copy.deepcopy(primary_doc); e['project']['revision'] += 1
    next(p for p in e['project']['pieces'] if p['id'] == 20)['grain_angle'] = D(literal)
    require(e == recovered_doc, 'Recovery has an unrelated serialized field delta')
    witnesses = {}
    for label, source_name, path, width in [
        ('one_ulp_point40', 'v1-original.weftforge', ('project', 'points', 22, 'position', 0), 64),
        ('one_ulp_stiffness', 'v1-extreme-stiffness.weftforge', ('project', 'materials', 0, 'warp_stiffness'), 64),
        ('wrong_f32_midpoint_color', 'v2-ieee-boundaries.weftforge', ('project', 'materials', 0, 'color', 0), 32)]:
        source = load(ROOT / source_name); changed = rounded_native(source)
        bits = nearest_bits(at(changed, path), width)
        put(changed, path, exact_decimal(0x3f000000 if width == 32 else bits + 1, width))
        witnesses[label] = reject(lambda: compare_native(source, changed))
    changed = copy.deepcopy(recovered_doc)
    next(p for p in changed['project']['pieces'] if p['id'] == 20)['grain_angle'] = exact_decimal(nearest_bits(D(literal), 64) + 1, 64)
    witnesses['one_ulp_recovered_grain'] = reject(lambda: compare_native(recovery_source, changed))
    witnesses['same_json_different_primary_bytes'] = reject(lambda: require((ROOT / 'before-quit.weftforge').read_bytes() ==
        (ROOT / 'before-quit.weftforge').read_bytes() + b' ', 'Primary bytes changed despite equivalent JSON'))
    reports = ['v1-saved-check.json', 'v1-reopened-check.json', 'v1-extreme-check.json', 'v2-saved-check.json',
               'v2-after-focus-check.json', 'after-invalid-number-check.json', 'after-invalid-schema-check.json',
               'before-quit-check.json', 'v2-quit-recovery-check.json']
    metadata = ['declaration.json', 'process.json', 'restart-process.json', 'quit-precondition.json', 'quit-result.json']
    names = list(manifest['inputs']) + ['input-manifest.json', 'expected-bits.json', 'ieee_document_oracle.py',
            *[output for _, output in SAVED], *RECOVERED, *reports, *metadata]
    result = {'scenario': declaration['id'], 'kind': 'implementation_only', 'status': 'passed_executed_native_io_subset',
              'full_contract_verified': False, 'evaluator_sha256': sha(Path(__file__)), 'oracle_sha256': ORACLE_SHA,
              'input_manifest_sha256': INPUT_MANIFEST_SHA, 'expected_bits_sha256': EXPECTED_SHA,
              'candidate_source_commit': declaration['source_commit'], 'candidate_executable_sha256': declaration['executable_sha256'],
              'all_five_input_files_unchanged': True, 'saved_outputs': checks, 'byte_identity_groups': groups,
              'modern_focused_quit_recovery': {'checks': recovery_checks, 'primary_unchanged': True,
                   'only_changes': ['revision11->12', 'piece20.grain_angle .25->predeclared literal'],
                   'grain_expected_binary64': f'0x{nearest_bits(D(literal), 64):016x}'},
              'negative_witnesses': witnesses,
              'completed_subset': ['v1 load/save/reopen with explicit migration and188 exact typed float fields',
                   'v1 extreme-stiffness load/save', 'modern load/save, untouched material focus/blur, invalid number/schema state retention',
                   'modern focused-grain CmdQ recovery, explicit Recover/save and native reopen/save'],
              'not_executed': ['v1 invalid-load branch', 'v1 focused-field branch', 'v1 focused-quit/recovery branch'],
              'artifacts': {name: {'sha256': sha(ROOT / name), 'bytes': (ROOT / name).stat().st_size} for name in names},
              'limits': ['Exact typed IEEE bits are compared; raw decimal token spelling need not remain identical after typed rounding/serialization.',
                   'UI gestures, errors, focused drafts, process identity and explicit recovery interaction are parent-observed evidence.',
                   'Parser cannot reconstruct values already saved after rounding; prior failed-candidate evidence remains historical.',
                   'Executed subset only; no full workflow, reference parity, fitting, simulation or performance claim.']}
    output = ROOT / 'independent-document-io-check.json'
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'full_contract_verified': False, 'report': output.name,
                      'report_sha256': sha(output), 'evaluator_sha256': result['evaluator_sha256'], 'negative_witnesses_rejected': len(witnesses)}))


if __name__ == '__main__':
    main()
