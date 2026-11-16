"""New synthetic fixtures only: no Harbor initialization, steps or old-frame grading."""
import ast
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
from unittest import mock

import common as b
from common import ROOT, REPO, c, r
import capsule_geometry
import metadata_adapter
import worker


def rejected(call):
    try:
        call()
    except (ValueError, AssertionError):
        return True
    raise AssertionError("Malformed synthetic input was accepted")


def main():
    results = []
    fixtures = ROOT / "checks/synthetic"
    fixtures.mkdir()
    env = b.environment()
    b.validate_environment(env)
    for field, wrong in [("version", [3, 13, 1]), ("sum_probe", 1.), ("optimize", 1), ("platform", "Linux")]:
        changed = dict(env, **{field: wrong})
        rejected(lambda changed=changed: b.validate_environment(changed))
    results.append({"check": "exact_interpreter_arithmetic_and_no_optimization_guard", "passed": True})

    local_names = {p.stem for p in ROOT.glob("*.py")} | {p.stem for p in (ROOT / "consumer").glob("*.py")}
    for path in list(ROOT.glob("*.py")) + list((ROOT / "consumer").glob("*.py")):
        for node in ast.walk(ast.parse(path.read_text())):
            imports = [q.name.split('.')[0] for q in node.names] if isinstance(node, ast.Import) else [node.module.split('.')[0]] if isinstance(node, ast.ImportFrom) and node.module else []
            for name in imports:
                assert name in sys.stdlib_module_names or name in local_names, (path.name, name)
    assert hashlib.sha256((ROOT / "consumer/static_geometry.py").read_bytes()).hexdigest() == "c4583bbda9250cfa32cdc305c1b21563c26ca46ddb75032f5c2ae4d128299b34"
    c.metrics.self_check()
    triangle = [[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]]
    for capsule, exact in [({"a": [0., 0., 2.], "b": [0., 0., 3.], "radius": .2}, 2.),
                           ({"a": [0., 0., -1.], "b": [0., 0., 1.], "radius": .2}, 0.)]:
        interval = capsule_geometry.distance_interval(triangle, capsule)
        assert interval["distance_lower_m"] <= exact <= interval["distance_upper_m"]
        assert not interval["uncertain"]
    results.append({"check": "complete_import_closure_and_independent_new_analytic_geometry_fixtures", "passed": True})

    legacy = {"pieces": [{"id": 2}, {"id": 31}]}
    upgraded = copy.deepcopy(legacy)
    metadata_adapter.extend_empty_piece_fields(upgraded)
    assert upgraded == {"pieces": [{"id": 2, "holes": [], "internal_curves": []}, {"id": 31, "holes": [], "internal_curves": []}]}
    rejected(lambda: metadata_adapter.extend_empty_piece_fields(upgraded))
    rejected(lambda: metadata_adapter.extend_empty_piece_fields({"pieces": [{"id": 2}]}))
    assert not c.same(0., -0.) and not c.same(0, 0.)
    rejected(lambda: c.strict_loads('{"x":1,"x":2}'))
    rejected(lambda: c.strict_loads('{"x":NaN}'))
    meta = {"piece_ids": [1, 1, 1], "rest_positions_m": [[0., 1., 0.], [1., 1., 0.], [0., 2., 0.]]}
    initial = {"positions_m": meta["rest_positions_m"], "velocities_m_per_s": [[0., 0., 0.]]*3,
               "simulation_time_s": 0., "diagnostics": {"finite": True, "simulation_time": 0.},
               "cloth_step_calls": 0, "metadata_compared_before_steps": False}
    worker.initialized_state(meta, initial, False)
    for field, value in [("cloth_step_calls", 1), ("simulation_time_s", 1/60), ("metadata_compared_before_steps", True)]:
        rejected(lambda field=field, value=value: worker.initialized_state(meta, dict(initial, **{field: value}), False))
    changed = copy.deepcopy(initial); changed["velocities_m_per_s"][0][0] = -0.
    rejected(lambda: worker.initialized_state(meta, changed, False))
    results.append({"check": "strict_empty_field_adapter_numeric_types_signed_zero_and_zero_step_gate", "passed": True})

    project = {"revision": 0, "points": [{"id": 3, "position": [0., 1.]}, {"id": 4, "position": [1., 1.]}, {"id": 5, "position": [0., 2.]}],
               "curves": [{"id": i, "start": a, "end": z, "controls": None} for i, a, z in [(6, 3, 4), (7, 4, 5), (8, 5, 3)]],
               "pieces": [{"id": 2, "name": "New analytic triangle", "boundary": [6, 7, 8], "holes": [], "internal_curves": [], "placement": {"translation": [0., 0., 0.], "rotation": [0., 0., 0.]}, "material": 1, "grain_angle": 0.}],
               "materials": [{"id": 1, "name": "Synthetic known areal mass", "areal_density": .2, "warp_stiffness": 400., "weft_stiffness": 400., "shear_stiffness": 100., "bending_stiffness": .003, "damping": .2, "friction": .4, "collision_thickness": .003, "color": [.3, .4, .5]}],
               "seams": [], "sewing_groups": [], "mannequin": {"capsules": []}, "resolution": 1., "next_id": 9}
    input_path = fixtures / "new-triangle.weftforge"
    b.persist(input_path, {"format": "weftforge", "version": 5, "project": project})
    binary = str(REPO / "target/release/harbor-coupled-producer")

    def producer_check(name, argv):
        return r.monitor(argv, fixtures, fixtures / (name+".stdout"), fixtures / (name+".stderr"),
                         time.monotonic()+5., lambda: sum(p.stat().st_size for p in fixtures.rglob('*') if p.is_file()),
                         rss_cap=b.CAP_BYTES, output_cap=b.CAP_BYTES)

    prepared = fixtures / "prepared"
    result = producer_check("prepare", [binary, "prepare", str(input_path), str(prepared)])
    assert result["exit_code"] == 0 and result["stop_reason"] is None, result
    prepared_meta = b.read(prepared / "rest-metadata.json")
    prepared_initial = b.read(prepared / "initialized-state.json")
    worker.initialized_state(prepared_meta, prepared_initial, False)
    assert abs(sum(prepared_meta["masses_kg"]) - .1) < 1e-12
    approved = fixtures / "approved-synthetic.json"
    b.persist(approved, prepared_meta)
    positive = producer_check("approval-positive", [binary, "check", str(input_path), str(fixtures / "checked"), str(approved)])
    assert positive["exit_code"] == 0 and positive["stop_reason"] is None, positive
    worker.initialized_state(prepared_meta, b.read(fixtures / "checked/initialized-state.json"), True)
    guards = []
    for name, replacement in [("signed-zero", -0.), ("numeric-type", 0)]:
        bad = copy.deepcopy(prepared_meta)
        assert c.same(bad["rest_positions_m"][0][0], 0.)
        bad["rest_positions_m"][0][0] = replacement
        path = fixtures / (name+"-approved.json"); b.persist(path, bad)
        output = fixtures / (name+"-must-not-exist")
        negative = producer_check(name, [binary, "check", str(input_path), str(output), str(path)])
        assert negative["exit_code"] != 0 and not output.exists(), negative
        guards.append({"case": name, **negative})
    assert not any(fixtures.rglob('trajectory.jsonl')) and not any(fixtures.rglob('steps.jsonl'))
    results.append({"check": "actual_release_synthetic_init_and_approval_only_no_steps", "passed": True,
                    "init": result, "positive": positive, "negative_guards": guards, "known_mass_kg": .1})

    # Real new child/grandchild fixtures exercise the exact retained process-group monitor.
    code = "import subprocess,sys,time; p=subprocess.Popen([sys.executable,'-B','-c','import time;time.sleep(3)']); print(p.pid,flush=True); time.sleep(3)"
    timed = r.monitor([sys.executable, '-B', '-c', code], fixtures, fixtures/'timeout.stdout', fixtures/'timeout.stderr',
                      time.monotonic()+.25, lambda: 0, rss_cap=b.CAP_BYTES)
    assert timed['stop_reason'] == 'whole_case_wall_cap' and timed['exit_code'] == -signal.SIGKILL, timed
    grandchild = int((fixtures/'timeout.stdout').read_text().strip())
    status = subprocess.run(['/bin/ps', '-o', 'state=', '-p', str(grandchild)], capture_output=True, text=True).stdout.strip()
    assert not status or status.startswith('Z'), status
    overflow = fixtures/'synthetic-overflow.bin'
    output = r.monitor([sys.executable, '-B', '-c', f"from pathlib import Path;import time;Path({str(overflow)!r}).write_bytes(b'x'*8192);time.sleep(3)"],
                       fixtures, fixtures/'overflow.stdout', fixtures/'overflow.stderr', time.monotonic()+1.,
                       lambda: overflow.stat().st_size if overflow.exists() else 0, rss_cap=b.CAP_BYTES, output_cap=4096, reserve=128)
    assert output['stop_reason'] == 'numerical_output_cap_stop_budget' and output['exit_code'] == -signal.SIGKILL, output
    for name in r.OUTPUT_DIRECTORIES:
        (fixtures / 'volume' / name).mkdir(parents=True)
        (fixtures / 'volume' / name / 'synthetic.bin').write_bytes(b'x'*17)
    assert r.numerical_bytes(fixtures/'volume') == len(r.OUTPUT_DIRECTORIES)*17
    results.append({"check": "real_timeout_process_group_output_cap_and_all_numerical_paths", "passed": True,
                    "timeout": timed, "grandchild_pid": grandchild, "grandchild_state_after_kill": status, "overflow": output})

    # Actual receipt I/O must finish before the terminal clock; delayed persistence cannot pass.
    small = fixtures/'closing-pass'; (small/'runtime').mkdir(parents=True)
    payload = {"status": "pending_post_persistence_terminal_verdict", "strict_quality_passed": True, "started": time.monotonic()}
    with mock.patch.object(b, 'ROOT', small), mock.patch.object(r, 'numerical_bytes', lambda: 100):
        terminal = b.close_supervisor(payload, time.monotonic()+2.)
    assert terminal['status'] == 'complete' and (small/'runtime/SUPERVISOR.json').exists()
    slow = fixtures/'closing-fail'; (slow/'runtime').mkdir(parents=True)
    real_persist = b.persist
    def delayed_persist(path, value, deadline=None):
        digest = real_persist(path, value)
        time.sleep(.05)
        return digest
    with mock.patch.object(b, 'ROOT', slow), mock.patch.object(b, 'persist', delayed_persist), mock.patch.object(r, 'numerical_bytes', lambda: 100):
        rejected(lambda: b.close_supervisor(payload, time.monotonic()+.02))
    assert b.read(slow/'runtime/SUPERVISOR.json')['status'] == 'pending_post_persistence_terminal_verdict'
    results.append({"check": "post_persistence_clock_rejects_delayed_receipt_without_success_verdict", "passed": True})

    assert not any((ROOT / name).exists() for name in r.OUTPUT_DIRECTORIES)
    refusal = subprocess.run([sys.executable, '-B', str(ROOT/'controller.py'), '--go', 'MISSING-ROOT-GO.json', '--go-sha256', '0'*64], cwd=ROOT, capture_output=True, text=True)
    (fixtures/'go-refusal.stdout').write_text(refusal.stdout); (fixtures/'go-refusal.stderr').write_text(refusal.stderr)
    assert refusal.returncode == 1 and not any((ROOT / name).exists() for name in r.OUTPUT_DIRECTORIES)
    results.append({"check": "real_command_refuses_without_root_GO_no_case_started", "passed": True})
    report = {"status": "passed", "synthetic_checks": results, "producer_binary_sha256": r.sha(binary),
              "environment": env, "Harbor_initialized": False, "Cloth_step_calls": 0,
              "old_saved_frames_read_or_graded": False, "real_case_clock_started": False,
              "tested_sources": {str(p.relative_to(ROOT)): r.sha(p) for p in list(ROOT.glob('*.py'))+list((ROOT/'consumer').glob('*.py'))}}
    b.persist(ROOT/'checks/SELF-CHECKS.json', report)
    print(json.dumps({"status": "passed", "checks": len(results), "Harbor_initialized": False, "Cloth_step_calls": 0}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
