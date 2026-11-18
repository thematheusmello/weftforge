"""Prospective root-only source installer. Does not invoke bind_source or Cargo."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys

OWN = Path(__file__).resolve().parent
ROOT = OWN.parent.parent
INT = ROOT / '.local/notch-bend-integration-20261117'
PREP = ROOT / '.local/capsule-supporting-plane-fullcase-prep-20261118'
HREVIEW = ROOT / '.local/capsule-supporting-plane-fullcase-review-20261118'
DEST = INT / '.local/harbor-capsule-supporting-plane-preparation-20261118'
FCASE = INT / '.local/capsule-supporting-plane-integrated-source-20261118'
GCASE = INT / '.local/capsule-constructor-clean-verify-20261118'
FCLOSURE = INT / '.local/capsule-supporting-plane-integrated-source-20261118-closure/CLOSURE.json'
GCLOSURE = INT / '.local/capsule-constructor-clean-verify-closure-20261118/ROOT-CLOSURE.json'
PRODUCTION = ROOT / '.local/capsule-supporting-plane-source-review-20261118/REVIEW-002.md'
F = 'c766f3fa48c03ac698da8b22cc3bf927ee71f195'
G = 'e30a71a5ffbaef9de00e1d4b10a6fa51a28ff621'
PREP_SHA = '44699899258f958c892d2a470ade283d49986119b71082e398742c92c47ef0e3'
REVIEW_SHA = '5417e9160c6bd9f75baab57ba303026331107844ac5a9af1d25a4062cc6e3f44'
FLAGS = ['root_read_production_diff_validation_callers_final_tests_physics_and_independent_review',
         'root_has_reviewed_v6_empty_notch_extension_and_original_metadata_fields',
         'root_read_actual_current_required_check_receipts',
         'root_reviewed_initial_recovery_policy_counter_and_remaining_nonlinear_limits']


def require(ok, message):
    if not ok: raise ValueError(message)


def raw(path):
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), 'Missing/pending/nonregular input: ' + str(path))
    return path.read_bytes()


def sha(path): return hashlib.sha256(raw(path)).hexdigest()
def read(path): return json.loads(raw(path))
def encoded(value): return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n').encode()
def same(a, b): return encoded(a) == encoded(b)
def rel(path): return str(Path(path).relative_to(INT))
def descriptor(path): return {'path': rel(path), 'sha256': sha(path)}


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=INT, timeout=10).decode().strip()


def descriptor_matches(path, expected):
    return sha(path) == expected['sha256'] and path.stat().st_size == expected['bytes'] and stat.S_IMODE(path.stat().st_mode) == expected['mode']


def chain(case, phase, commit):
    paths = {'plan': case / (phase + '.json'), 'result': case / phase / 'RESULT.json',
             'terminal': case / phase / 'TERMINAL.json', 'external_terminal': case / 'EXTERNAL-TERMINAL.json',
             'post_terminal_clock': case / 'POST-TERMINAL-CLOCK.json'}
    value = {name: read(path) for name, path in paths.items()}
    plan, result, terminal, external, post = [value[k] for k in paths]
    require(plan['head'] == result['head'] == commit and plan['tracked_status'] == '', 'Wrong actual execution commit or unclean plan')
    require(result['errors'] == [] and result['all_source91_exact'] is True and same(result['source91'], plan['source91']), 'Actual source proof failed')
    require(result['plan_sha256'] == sha(paths['plan']) and terminal['result_sha256'] == sha(paths['result']), 'Plan/result/terminal chain changed')
    require(external['metadata_commit'] == commit and external['controller_terminal']['terminal_sha256'] == sha(paths['terminal']) and post['external_terminal_sha256'] == sha(paths['external_terminal']), 'Execution chain changed')
    require(terminal['all_direct_children_reaped'] is True and terminal['remaining_owned_groups'] == [] and external['controller_exit_code'] == 0 and external['launcher_timeout'] is False and external['controller_terminal']['protocol_complete_under_caps'] is True and post['accepted'] is True, 'Pending, failed or unaccepted execution')
    require(len(result['stages']) == len(plan['commands']) and all(same(stage['argv'], command['argv']) and stage['exit_code'] == 0 and stage['stop_reason'] is None for stage, command in zip(result['stages'], plan['commands'])), 'Actual stage/command failed or differs')
    return paths, value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root-approval', type=Path, required=True)
    parser.add_argument('--root-approval-sha256', required=True)
    args = parser.parse_args()
    require(sys.version_info[:3] == (3, 13, 0) and sys.dont_write_bytecode and sys.flags.optimize == 0, 'Use pinned Python3.13 -B optimization0')
    require(sha(args.root_approval) == args.root_approval_sha256, 'Root approval hash changed')
    approval = read(args.root_approval)
    require(approval['status'] == 'authorized_source_install_and_bind_only' and approval['commit'] == G and approval['actual_workspace_execution_commit'] == F, 'Explicit current root source-only authorization missing')
    require(approval['preparation_freeze_sha256'] == PREP_SHA and approval['independent_harness_review_freeze_sha256'] == REVIEW_SHA and all(approval[k] is True for k in FLAGS), 'Genuine root review/packet approval missing')
    require(git('rev-parse', 'HEAD') == G and git('status', '--porcelain') == '', 'Require untouched clean G checkout')
    require(not DEST.exists() and not DEST.is_symlink(), 'New destination already exists; never replace or retry it')
    require(not (OWN / 'INSTALLATION.json').exists(), 'Installer receipt already exists; no replay')
    require(sha(PREP / 'FREEZE.json') == PREP_SHA and sha(HREVIEW / 'FREEZE.json') == REVIEW_SHA, 'Reviewed preparation changed')
    freeze, review_freeze = read(PREP / 'FREEZE.json'), read(HREVIEW / 'FREEZE.json')
    require(review_freeze['source_ready'] is True and review_freeze['must_fix_findings'] == [] and review_freeze['reviewed_preparation_freeze_sha256'] == PREP_SHA, 'Independent harness review is not source-ready')
    for name, expected in review_freeze['files'].items():
        require(descriptor_matches(HREVIEW / name, expected), 'Independent review bytes/modes changed')
    packet = {name.removeprefix('packet/'): entry for name, entry in freeze['files'].items() if name.startswith('packet/')}
    actual = {str(p.relative_to(PREP / 'packet')) for p in (PREP / 'packet').rglob('*') if p.is_file()}
    require(len(packet) == 33 and actual == set(packet), 'Reviewed packet roster changed')
    for name, expected in packet.items():
        require(descriptor_matches(PREP / 'packet' / name, expected), 'Reviewed packet changed: ' + name)
    fp, fv = chain(FCASE, 'full-001', F)
    gp, gv = chain(GCASE, 'verify-001', G)
    require(sha(fp['result']) == approval['actual_F_result_sha256'] == '77ca14e9ba5e853621d760a104f202694efa9c9adfa5d1e89889dec1fa5291bb', 'Actual F result differs')
    require(sha(gp['result']) == approval['actual_G_result_sha256'] and sha(gp['post_terminal_clock']) == approval['actual_G_post_clock_sha256'], 'Actual G approval differs; no pending receipts accepted')
    require([c['name'] for c in fv['plan']['commands']] == ['full-fmt', 'workspace-tests', 'workspace-clippy', 'native-cli-build'], 'Wrong F four-stage source case')
    stdout = FCASE / 'full-001/workspace-tests.stdout'
    summaries = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', raw(stdout).decode())
    counts = {'summaries': len(summaries), 'passed': sum(int(s[1]) for s in summaries), 'failed': sum(int(s[2]) for s in summaries), 'ignored': sum(int(s[3]) for s in summaries)}
    require(all(s[0] == 'ok' for s in summaries) and counts == fv['result']['test_summary'] == {'summaries': 32, 'passed': 480, 'failed': 0, 'ignored': 0}, 'Actual F raw workspace480/32 proof differs')
    tested = fv['plan']['source91']
    require(len(tested) == 91 and same(tested, gv['plan']['source91']), 'G source91 differs from actual F')
    require(fv['plan']['child_env']['PATH'] == gv['plan']['child_env']['PATH'] and same(fv['plan']['tools'], gv['plan']['tools']), 'Pinned F/G toolchain differs')
    for tool in fv['plan']['tools'].values(): require(sha(tool['path']) == tool['sha256'], 'Pinned compiler/Python changed')
    require(str(Path(sys.executable).resolve()) == fv['plan']['tools']['python']['path'], 'Use the original pinned Python path')
    for name, expected in tested.items():
        path = INT / name
        require(descriptor_matches(path, expected) and hashlib.sha256(subprocess.check_output(['/usr/bin/git', 'show', F + ':' + name], cwd=INT, timeout=10)).hexdigest() == expected['sha256'], 'Actual F-tested input changed: ' + name)
    stage_path = GCASE / 'verify-001/cargo-xtask-verify-RESULT.json'
    stage = read(stage_path)
    require(len(gv['result']['stages']) == 1 and any(same(stage, s) for s in gv['result']['stages']) and Path(stage['argv'][0]).name == 'cargo' and stage['argv'][1:3] == ['xtask', 'verify'] and gv['result']['workspace_Rust_tests_not_repeated'] is True, 'Selected stage is not the actual current G xtask stage')
    require(sha(FCLOSURE) == 'f91d8b7ba6115df532504b7b9a6501eee241ba5ebc4f11b13c0feef0dfc6bef5' and read(FCLOSURE)['source91_exact'] is True, 'Actual F closure changed')
    require(sha(GCLOSURE) == '4c63d7668546c5efca86312ebbc724fafd83674a6f61304dce35e58a9103d64c', 'Actual G closure changed or pending')
    gc = read(GCLOSURE)
    require(gc['accepted'] is True and gc['head'] == G and gc['HEAD_index_tracked_clean'] is True and gc['source91_and478_configs_exact'] is True and gc['result_sha256'] == sha(gp['result']) and gc['external_terminal_sha256'] == sha(gp['external_terminal']) and gc['post_terminal_clock_sha256'] == sha(gp['post_terminal_clock']), 'G clean-after closure differs')
    require(sha(PRODUCTION) == 'dea35d3285566f8b226978618f767d06d043f1022bd77a51cb8ee301941a7f66', 'Independent production review changed')
    spec = importlib.util.spec_from_file_location('reviewed_compiler_inputs', PREP / 'packet/compiler_inputs.py')
    compiler = importlib.util.module_from_spec(spec); spec.loader.exec_module(compiler)
    approved = read(PREP / 'packet/REUSED-SOURCES.json')['producer_compile_inputs']
    require(len(approved) == 3, 'Wrong approved producer roster')
    compiler.validate_compile_coverage(compiler.studio_input_roster(INT), tested, compiler.compiler_input_roster(PREP / 'packet', ('producer',)), approved)
    hashes = {name: entry['sha256'] for name, entry in tested.items()}
    source = dict(read(PREP / 'packet/SOURCE.json'), commit=G, source_count=91, source_hashes=hashes, approval_or_installation=True, status='installed_source_only_F_equivalent_G_not_built_or_bound')
    facts = dict(read(PREP / 'SOURCE-GREEN-FACTS.pending.json'), commit=G, actual_workspace_execution_commit=F, execution_transfer_claimed=False, source_hashes=hashes)
    facts['workspace'] = {name: descriptor(path) for name, path in fp.items()}; facts['workspace']['stdout'] = descriptor(stdout)
    facts['xtask'] = dict(commit=G, clean_before=True, clean_after=True, source_hashes_before=hashes, source_hashes_after=hashes, stage=descriptor(stage_path), **{name: descriptor(path) for name, path in gp.items()})
    files = {name: raw(PREP / 'packet' / name) for name in packet}
    require(all(hashlib.sha256(value).hexdigest() == packet[name]['sha256'] for name, value in files.items()), 'Packet changed after review validation')
    files['SOURCE.json'] = encoded(source)
    files['evidence/CURRENT-SOURCE-PREREQUISITES.json'] = encoded(facts)
    facts_sha = hashlib.sha256(files['evidence/CURRENT-SOURCE-PREREQUISITES.json']).hexdigest()
    source_review = dict(read(PREP / 'SOURCE-REVIEW.pending.json'), commit=G, actual_workspace_execution_commit=F, approved_producer_compile_inputs=approved, source_hashes=hashes, source_prerequisites_sha256=facts_sha, independent_review_sha256=sha(PRODUCTION), **{key: approval[key] for key in FLAGS})
    source_review['root_authorization_scope'] = 'Source install and bind only; no build, initialization, numerical GO or runtime freeze.'
    source_review['root_approval_sha256'] = args.root_approval_sha256
    files['evidence/ROOT-SOURCE-REVIEW.json'] = encoded(source_review)
    copies = {'HARNESS-CAPSULE-CANDIDATE-FREEZE.json': PREP / 'FREEZE.json', 'HARNESS-CAPSULE-INDEPENDENT-REVIEW-FREEZE.json': HREVIEW / 'FREEZE.json', 'HARNESS-CAPSULE-INDEPENDENT-REVIEW-RECEIPT.json': HREVIEW / 'RECEIPT.json', 'HARNESS-CAPSULE-INDEPENDENT-REVIEW.md': HREVIEW / 'REVIEW.md', 'INDEPENDENT-PRODUCTION-REVIEW-002.md': PRODUCTION, 'ROOT-SOURCE-INSTALL-AUTHORIZATION.json': args.root_approval}
    for name, path in copies.items(): files['evidence/' + name] = raw(path)
    evidence = {rel(path): sha(path) for path in [*fp.values(), stdout, *gp.values(), stage_path, FCLOSURE, GCLOSURE]}
    evidence.update({rel(DEST / name): hashlib.sha256(value).hexdigest() for name, value in files.items() if name.startswith('evidence/')})
    candidate = dict(read(PREP / 'packet/CANDIDATE.json'), status='root_reviewed_candidate', checkout=str(INT), source_commit=G, test_source_commit=F, source_manifest=rel(DEST / 'SOURCE.json'), source_manifest_sha256=hashlib.sha256(files['SOURCE.json']).hexdigest(), source_review_evidence=rel(DEST / 'evidence/ROOT-SOURCE-REVIEW.json'), source_review_sha256=hashlib.sha256(files['evidence/ROOT-SOURCE-REVIEW.json']).hexdigest(), green_checks_evidence=rel(DEST / 'evidence/CURRENT-SOURCE-PREREQUISITES.json'), green_checks_sha256=facts_sha, independent_review_sha256=sha(PRODUCTION), evidence_hashes=evidence, root_confirms_green_source_and_review=True, root_install_and_source_bind_authorized=True, approved_parent_path=fv['plan']['child_env']['PATH'], rust_toolchain_bin=str(Path(fv['plan']['tools']['rustc']['path']).parent), workspace_gates_status_at_binding='Actual F39351 fmt/workspace480/Clippy/debug builds passed; actual clean G9395 xtask passed. No execution transfer; optimized/producer builds, source bind and runtime GO remain pending.')
    require(candidate['execution_authorized'] is False and candidate['root_GO'] is None and candidate['producer_binary_sha256'] is None and candidate['optimized_native_cli_build_receipt'] is None, 'Future build/runtime identities must stay pending')
    files['CANDIDATE.json'] = encoded(candidate)
    require(git('rev-parse', 'HEAD') == G and git('status', '--porcelain') == '', 'G changed before installation')
    DEST.mkdir()
    for name, content in files.items():
        path = DEST / name; path.parent.mkdir(parents=True, exist_ok=True)
        with path.open('xb') as stream: stream.write(content); stream.flush(); os.fsync(stream.fileno())
        os.chmod(path, packet[name]['mode'] if name in packet else 0o644)
    require(git('rev-parse', 'HEAD') == G and git('status', '--porcelain') == '', 'Installation changed tracked checkout')
    envelope = {'status': 'installed_source_metadata_only_bind_not_executed', 'source_commit': G, 'actual_workspace_execution_commit': F, 'reviewed_packet_files': 33, 'metadata_derivatives': ['SOURCE.json', 'CANDIDATE.json'], 'programs_and_physics_changed': False, 'installed_hashes': {name: sha(DEST / name) for name in files}, 'bind_argv': [str(Path(sys.executable).resolve()), '-B', str(DEST / 'bind_source.py')], 'bind_cwd': str(INT), 'bind_env_overrides': fv['plan']['child_env'], 'prospective_monitor': {'whole_seconds': 120, 'rss_cap_bytes': 1073741824, 'output_cap_bytes': 8388608, 'free_disk_floor_bytes': 8000000000, 'logs_directory_create_exclusively_before_launch': str(DEST / 'checks/source-bind'), 'policy': 'Root-owned retained monitor; shared absolute deadline through preflight and receipt persistence; kill/reap owned group on failure; no retry.'}, 'bind_executed': False, 'builds': 0, 'constructors': 0, 'runtime_GO_or_freeze_created': False}
    with (OWN / 'INSTALLATION.json').open('xb') as stream: stream.write(encoded(envelope)); stream.flush(); os.fsync(stream.fileno())
    print(json.dumps(envelope, sort_keys=True))


if __name__ == '__main__': main()
