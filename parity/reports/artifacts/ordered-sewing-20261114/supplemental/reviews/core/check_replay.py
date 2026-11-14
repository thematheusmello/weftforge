"""Check package integrity and classify the three already-built review binaries."""

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

PACKAGE = Path(__file__).resolve().parent
PRECISION_ERROR = 'called `Result::unwrap()` on an `Err` value: Error("Sewing correspondence exceeds the supported absolute precision budget")'


def accepted(kind, run):
    if kind == "original":
        errors = [float(value) for value in re.findall(r"paired physical error ([0-9.e+-]+)", run.stdout)]
        return (
            run.returncode == 0
            and run.stderr == ""
            and len(errors) == 6
            and max(errors) > 1e-8
            and abs(max(errors) - 5.01998207145255645e-7) < 1e-13
            and "Accepted group: lengths [1.9999999989472883e-7, 5e-6], 3 portions\n" in run.stdout
            and "Mesh accepted: 6 stitches, 41481 vertices\nSave serialization accepted\n" in run.stdout
        )
    if kind == "rejected":
        return (
            run.returncode == 101
            and run.stdout == ""
            and "thread 'main'" in run.stderr
            and "panicked at " in run.stderr
            and PRECISION_ERROR in run.stderr
        )
    expected = PACKAGE / "results" / f"{kind}.stdout.txt"
    return run.returncode == 0 and run.stderr == "" and run.stdout == expected.read_text()


def self_check():
    def result(code=0, out="", err=""):
        return subprocess.CompletedProcess([], code, out, err)

    original = (PACKAGE / "results/70897ee-counterexample.stdout.txt").read_text()
    rejection = "thread 'main' panicked at src/main.rs:23:8:\n" + PRECISION_ERROR
    assert accepted("original", result(out=original))
    assert not accepted("original", result(1, original))
    assert not accepted("original", result(out=original.replace("5.01998207145255645e-7", "5.01998207145255645e-10")))
    assert accepted("rejected", result(101, err=rejection))
    assert not accepted("rejected", result(101, err="error: could not compile\n" + PRECISION_ERROR))
    assert not accepted("rejected", result(101, err=rejection.replace("Sewing correspondence", "Unrelated validation")))
    assert not accepted("rejected", result(0, err=rejection))
    for kind in ["9cd0a21-precision_guard", "ownership"]:
        output = (PACKAGE / "results" / f"{kind}.stdout.txt").read_text()
        assert accepted(kind, result(out=output))
        assert not accepted(kind, result(101, output))


def verify_package():
    manifest = json.loads((PACKAGE / "manifest.json").read_text())
    expected = {"manifest.json"}
    for item in manifest["files"]:
        relative = Path(item["path"])
        assert not relative.is_absolute() and ".." not in relative.parts
        path = PACKAGE / relative
        assert path.is_file() and not path.is_symlink(), relative
        data = path.read_bytes()
        assert len(data) == item["bytes"], relative
        assert hashlib.sha256(data).hexdigest() == item["public_sha256"], relative
        private_prefixes = [b"/" + b"Users/", b"/private/" + b"tmp/", b"/tmp/" + b"weftforge-"]
        assert not any(prefix in data for prefix in private_prefixes), relative
        expected.add(item["path"])
    actual = {str(path.relative_to(PACKAGE)) for path in PACKAGE.rglob("*") if path.is_file()}
    assert actual == expected
    assert len(actual) == manifest["file_count_including_manifest"]
    return len(actual)


if __name__ == "__main__":
    file_count = verify_package()
    self_check()
    if sys.argv[1:] == ["--self-test"]:
        print(f"Package integrity and replay classifier self-check passed ({file_count} files; no builds run).")
    else:
        assert len(sys.argv) == 3 and sys.argv[1] in ["original", "fixed"], "Usage: check_replay.py original|fixed CARGO_TARGET_DIR"
        mode, target = sys.argv[1], Path(sys.argv[2]).resolve()
        runs = [("weftforge-chain-core-review", "original" if mode == "original" else "rejected")]
        if mode == "fixed":
            runs.append(("precision_guard", "9cd0a21-precision_guard"))
        runs.append(("ownership", "ownership"))
        for binary, kind in runs:
            run = subprocess.run([str(target / "debug" / binary)], capture_output=True, text=True)
            print(f"{binary}: exit {run.returncode}; expected {kind}")
            print(run.stdout, end="")
            print(run.stderr, end="")
            assert accepted(kind, run), f"Unexpected result from {binary}; this is not an accepted precision rejection"
        print(f"{mode} core review replay passed; no native/reference parity claim.")
