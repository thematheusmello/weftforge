"""Exact current metadata validation, with the declared empty v4 annotation fields."""
from pathlib import Path

import consumer16 as c

ROOT = Path(__file__).resolve().parent


def expected():
    binding = c.read(ROOT / "READONLY-BINDING.json")
    repo = Path(binding["repo"])
    original = repo / "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/runs/base/rest-metadata.json"
    c.require(c.sha(original) == binding["files"][str(original.relative_to(repo))], "Original rest metadata changed")
    declared = c.read(ROOT / "expected/current-rest.json")
    derived = c.read(original)
    for piece in derived["project"]["pieces"]:
        c.require("internal_curves" not in piece, "Unexpected old annotation field")
        piece["internal_curves"] = []
    c.require(c.same(derived, declared), "Expected metadata differs beyond empty internal_curves fields")
    return declared


def check_preflight(directory):
    declared = expected()
    declared.pop("masses_kg")
    actual = c.read(c.safe_file("mesh-metadata.json", directory))
    c.require(c.same(actual, declared), "Full exact physical rest/project/construction signature differs")
    binding = c.read(ROOT / "READONLY-BINDING.json")
    repo = Path(binding["repo"])
    old_obj = repo / "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/runs/base/stage-000.obj"
    c.require(c.sha(old_obj) == binding["files"][str(old_obj.relative_to(repo))], "Old rest OBJ changed")
    c.require(c.safe_file("rest.obj", directory).read_bytes() == old_obj.read_bytes(), "Rest OBJ bytes differ")
    return {"full_rest_signature_exact": True, "rest_obj_bytes_exact": True,
            "physical_fields_normalized": [], "serialization_extension": "two empty internal_curves arrays only",
            "derived_mass_scope": "Identical UV/material/topology plus identical solver/bridge; actual masses checked exactly after producer initialization",
            "vertices": len(actual["piece_ids"]), "triangles": len(actual["triangles"]),
            "stitches": len(actual["stitches"]), "doc_revision": actual["doc_revision"]}


def check_current_metadata(meta, case):
    c.require(case == "base", "Only Harbor A is authorized")
    declared = expected()
    c.require(c.same(meta, declared), "Full initialized rest signature including every mass differs")
    calculated = c.metrics.masses(meta)
    error = max(abs(a-b) for a, b in zip(calculated, meta["masses_kg"]))
    c.require(error <= 1e-12 and all(m > 0 for m in calculated), "Independent physical mass check failed")
    return {"exact_current_rest_metadata": True, "original_physical_data_exact": True,
            "serialization_extension": "two empty internal_curves arrays only",
            "doc_revision": 20, "vertices": len(calculated), "triangles": len(meta["triangles"]),
            "max_independent_mass_error_kg": error, "total_independent_mass_kg": sum(calculated)}
