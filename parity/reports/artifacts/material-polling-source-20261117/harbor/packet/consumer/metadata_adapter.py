"""Strict v3-to-v5 empty-field compatibility; no numerical acceptance changes."""
import copy
from pathlib import Path
import struct

import evaluator as c

ROOT = Path(__file__).resolve().parents[3]
INPUT = ROOT / "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/inputs/base.weftforge"
GOLD = ROOT / "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/runs/base/rest-metadata.json"
INPUT_SHA = "fc494974955d440aacbfde49c332a50218bd3a9354d06f4fbbf0596e5da8eda7"
GOLD_SHA = "d96697949c686fbf15f19b99b6d7551b35d3449b17157009aceb75649fbaeeb9"


def extend_empty_piece_fields(project):
    c.require([p["id"] for p in project["pieces"]] == [2, 31], "Wrong original pieces")
    for piece in project["pieces"]:
        for field in ["holes", "internal_curves"]:
            c.require(field not in piece, "Unexpected compatibility input already contains new field")
            piece[field] = []


def check_current_metadata(meta, case):
    c.require(case == "base", "Only prospectively declared Harbor A is allowed")
    c.require(c.sha(INPUT) == INPUT_SHA and c.sha(GOLD) == GOLD_SHA, "Original input/rest binding changed")
    original = c.read(GOLD)
    expected = copy.deepcopy(original)
    extend_empty_piece_fields(expected["project"])
    c.require(c.same(meta, expected), "Exact original physical rest/project/UV/topology/material/boundary/stitch/mass identity failed")
    authored = c.read(INPUT)["project"]
    extend_empty_piece_fields(authored)
    for material in authored["materials"]:
        material["color"] = [struct.unpack(">f", struct.pack(">f", x))[0] for x in material["color"]]
    c.require(c.same(meta["project"], authored), "Unexpected authoring conversion beyond four empty piece fields and known f32 colors")
    c.require(meta["doc_revision"] == meta["project"]["revision"] == 20, "Wrong authored revision")
    c.require(meta["all_contact_enabled"] is True and meta["pins"] == [] and c.same(meta["ground_height_m"], 0.), "Wrong contact/pin/ground scope")
    masses = c.metrics.masses(meta)
    c.require(len(masses) == 654 and len(meta["triangles"]) == 1172, "Wrong exact rest mesh")
    error = max(abs(a-b) for a, b in zip(masses, meta["masses_kg"]))
    c.require(error <= 1e-12 and all(m > 0 for m in masses), "Independent rest-area mass check failed")
    return {"exact_original_physical_rest": True, "serialization_only_extension": "holes=[] and internal_curves=[] on pieces2/31 only", "vertices": 654, "triangles": 1172, "doc_revision": 20, "max_independent_mass_error_kg": error, "total_independent_mass_kg": sum(masses)}
