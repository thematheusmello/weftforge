"""Compare the packaged CLI's Harbor results to the original native-project protocol."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ORIGINAL = HERE.parent / "native-harbor-studio-20261122"
TIMING = {
    "median_step_seconds", "p95_step_seconds", "step_seconds", "wall_seconds",
    "simulated_seconds_per_wall_second",
}


def physical_result(value):
    if isinstance(value, list):
        return [physical_result(item) for item in value]
    if isinstance(value, dict):
        return {key: physical_result(item) for key, item in value.items() if key not in TIMING}
    return value


def main():
    protocol = json.loads((HERE / "fit-replay-protocol.json").read_bytes())
    for case, filename in [("original", "Harbor authored.weftforge"), ("reopened", "Harbor 670.weftforge")]:
        fixture = (ORIGINAL / filename).read_bytes()
        assert hashlib.sha256(fixture).hexdigest() == protocol["cases"][case]
        actual = json.loads((HERE / f"{case}-fit.json").read_bytes())
        prior = json.loads((ORIGINAL / f"{case}-trajectory.json").read_bytes())
        assert actual["quality_states_checked"] == 181 and actual["first_quality_failure_state"] == 28
        assert actual["status"] == "failed" and actual["solver_error"] is None
        assert physical_result(actual) == physical_result(prior)
        strain = actual["first_quality_failure"]["diagnostics"]["max_stretch_error"]
        print(f"{case}: {actual['quality_states_checked']} states, first strain failure at 28 ({strain:.4%})")


if __name__ == "__main__":
    main()
