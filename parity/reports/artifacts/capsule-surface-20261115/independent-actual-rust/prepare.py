"""Freeze the existing 83 inputs and exact engine bytes before Rust execution."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import random
import subprocess
import sys
import textwrap

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
PRIOR = ROOT / '.local/capsule-independent-review-20261115'
COMMIT = '24dba7715233a7423088772d49c32a632d901a0c'
PRIOR_MANIFEST_SHA = '1041f3bcee5c3c313c72c590b455fc124bcba2ef268251ccef5e406c91ca4e8a'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify_prior():
    data = (PRIOR / 'MANIFEST.json').read_bytes()
    assert sha(data) == PRIOR_MANIFEST_SHA
    for name, binding in json.loads(data)['files'].items():
        raw = (PRIOR / name).read_bytes()
        assert len(raw) == binding['bytes'] and sha(raw) == binding['sha256'], name


verify_prior()
assert not (OUT / 'PLAN.json').exists(), 'Preparation already frozen'
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('prior_static_checker', PRIOR / 'check_static.py')
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)
prior_source = (PRIOR / 'check_static.py').read_text()
factory = '    cases = []\n' + prior_source.split('    cases = []\n', 1)[1].split('    results, failures = [], []\n', 1)[0]
factory = textwrap.dedent(factory)
context = {'ROOT': ROOT, 'random': random, 'json': json}
exec(compile(factory, '<immutable-prior-case-factory>', 'exec'), context)
cases = context['cases']
expected = json.loads((PRIOR / 'static-check.json').read_text())
assert len(cases) == expected['case_count'] == 83
inputs, frozen = [], []
for (label, triangle, a, b), row in zip(cases, expected['rows']):
    assert label == row['label']
    distance, uncertain, bary, axis_t, scale = prior.transcribed_closest(triangle, a, b)
    assert distance == row['distance_transcribed_f64'] and uncertain == row['uncertain']
    assert bary == row['bary'] and axis_t == row['axis_fraction'] and scale == row['scale']
    inputs.append({'label': label, 'triangle': triangle, 'axis_a': a, 'axis_b': b})
    frozen.append({'label': label, 'independent_qp_distance': row['independent_qp_distance'],
                   'tolerance': row['tolerance'], 'expected_uncertain': row['uncertain']})
(OUT / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
(OUT / 'expected.json').write_text(json.dumps(frozen, indent=2) + '\n')
(OUT / 'case-factory.py.txt').write_text(factory)
(OUT / 'sources').mkdir()
engine = {}
for name, expected_sha in [('math.rs', '4f344da57b5bcb7fb994bb67ae40f3909b7e23814969971dbe457c9094d1b424'),
                            ('capsule.rs', '67bed0887b2a0644dbab7c7003228427c0f9540e5d09a1170961f46cd14e9de7')]:
    raw = subprocess.check_output(['git', 'show', COMMIT + ':crates/weftforge-sim/src/' + name], cwd=ROOT)
    assert sha(raw) == expected_sha
    (OUT / 'sources' / name).write_bytes(raw)
    engine['sources/' + name] = expected_sha
lines = ['#![allow(dead_code)]', 'type Vec3 = [f64; 3];', '#[path = "sources/math.rs"] mod math;',
         '#[path = "sources/capsule.rs"] mod capsule;', 'fn main() {',
         '    let cases: &[(&str, [[f64; 3]; 3], Vec3, Vec3)] = &[']
for row in inputs:
    lines.append('        (' + ', '.join(json.dumps(row[key]) for key in ['label', 'triangle', 'axis_a', 'axis_b']) + '),')
lines += ['    ];', '    for &(label, triangle, a, b) in cases {',
          '        let result = capsule::closest(triangle, a, b);',
          '        println!("{{\\\"label\\\":\\\"{}\\\",\\\"distance\\\":{:.17e},\\\"uncertain\\\":{},\\\"bary\\\":[{:.17e},{:.17e},{:.17e}],\\\"axis_fraction\\\":{:.17e}}}", label, result.distance, result.uncertain, result.bary[0], result.bary[1], result.bary[2], result.axis_fraction);',
          '    }', '}']
(OUT / 'harness.rs').write_text('\n'.join(lines) + '\n')
command = ['rustup', 'run', '1.94.1', 'rustc', '--edition=2024', '--crate-name', 'capsule_static_check',
           'harness.rs', '-o', 'capsule-static-check']
version = subprocess.check_output(['rustup', 'run', '1.94.1', 'rustc', '--version', '--verbose'], cwd=OUT)
assert b'release: 1.94.1\n' in version
(OUT / 'compiler-version.txt').write_bytes(version)
compiler = subprocess.check_output(['rustup', 'which', '--toolchain', '1.94.1', 'rustc'], cwd=OUT).decode().strip()
plan = {'commit': COMMIT, 'case_count': 83, 'compile_command': command, 'execution_command': ['./capsule-static-check'],
        'working_directory': '.', 'rustc_sha256': sha(Path(compiler).read_bytes()), 'engine_sha256': engine,
        'prior_manifest_sha256': PRIOR_MANIFEST_SHA, 'source_factory_sha256': sha(factory.encode()),
        'frozen_files': {p: sha((OUT / p).read_bytes()) for p in ['METHOD.md', 'prepare.py', 'case-factory.py.txt', 'inputs.json', 'expected.json', 'harness.rs', 'compiler-version.txt']}}
(OUT / 'PLAN.json').write_text(json.dumps(plan, indent=2) + '\n')
verify_prior()
print(json.dumps({'prepared': True, 'case_count': 83, 'plan_sha256': sha((OUT / 'PLAN.json').read_bytes()),
                  'harness_sha256': sha((OUT / 'harness.rs').read_bytes()), 'engine_sha256': engine,
                  'prior_report_unchanged': True}))
