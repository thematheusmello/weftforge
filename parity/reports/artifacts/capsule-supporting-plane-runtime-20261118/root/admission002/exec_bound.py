"""Enter the frozen compiler/Python parent environment before external admission."""
import hashlib
import json
import os
from pathlib import Path

OUT = Path(__file__).resolve().parent


def bound_environment(binding):
    env = dict(os.environ)
    for key in list(env):
        if key.startswith(('CARGO_BUILD_', 'CARGO_PROFILE_', 'CARGO_TARGET_')):
            del env[key]
    env.update(binding['child_env'])
    env.update(binding['ambient_build_env'])
    env['PATH'] = binding['parent_path']
    actual = {key: value for key, value in sorted(env.items())
              if key.startswith(('CARGO_BUILD_', 'CARGO_PROFILE_', 'CARGO_TARGET_'))}
    assert actual == binding['ambient_build_env'], 'Bound ambient compiler context differs'
    assert all(env[key] == value for key, value in binding['child_env'].items()), 'Bound child context differs'
    return env


if __name__ == '__main__':
    admission = json.loads((OUT / 'ROOT-ADMISSION.json').read_text())
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == admission['bound_parent_launcher_sha256']
    packet = Path(admission['packet'])
    raw = (packet / 'SOURCE-PREFLIGHT.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest() == admission['source_preflight_sha256']
    preflight = json.loads(raw)
    env = bound_environment(preflight['build_binding'])
    python = preflight['python']['binary_path']
    assert hashlib.sha256(Path(python).read_bytes()).hexdigest() == preflight['python']['binary_sha256']
    os.execve(python, [python, '-B', str(OUT / 'launch_and_record.py')], env)
