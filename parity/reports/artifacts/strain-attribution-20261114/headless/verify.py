"""Verify public artifact closure and replay saved states; no simulator is used."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

base = Path(__file__).resolve().parent
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
manifest = json.loads((base / 'MANIFEST.json').read_text())
actual = {p.relative_to(base).as_posix() for p in base.rglob('*') if p.is_file()}
assert actual == set(manifest['files']) | {'MANIFEST.json'}, 'artifact closure mismatch'
for name, expected in manifest['files'].items():
    path = base / name
    assert not path.is_symlink() and '..' not in Path(name).parts and not Path(name).is_absolute()
    assert digest(path) == expected, name
    content = path.read_bytes()
    private_roots = [b'/' + part + b'/' for part in [b'Users', b'home', b'private/var/folders']]
    assert not any(root in content for root in private_roots), name
source = json.loads((base / 'source-manifest.json').read_text())
assert len(source['files']) == 37
for name, expected in source['files'].items():
    assert digest(base / 'runtime-source' / name) == expected, name
binding = json.loads((base / 'binary-binding.json').read_text())
provenance = json.loads((base / 'PROVENANCE.json').read_text())
for stage in ['build', 'clippy']:
    name = stage + '-job.json'
    assert provenance['raw_file_sha256'][name] == binding[stage + '_job_sha256']
for name, entry in provenance['path_only_derivatives'].items():
    assert digest(base / name) == entry['published_sha256']
    assert entry['raw_sha256'] == provenance['raw_file_sha256'][name]
result = json.loads(subprocess.check_output([sys.executable, '-B', str(base / 'replay_acceptance.py'), str(base)],
                                           env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'}, text=True))
assert result['positions_velocities_settings_all_prior_diagnostics_exact']
assert not result['original_assembly_quality_passed'] and not result['original_final_quality_passed']
print(json.dumps({'artifact_files': len(manifest['files']), 'source_files': len(source['files']),
                  'saved_output_replay_passed': True, 'new_simulations_from_verifier': 0,
                  'source_commit': result['source_commit'], 'meaningful_attribution_frames': result['independent_attribution_all_meaningful_frames_passed'],
                  'max_absolute_errors': result['max_absolute_errors'], 'public_replay_source_state': result['source_verification'],
                  'public_replay_binary_state': result['binary_verification'], 'original_quality_failures_preserved': True}, indent=2))
