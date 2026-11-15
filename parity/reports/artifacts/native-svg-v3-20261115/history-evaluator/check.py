#!/usr/bin/env python3
"""Prospective v3 checkpoint supplement; uses only the frozen independent checker."""
import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ORIGINAL_CHECKER = ROOT / ".local/svg-native-evaluator-20261115"
CASE = ROOT / ".local/native-svg-acceptance-v3-20261115"
PINNED_CHECKER = "a015c01c864533d2f4e3b872da78c9fc72892b7b8f54a79aae9544fe5c984c6d"
PINNED_MANIFEST = "bb25bd15266c810ba2bc844f68595a586e5e202db9311f24926e2cd26002c24f"
REVISIONS = {"baseline": 1, "imported": 2, "undo": 3, "redo": 4,
             "edited": 5, "rejected": 5, "reopened": 5}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def independent_checker():
    require(digest(ORIGINAL_CHECKER / "check.py") == PINNED_CHECKER, "original independent checker changed")
    require(digest(ORIGINAL_CHECKER / "FROZEN.json") == PINNED_MANIFEST, "original evaluator freeze changed")
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("frozen_original_svg_evaluator", ORIGINAL_CHECKER / "check.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.frozen()
    return module


def expectations(checker):
    baseline = checker.originals()
    require(baseline["project"]["revision"] == 1 and baseline["project"]["next_id"] == 11, "wrong prospective baseline")
    old, edited_geometry, _ = checker.expected_geometry(baseline)
    source, _ = checker.read_svg((checker.ORIGINALS / "original-editable-patterns.svg").read_bytes())
    imported_geometry = checker.native_geometry(old["project"]) + source
    allocation = [{"name": name, "pieces": 1, "points": len({span[0] for span in spans}), "curves": len(spans)}
                  for name, spans in source]
    next_id = baseline["project"]["next_id"] + sum(p["pieces"] + p["points"] + p["curves"] for p in allocation)
    require(next_id == 29 and len(allocation) == 2, "unexpected analytic allocation")
    return baseline, old, imported_geometry, edited_geometry, next_id, allocation


def changed_revision(document, revision):
    result = copy.deepcopy(document)
    result["project"]["revision"] = revision
    return result


def first_cubic(project):
    front = project["pieces"][1]
    curve_id = front["boundary"][1]
    matches = [curve for curve in project["curves"] if curve["id"] == curve_id]
    require(len(matches) == 1 and matches[0]["controls"] is not None, "first imported cubic missing")
    return matches[0]


def check_history(checker, documents, expected):
    baseline, old, imported_geometry, edited_geometry, next_id, _ = expected
    checker.exact(documents["baseline"], baseline, "baseline checkpoint")
    for name, revision in REVISIONS.items():
        project = documents[name]["project"]
        checker.exact(project["revision"], revision, f"{name} revision")
        checker.exact(project["next_id"], 11 if name == "baseline" else next_id, f"{name} allocator")
    checker.check_native(documents["imported"], baseline, old, imported_geometry)
    undo_expected = changed_revision(baseline, 3)
    undo_expected["project"]["next_id"] = next_id
    checker.exact(documents["undo"], undo_expected, "undo baseline authoring and high-water allocator")
    checker.exact(documents["redo"], changed_revision(documents["imported"], 4), "redo imported authoring and IDs")
    checker.check_native(documents["edited"], baseline, old, edited_geometry)
    edit_expected = changed_revision(documents["imported"], 5)
    before = first_cubic(edit_expected["project"])
    after = first_cubic(documents["edited"]["project"])
    require(abs(after["controls"][0][0] - before["controls"][0][0] - .01) <= checker.POINT_TOLERANCE,
            "first cubic control X did not change by exactly .01 m within tolerance")
    before["controls"][0][0] = after["controls"][0][0]
    checker.exact(documents["edited"], edit_expected, "only cubic control X and declared revision changed after redo")
    for name in ("rejected", "reopened"):
        checker.exact(documents[name], documents["edited"], f"{name} exactly preserves edited checkpoint")


def synthetic_history(checker, expected):
    baseline, old, imported_geometry, _, next_id, _ = expected
    imported = checker.synthetic_native(baseline, old, imported_geometry)
    project = imported["project"]
    old_ids = {obj["id"] for key in ("points", "curves", "pieces", "materials") for obj in baseline["project"][key]}
    new_ids = sorted(obj["id"] for key in ("points", "curves", "pieces") for obj in project[key] if obj["id"] not in old_ids)
    # Permutation proves the checker does not predict newly allocated stable-ID values.
    mapping = dict(zip(new_ids, reversed(range(11, next_id))))
    for key in ("points", "curves", "pieces"):
        for obj in project[key]:
            obj["id"] = mapping.get(obj["id"], obj["id"])
            if key == "curves":
                for endpoint in ("start", "end"):
                    obj[endpoint] = mapping.get(obj[endpoint], obj[endpoint])
            if key == "pieces":
                obj["boundary"] = [mapping.get(i, i) for i in obj["boundary"]]
    project["revision"], project["next_id"] = 2, next_id
    undo = changed_revision(baseline, 3)
    undo["project"]["next_id"] = next_id
    edited = changed_revision(imported, 5)
    first_cubic(edited["project"])["controls"][0][0] = .09
    return {"baseline": copy.deepcopy(baseline), "imported": imported, "undo": undo,
            "redo": changed_revision(imported, 4), "edited": edited,
            "rejected": copy.deepcopy(edited), "reopened": copy.deepcopy(edited)}


def self_checks(checker, expected):
    good = synthetic_history(checker, expected)
    check_history(checker, good, expected)
    rejected = []

    def reject(label, action):
        try:
            action()
        except (ValueError, KeyError, TypeError, IndexError):
            rejected.append(label)
        else:
            raise ValueError(f"self-check did not reject {label}")

    def mutate(name, action):
        wrong = copy.deepcopy(good)
        action(wrong[name]["project"])
        check_history(checker, wrong, expected)

    reject("undo incorrectly restores allocator to 11", lambda: mutate("undo", lambda p: p.__setitem__("next_id", 11)))
    reject("wrong import allocator", lambda: mutate("imported", lambda p: p.__setitem__("next_id", 30)))
    reject("wrong surviving original ID", lambda: mutate("undo", lambda p: p["points"][0].__setitem__("id", 30)))
    reject("redo changes imported stable ID", lambda: mutate("redo", lambda p: p["points"][4].__setitem__("id", 99)))
    reject("wrong imported scale", lambda: mutate("imported", lambda p: p["points"][4]["position"].__setitem__(0, -.08)))
    reject("wrong edited cubic geometry", lambda: mutate("edited", lambda p: first_cubic(p)["controls"][0].__setitem__(0, .091)))
    reject("missing imported piece", lambda: mutate("imported", lambda p: p["pieces"].pop()))
    reject("rejected input changes material", lambda: mutate("rejected", lambda p: p["materials"][0].__setitem__("friction", .5)))
    reject("reopen changes authoring", lambda: mutate("reopened", lambda p: p["points"][0]["position"].__setitem__(0, .51)))
    for name in REVISIONS:
        reject(f"changed {name} revision", lambda name=name: mutate(name, lambda p: p.__setitem__("revision", p["revision"] + 1)))
    for changed in checker.HASHES:
        with tempfile.TemporaryDirectory(prefix="weftforge-history-self-check-") as temporary:
            directory = Path(temporary)
            for name in checker.HASHES:
                (directory / name).write_bytes((checker.ORIGINALS / name).read_bytes() + (b"\n" if name == changed else b""))
            reject(f"original source edit: {changed}", lambda: checker.originals(directory))
    return {"status": "SELF_CHECKS_PASSED", "native_v3_acceptance": "NOT_RUN", "synthetic_positive_case": True,
            "synthetic_ids": "permuted fresh IDs, not predicted allocation order", "rejected_cases": rejected,
            "note": "No v2 or v3 native outputs read; these are synthetic checker tests only."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true")
    for name in REVISIONS:
        parser.add_argument("--" + name, type=Path)
    args = parser.parse_args()
    checker = independent_checker()
    expected = expectations(checker)
    paths = {name: getattr(args, name) for name in REVISIONS}
    if args.self_check:
        require(not any(paths.values()), "self-check cannot grade native artifacts")
        return self_checks(checker, expected)
    require(all(paths.values()), "all seven actual v3 checkpoint paths are required; missing artifacts never pass")
    require(len({p.resolve() for p in paths.values()}) == len(paths), "checkpoint paths must be distinct")
    require(all(p.resolve().is_relative_to(CASE.resolve()) for p in paths.values()), "supplement is prospectively bound to native-svg-acceptance-v3-20261115, never v2")
    manifest = checker.load_json(HERE / "FROZEN.json")
    for name, sha in manifest["supplement_sha256"].items():
        require(digest(HERE / name) == sha, f"frozen supplement changed: {name}")
    documents = {name: checker.load_json(path) for name, path in paths.items()}
    check_history(checker, documents, expected)
    checker.originals()
    return {"status": "STATIC_HISTORY_CHECKS_PASSED", "frozen_utc": manifest["frozen_utc"],
            "supplement_sha256": manifest["supplement_sha256"], "original_checker_sha256": PINNED_CHECKER,
            "actual_checkpoint_sha256": {name: digest(path) for name, path in paths.items()},
            "declared_revisions": REVISIONS, "allocator_baseline": 11, "allocator_after_import_and_history": expected[4],
            "analytic_allocation": expected[5], "exact_relations": ["baseline=original", "undo=baseline except revision3/next_id29",
            "redo=imported except revision4", "edited=imported except first cubic control X+.01m and revision5", "rejected=edited", "reopened=edited"],
            "native_action_execution": "Requires separate native trace review, including Cancel/pending draft, actual Undo/Redo and actual quit/relaunch",
            "final_svg_checks": "Must also run unchanged original evaluator on final v3 edited/reopened/exported SVG artifacts",
            "v2": "Not read, changed or regraded; original strict failure remains separate evidence"}


if __name__ == "__main__":
    try:
        report = main()
    except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
        print(json.dumps({"status": "FAILED", "error": str(error), "native_v3_acceptance": "NOT_ESTABLISHED"}, indent=2))
        sys.exit(1)
    print(json.dumps(report, indent=2))
