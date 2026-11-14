"""Replay saved authoring and incomplete-prefix evidence, never product physics."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
BASE = Path(__file__).resolve().parent
SOURCE = "6c45535cb312f7fed8ad872ee87b6ba5e5974e35"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_bytes())


def replay(path, *args):
    result = subprocess.run(
        [sys.executable, "-B", str(path), *map(str, args)],
        cwd=path.parent, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
        text=True, capture_output=True, check=True, timeout=60,
    )
    assert not result.stderr, result.stderr
    return json.loads(result.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-only", action="store_true")
    args = parser.parse_args()
    manifest = read(BASE / "MANIFEST.json")
    actual = {p.relative_to(BASE).as_posix() for p in BASE.rglob("*") if p.is_file()}
    assert actual == set(manifest["files"]) | {"MANIFEST.json"}
    private_prefixes = [b"/" + value for value in (b"Users/", b"home/", b"private/var/", b"Volumes/")]
    for name, record in manifest["files"].items():
        path = BASE / name
        assert not path.is_symlink() and not Path(name).is_absolute() and ".." not in Path(name).parts
        assert not any(part in ("target", ".git", "__pycache__") for part in Path(name).parts)
        assert path.stat().st_size == record["bytes"] and sha(path) == record["sha256"], name
        assert not any(prefix in path.read_bytes() for prefix in private_prefixes), name
    provenance = read(BASE / "PROVENANCE.json")
    assert provenance["runtime_source_commit"] == SOURCE
    for subtree in provenance["subtrees"]:
        path = BASE / subtree["subtree"]
        assert sha(path / subtree["frozen_manifest"]) == subtree["sha256"]
        assert sum(p.is_file() for p in path.rglob("*")) == subtree["copied_files"]
        assert subtree["all_bytes_identical"]
    source = read(BASE / provenance["source_pretest"])
    assert sha(BASE / provenance["source_pretest"]) == provenance["source_pretest_sha256"]
    assert source["source_commit"] == SOURCE and len(source["source_hashes"]) == 37
    git_state = "candidate preparation; clean/tracked publication checks skipped"
    if not args.candidate_only:
        repository = Path(subprocess.check_output(["git", "-C", str(BASE), "rev-parse", "--show-toplevel"], text=True).strip())
        assert not subprocess.check_output(["git", "-C", str(repository), "status", "--porcelain", "--untracked-files=all"])
        relative = BASE.relative_to(repository)
        tracked = set(subprocess.check_output(["git", "-C", str(repository), "ls-files", "-z", "--", relative.as_posix()]).decode().split("\0")) - {""}
        assert tracked == {(relative / name).as_posix() for name in actual}
        subprocess.run(["git", "-C", str(repository), "merge-base", "--is-ancestor", SOURCE, "HEAD"], check=True)
        for name, expected in source["source_hashes"].items():
            data = subprocess.check_output(["git", "-C", str(repository), "show", SOURCE + ":" + name])
            assert hashlib.sha256(data).hexdigest() == expected, name
        git_state = "clean checkout; complete tracked closure; 37 historical committed source hashes verified"
    authoring = BASE / "native-authoring"
    replay(authoring / "verify_candidate.py")
    authored = replay(authoring / "verify_authoring.py")
    assert authored == read(authoring / "authoring-verification.json")
    attempt = replay(BASE / "native-attempt/verify_attempt.py")
    assert attempt["A_accepted_frames"] == 60 and attempt["B"] == "not_run"
    headless = BASE / "headless-attempt"
    replay(headless / "verify.py")
    with tempfile.TemporaryDirectory(prefix="harbor-retained-review-") as temporary:
        replay(headless / "partial-review/qualify_prefix.py", headless, temporary, headless / "native/base")
        expected_paths = {p.name for p in (headless / "partial-review/results").glob("*.json")}
        assert {p.name for p in Path(temporary).glob("*.json")} == expected_paths
        for name in expected_paths:
            expected, observed = read(headless / "partial-review/results" / name), read(Path(temporary) / name)
            expected.pop("completed_utc", None)
            observed.pop("completed_utc", None)
            assert expected == observed, name
    images = read(BASE / "root-qualification/image-metadata-review.json")["images"]
    assert len(images) == 45
    for record in images:
        path = BASE / record["path"]
        assert sha(path) == record["sha256"] and path.read_bytes().startswith(b"\xff\xd8\xff")
        assert record["EXIF"] == {"34665": 26}
        assert record["EXIF_IFD"] == {"40962": record["dimensions"][0], "40963": record["dimensions"][1]}
    print(json.dumps({"git": git_state, "files": len(actual), "runtime_source": SOURCE,
                      "authoring": "bounded checks passed", "prefix_states_checked": 323,
                      "physical_completion": "failed caps; no frame180; nativeB not_run",
                      "overall_quality": "not qualified", "reference_parity": False,
                      "new_physics_or_UI": False}, indent=2))


if __name__ == "__main__":
    main()
