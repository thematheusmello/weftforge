#!/usr/bin/env python3
"""Frozen prospective file-adapter evaluator; standard-library arithmetic only.

Self-check never executes WeftForge. Evaluate requires a separately reviewed,
immutable CLI SHA-256 and a fresh output directory, and verifies this freeze first.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
CRITERIA = json.loads((ROOT / "CRITERIA.json").read_text())
FIXTURE = json.loads((ROOT / "fixture.weftforge").read_text())
TOL = CRITERIA["metric_tolerance_m"]
ORACLE_TOL = CRITERIA["oracle_crosscheck_tolerance_m"]
MODERN = "2026-07-28"
LEGACY = "2025-11-25"
TOOLS = {"project_inspect", "project_lengths", "project_create", "project_execute", "project_import_svg",
         "project_import_dxf", "project_export_svg", "project_export_obj", "project_export_dxf"}


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def finite(value):
    return type(value) in (int, float) and math.isfinite(value)


def close(actual, expected, tolerance=TOL):
    require(finite(actual) and abs(actual - expected) <= tolerance,
            f"number {actual!r} differs from independent {expected!r} by more than {tolerance}")


def keys(value, expected):
    require(type(value) is dict and set(value) == set(expected), f"unexpected keys: {value!r}")


def loads(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, f"duplicate JSON field {key}")
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(AssertionError(f"nonfinite JSON {value}")))


def encoded(value):
    return json.dumps(value, indent=2, allow_nan=False).encode() + b"\n"


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def primitive(t):
    return 0.1 * (t * math.sqrt(1 + t * t) + math.asinh(t))


def curve_data(doc, curve_id):
    p = doc["project"]
    curve = next(x for x in p["curves"] if x["id"] == curve_id)
    points = {x["id"]: x["position"] for x in p["points"]}
    a, b = points[curve["start"]], points[curve["end"]]
    return a, b, curve["controls"]


def point(doc, curve, t):
    a, b, controls = curve_data(doc, curve)
    if controls is None:
        return [a[i] + (b[i] - a[i]) * t for i in range(2)]
    c, d = controls
    s = 1 - t
    return [s ** 3 * a[i] + 3 * s * s * t * c[i] + 3 * s * t * t * d[i] + t ** 3 * b[i]
            for i in range(2)]


def distance(a, b):
    return math.hypot(a[0] - b[0], a[1] - b[1])


def quadrature(doc, curve, t0, t1, steps=4096):
    """Independent derivative Simpson quadrature; no production table or metric."""
    a, b, controls = curve_data(doc, curve)
    if controls is None:
        return distance(a, b) * abs(t1 - t0)
    c, d = controls
    def speed(t):
        s = 1 - t
        v = [3 * (s * s * (c[i] - a[i]) + 2 * s * t * (d[i] - c[i]) + t * t * (b[i] - d[i]))
             for i in range(2)]
        return math.hypot(*v)
    h = (t1 - t0) / steps
    return abs(h / 3 * math.fsum([speed(t0), speed(t1)] + [
        (4 if i % 2 else 2) * speed(t0 + i * h) for i in range(1, steps)]))


def quad_inverse(doc, curve, t0, t1, length):
    lo, hi = 0.0, 1.0
    for _ in range(56):
        u = (lo + hi) / 2
        if quadrature(doc, curve, t0, t0 + (t1 - t0) * u, 2048) < length:
            lo = u
        else:
            hi = u
    return t0 + (t1 - t0) * ((lo + hi) / 2)


def analytic_length(doc, curve, t0, t1):
    if curve == 6:
        return abs(primitive(t1) - primitive(t0))
    if curve == 15 and curve_data(doc, curve)[2] is not None:
        f = lambda t: 0.03 * t * t + 0.17 * t * t * t
        return abs(f(t1) - f(t0))
    return distance(*curve_data(doc, curve)[:2]) * abs(t1 - t0)


def variant(spec):
    doc = copy.deepcopy(FIXTURE)
    p = doc["project"]
    if spec.get("collinear_b"):
        next(x for x in p["curves"] if x["id"] == 15)["controls"] = [[0.4, 0.0], [0.41, 0.0]]
    if spec.get("legacy_seam"):
        p["seams"] = [{"id": 20, "name": "Original nonlinear legacy t map",
                       "a": {"curve": 6, "t0": 0.1, "t1": 0.9},
                       "b": {"curve": 15, "t0": 0.9, "t1": 0.1},
                       "correspondence": [[0.0, 0.0], [0.25, 0.6], [0.5, 0.8], [1.0, 1.0]]}]
        p["next_id"] = 21
    for key in ["revision", "next_id", "resolution"]:
        if key in spec:
            p[key] = spec[key]
    return doc


def expected_document(source, case):
    result = copy.deepcopy(source)
    p = result["project"]
    p["revision"] += 1
    group = {"id": p["next_id"], "name": case["command"]["data"]["name"],
             "a": [{k: case["expected"]["a"][k] for k in ["curve", "t0", "t1"]}],
             "b": [{k: case["expected"]["b"][k] for k in ["curve", "t0", "t1"]}],
             "correspondence": [[0.0, 0.0], [1.0, 1.0]]}
    p["next_id"] += 1
    p["sewing_groups"].append(group)
    return result


def check_document(actual, source, case):
    expected = expected_document(source, case)
    keys(actual, ["format", "version", "project"])
    keys(actual["project"], expected["project"])
    require(actual["format"] == "weftforge" and actual["version"] == 3, "native envelope changed")
    require(len(actual["project"]["sewing_groups"]) == 1, "creation must store exactly one ordinary group")
    group = actual["project"]["sewing_groups"][0]
    keys(group, ["id", "name", "a", "b", "correspondence"])
    for side in ["a", "b"]:
        require(type(group[side]) is list and len(group[side]) == 1, "creation must store one interval per side")
        interval = group[side][0]
        keys(interval, ["curve", "t0", "t1"])
        want = case["expected"][side]
        require(interval["curve"] == want["curve"] and interval["t0"] == want["t0"], "start attachment changed")
        require(finite(interval["t1"]) and 0 <= interval["t1"] <= 1, "invalid endpoint")
        close(distance(point(source, interval["curve"], interval["t1"]), want["samples"][-1]["point_m"]), 0.0)
        close(analytic_length(source, interval["curve"], interval["t0"], interval["t1"]), want["length_m"])
        close(quadrature(source, interval["curve"], interval["t0"], interval["t1"]), want["length_m"])
        if want["exact_end"] is not None:
            require(interval["t1"] == want["exact_end"], "full remaining request lost exact endpoint t")
        elif curve_data(source, interval["curve"])[2] is None:
            close(interval["t1"], want["t1"], CRITERIA["straight_t_tolerance"])
        # Endpoints use the metric budget; other construction fields compare as parsed JSON values.
        expected["project"]["sewing_groups"][0][side][0]["t1"] = interval["t1"]
    require(actual == expected, "unrelated parsed construction, group semantics, revision or allocator changed")
    return group


def check_lengths(report, doc, case, query):
    keys(report, ["source_revision", "length_unit", "query", "lengths_m", "mismatch_b_minus_a_m"])
    require(report["source_revision"] == doc["project"]["revision"] and report["length_unit"] == "m", "stale/unit length report")
    require(report["query"] == query, "wrong length query echo")
    require(type(report["lengths_m"]) is list and len(report["lengths_m"]) == 2, "length report shape")
    lengths = [case["expected"][s]["length_m"] for s in ["a", "b"]]
    for got, want in zip(report["lengths_m"], lengths):
        close(got, want)
    close(report["mismatch_b_minus_a_m"], lengths[1] - lengths[0])


def weighted_t(boundary, endpoint, curve):
    keys(endpoint, ["vertices", "weight"])
    require(type(endpoint["vertices"]) is list and len(endpoint["vertices"]) == 2, "weighted vertex shape")
    require(finite(endpoint["weight"]) and 0 <= endpoint["weight"] <= 1, "weighted vertex range")
    ts = []
    for vertex in endpoint["vertices"]:
        require(type(vertex) is int, "vertex is not an index")
        matches = [b["t"] for b in boundary if b["curve"] == curve and b["vertex"] == vertex]
        require(len(matches) == 1, "stitch lost its owning curve")
        ts.append(matches[0])
    return ts[0] * (1 - endpoint["weight"]) + ts[1] * endpoint["weight"]


def mapped(knots, value):
    for a, b in zip(knots, knots[1:]):
        if value <= b[0]:
            return a[1] + (value - a[0]) / (b[0] - a[0]) * (b[1] - a[1])
    return 1.0


def check_inspection(report, doc, case, quarter_probe=False):
    keys(report, ["kind", "source", "sewing_groups", "mesh"])
    require(report["kind"] == "sewing_inspection" and report["source"] == doc, "inspection source changed")
    mesh = report["mesh"]
    keys(mesh, ["doc_revision", "positions", "uv", "triangles", "piece_ids", "boundary", "stitches"])
    require(mesh["doc_revision"] == doc["project"]["revision"], "stale mesh")
    require(len(mesh["positions"]) == len(mesh["uv"]) == len(mesh["piece_ids"]) > 0, "mesh vertex arrays")
    for i, uv in enumerate(mesh["uv"]):
        require(len(uv) == 2 and all(finite(x) for x in uv), "nonfinite material position")
        require(len(mesh["positions"][i]) == 3 and all(finite(x) for x in mesh["positions"][i]), "nonfinite rest position")
        require(mesh["piece_ids"][i] in [10, 19], "unknown rest piece")
    require(len(mesh["triangles"]) > 0, "missing rest triangles")
    for triangle in mesh["triangles"]:
        require(len(triangle) == 3 and all(type(v) is int and 0 <= v < len(mesh["uv"]) for v in triangle), "invalid triangle")
    for b in mesh["boundary"]:
        keys(b, ["piece", "curve", "t", "vertex"])
        require(type(b["vertex"]) is int and 0 <= b["vertex"] < len(mesh["uv"]), "bad boundary index")
        require(finite(b["t"]) and 0 <= b["t"] <= 1, "bad boundary parameter")
        close(distance(mesh["uv"][b["vertex"]], point(doc, b["curve"], b["t"])), 0.0)
    group = doc["project"]["sewing_groups"][0]
    require(type(report["sewing_groups"]) is list and len(report["sewing_groups"]) == 1, "derived group count")
    geometry = report["sewing_groups"][0]
    keys(geometry, ["id", "lengths", "portions"])
    require(geometry["id"] == group["id"] and len(geometry["lengths"]) == 2, "derived group identity")
    for got, side in zip(geometry["lengths"], ["a", "b"]):
        close(got, case["expected"][side]["length_m"])
    require(len(geometry["portions"]) == (4 if quarter_probe else 1), "source-derived portion count")
    for index, portion in enumerate(geometry["portions"]):
        keys(portion, ["a", "b"])
        for side in ["a", "b"]:
            keys(portion[side], ["curve", "t0", "t1"])
            expected = case["expected"][side]
            require(portion[side]["curve"] == expected["curve"], "portion owner changed")
            for which, sample in [("t0", index if quarter_probe else 0),
                                  ("t1", index + 1 if quarter_probe else 4)]:
                t = portion[side][which]
                require(finite(t) and 0 <= t <= 1, "invalid portion t")
                close(distance(point(doc, expected["curve"], t), expected["samples"][sample]["point_m"]), 0.0)
    stitched = [s for s in mesh["stitches"] if s["seam"] == group["id"]]
    require(len(stitched) >= 2, "group lacks real rest stitches")
    for stitch in stitched:
        keys(stitch, ["seam", "a", "b"])
        fractions = []
        for side in ["a", "b"]:
            iv = group[side][0]
            t = weighted_t(mesh["boundary"], stitch[side], iv["curve"])
            require(min(iv["t0"], iv["t1"]) - 1e-12 <= t <= max(iv["t0"], iv["t1"]) + 1e-12, "stitch outside source interval")
            fractions.append(analytic_length(doc, iv["curve"], iv["t0"], t) / case["expected"][side]["length_m"])
        close((fractions[1] - fractions[0]) * max(x["length_m"] for x in case["expected"].values()), 0.0)
    allowed = {group["id"]} | {s["id"] for s in doc["project"]["seams"]}
    require(all(s["seam"] in allowed for s in mesh["stitches"]), "unexpected stitch relationship")
    for seam in doc["project"]["seams"]:
        legacy_stitches = [s for s in mesh["stitches"] if s["seam"] == seam["id"]]
        require(len(legacy_stitches) >= 2, "missing legacy stitches")
        for stitch in legacy_stitches:
            fractions = []
            for side in ["a", "b"]:
                iv = seam[side]
                t = weighted_t(mesh["boundary"], stitch[side], iv["curve"])
                fractions.append((t - iv["t0"]) / (iv["t1"] - iv["t0"]))
            close(fractions[1], mapped(seam["correspondence"], fractions[0]), 1e-10)


def check_schema(result):
    require(set(t["name"] for t in result["tools"]) == TOOLS and len(result["tools"]) == 9, "MCP tool surface changed")
    tool = next(t for t in result["tools"] if t["name"] == "project_execute")
    schema = tool["inputSchema"]
    require(schema["additionalProperties"] is False and set(schema["required"]) == {"input", "output", "commands"}, "execute schema is open")
    commands = schema["properties"]["commands"]
    require(commands["minItems"] == 1 and commands["maxItems"] == 1024, "command count schema")
    choices = commands["items"]["oneOf"]
    require(len(choices) == 26 and len({c["properties"]["type"]["const"] for c in choices}) == 26, "expected 26 distinct shared command variants")
    matches = [c for c in choices if c["properties"]["type"]["const"] == "add_sewing_by_length"]
    require(len(matches) == 1, "missing/new command schema")
    c = matches[0]
    require(c["additionalProperties"] is False and set(c["required"]) == {"type", "data"}, "command envelope is open")
    d = c["properties"]["data"]
    require(d["additionalProperties"] is False and set(d["required"]) == {"name", "a", "b"}, "command data is open")
    require(set(d["properties"]) == {"name", "a", "b"} and d["properties"]["name"]["type"] == "string", "command fields schema")
    for s in ["a", "b"]:
        p = d["properties"][s]
        fields = {"curve", "start_t", "direction", "length_m"}
        require(p["type"] == "object" and p["additionalProperties"] is False and set(p["required"]) == fields and set(p["properties"]) == fields, "sewing side schema is open")
        p = p["properties"]
        require(p["curve"]["type"] == "integer" and p["curve"].get("minimum", 0) >= 0 and p["curve"].get("maximum") == 18446744073709551615, "curve ID schema")
        require(p["start_t"]["type"] == "number" and p["start_t"]["minimum"] == 0 and p["start_t"]["maximum"] == 1, "start range schema")
        require(p["length_m"]["type"] == "number" and p["length_m"]["exclusiveMinimum"] == 0, "positive SI length schema")
        require(p["direction"]["type"] == "string" and set(p["direction"]["enum"]) == {"forward", "reverse"}, "closed direction schema")


def meta():
    return {"io.modelcontextprotocol/protocolVersion": MODERN,
            "io.modelcontextprotocol/clientCapabilities": {}}


def request(method, params, ident, modern):
    params = copy.deepcopy(params)
    if modern:
        params["_meta"] = meta()
    return {"jsonrpc": "2.0", "id": ident, "method": method, "params": params}


def unwrap(response, ident, modern, tool=False, success=True):
    require(type(response) is dict and response.get("jsonrpc") == "2.0" and response.get("id") == ident, "JSON-RPC binding failed")
    if "error" in response:
        keys(response, ["jsonrpc", "id", "error"])
        require(not success and response["error"].get("code") in [-32700, -32600, -32602], "unexpected protocol error")
        require(type(response["error"].get("message")) is str and response["error"]["message"], "error lacks diagnosis")
        return response["error"]
    keys(response, ["jsonrpc", "id", "result"])
    result = response["result"]
    require(type(result) is dict, "MCP result shape")
    if modern:
        require(result.get("resultType") == "complete", "modern response lacks complete resultType")
    else:
        require("resultType" not in result, "legacy response gained modern resultType")
    if tool:
        keys(result, ["content", "structuredContent", "isError"] + (["resultType"] if modern else []))
        require(result["isError"] is (not success), "MCP tool success/error flag")
        require(type(result["content"]) is list and len(result["content"]) == 1, "MCP content shape")
        keys(result["content"][0], ["type", "text"])
        require(result["content"][0]["type"] == "text" and loads(result["content"][0]["text"]) == result["structuredContent"], "MCP text differs from structured output")
        if not success:
            keys(result["structuredContent"], ["error"])
            require(type(result["structuredContent"]["error"]) is str and result["structuredContent"]["error"], "tool error lacks diagnosis")
        return result["structuredContent"]
    require(success, "failure accepted as a successful non-tool result")
    return result


def run_process(cli, args, cwd, stdin=None):
    completed = subprocess.run([str(cli)] + args, cwd=cwd, input=stdin,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=120)
    return completed


def mcp(cli, cwd, method, params, modern=True, tool=False, success=True, raw_commands=None):
    messages = []
    if not modern:
        messages = [{"jsonrpc": "2.0", "id": "init", "method": "initialize", "params": {
            "protocolVersion": LEGACY, "capabilities": {}, "clientInfo": {"name": "independent-acceptance", "version": "1"}}},
            {"jsonrpc": "2.0", "method": "notifications/initialized"}]
    call = request(method, params, "acceptance", modern)
    raw = json.dumps(call, allow_nan=False)
    if raw_commands is not None:
        require('"commands": []' in raw, "raw command injection binding")
        raw = raw.replace('"commands": []', '"commands": ' + raw_commands)
    stdin = "\n".join([json.dumps(m) for m in messages] + [raw]) + "\n"
    cp = run_process(cli, ["mcp"], cwd, stdin.encode())
    require(cp.returncode == 0 and not cp.stderr, "MCP server exited or wrote stderr")
    lines = cp.stdout.splitlines()
    require(len(lines) == (1 if modern else 2), "unexpected MCP response count")
    responses = [loads(line) for line in lines]
    if not modern:
        initial = unwrap(responses[0], "init", False)
        require(initial.get("protocolVersion") == LEGACY, "legacy negotiation changed")
    last = responses[-1]
    # Syntax failures may have no id because the malformed request could not be decoded.
    if not success and "error" in last and "id" not in last:
        require(set(last) == {"jsonrpc", "error"} and last["jsonrpc"] == "2.0" and last["error"].get("code") in [-32700, -32600], "unbound syntax error shape")
        result = last["error"]
    else:
        result = unwrap(last, "acceptance", modern, tool, success)
    return result, dict(returncode=cp.returncode, stdout=cp.stdout.decode(), stderr=cp.stderr.decode())


def execute(cli, cwd, channel, commands, success=True, input_name="input.weftforge", output_name="output.weftforge", raw_commands=None):
    if channel == "cli_execute":
        raw = raw_commands if raw_commands is not None else json.dumps(commands, allow_nan=False)
        (cwd / "commands.json").write_text(raw)
        cp = run_process(cli, ["execute", input_name, output_name, "commands.json"], cwd)
        if success:
            require(cp.returncode == 0 and not cp.stderr, "CLI execute failed")
            report = loads(cp.stdout)
            keys(report, ["revision", "commands_applied"])
        else:
            require(cp.returncode != 0 and not cp.stdout and cp.stderr, "CLI failure was accepted/undignosed")
            report = {"error": cp.stderr.decode()}
        return report, dict(returncode=cp.returncode, stdout=cp.stdout.decode(), stderr=cp.stderr.decode())
    report, transcript = mcp(cli, cwd, "tools/call", {"name": "project_execute", "arguments": {
        "input": input_name, "output": output_name, "commands": commands}}, modern=channel.endswith(MODERN),
        tool=True, success=success, raw_commands=raw_commands)
    if success:
        keys(report, ["input", "output", "revision", "commands_applied"])
        require(report["input"] == input_name and report["output"] == output_name, "MCP paths changed")
    return report, transcript


def snapshot(path):
    records = {}
    for entry in sorted(path.rglob("*")):
        name = entry.relative_to(path).as_posix()
        if name == "commands.json":
            continue
        if entry.is_symlink():
            records[name] = {"kind": "symlink", "target": str(entry.readlink())}
        elif entry.is_file():
            raw = entry.read_bytes()
            records[name] = {"kind": "file", "bytes": len(raw), "sha256": sha(raw)}
        elif entry.is_dir():
            records[name] = {"kind": "directory"}
        else:
            raise AssertionError("unexpected filesystem object")
    return records


def preserved(before, after):
    require(before == after, "failed operation changed input, destination, recovery or left a temporary file")


def crosscheck_oracles():
    maximum = 0.0
    for case in CRITERIA["positive_cases"]:
        doc = variant(case["variant"])
        for side in ["a", "b"]:
            want = case["expected"][side]
            t0, t1 = want["t0"], want["t1"]
            q = quadrature(doc, want["curve"], t0, t1)
            q_coarse = quadrature(doc, want["curve"], t0, t1, 2048)
            error = max(abs(q - want["length_m"]), abs(q - q_coarse))
            require(error <= ORACLE_TOL, "independent integral methods disagree")
            maximum = max(maximum, error)
            for sample in want["samples"]:
                t = quad_inverse(doc, want["curve"], t0, t1, want["length_m"] * sample["fraction"])
                error = distance(point(doc, want["curve"], t), sample["point_m"])
                require(error <= ORACLE_TOL, "independent inverse methods disagree")
                maximum = max(maximum, error)
    a = CRITERIA["positive_cases"][0]["expected"]["a"]
    require(distance(a["samples"][2]["point_m"], [0.2 * a["t1"] / 2, 0.1 * (a["t1"] / 2) ** 2]) > 1e-4,
            "asymmetric fixture cannot distinguish linear t matching")
    collinear = next(c for c in CRITERIA["positive_cases"] if c["id"] == "nonuniform_collinear")
    b = collinear["expected"]["b"]
    require(abs(0.03 * (b["t1"] / 2) ** 2 + 0.17 * (b["t1"] / 2) ** 3 - 0.05) > 1e-3,
            "nonuniform collinear fixture cannot distinguish linear t matching")
    return maximum


def synthetic_schema():
    side = {"type": "object", "additionalProperties": False,
            "required": ["curve", "start_t", "direction", "length_m"], "properties": {
                "curve": {"type": "integer", "minimum": 0, "maximum": 18446744073709551615},
                "start_t": {"type": "number", "minimum": 0, "maximum": 1},
                "direction": {"type": "string", "enum": ["forward", "reverse"]},
                "length_m": {"type": "number", "exclusiveMinimum": 0}}}
    choice = {"type": "object", "additionalProperties": False, "required": ["type", "data"],
              "properties": {"type": {"const": "add_sewing_by_length"}, "data": {
                  "type": "object", "additionalProperties": False, "required": ["name", "a", "b"],
                  "properties": {"name": {"type": "string"}, "a": side, "b": copy.deepcopy(side)}}}}
    choices = [choice] + [{"properties": {"type": {"const": f"synthetic_existing_{i}"}}} for i in range(25)]
    return {"tools": [{"name": name, "inputSchema": {"additionalProperties": False,
        "required": ["input", "output", "commands"], "properties": {"commands": {
            "minItems": 1, "maxItems": 1024, "items": {"oneOf": choices}}}}} for name in sorted(TOOLS)]}


def synthetic_inspection(doc, case, quarter_probe=True):
    """Minimal invented rest samples, used only to exercise the checker."""
    group = doc["project"]["sewing_groups"][0]
    boundary, uv = [], []
    for side in ["a", "b"]:
        for s in case["expected"][side]["samples"]:
            vertex = len(uv)
            uv.append(s["point_m"])
            boundary.append(dict(piece=10 if side == "a" else 19,
                                 curve=case["expected"][side]["curve"], t=s["t"], vertex=vertex))
    stitches = [dict(seam=group["id"], a={"vertices": [i, i], "weight": 0.0},
                     b={"vertices": [i + 5, i + 5], "weight": 0.0}) for i in range(5)]
    portions = [{s: {"curve": case["expected"][s]["curve"],
                     "t0": case["expected"][s]["samples"][i]["t"],
                     "t1": case["expected"][s]["samples"][i + 1]["t"]}
                 for s in ["a", "b"]} for i in range(4)]
    return {"kind": "sewing_inspection", "source": copy.deepcopy(doc),
            "sewing_groups": [{"id": group["id"], "lengths": [0.1, 0.1], "portions": portions}],
            "mesh": {"doc_revision": doc["project"]["revision"], "uv": uv,
                     "positions": [[x, y, 0.0] for x, y in uv], "piece_ids": [10] * 5 + [19] * 5,
                     "triangles": [[0, 1, 2], [5, 6, 7]], "boundary": boundary, "stitches": stitches}}


def self_check():
    results = []
    maximum = crosscheck_oracles()
    case = CRITERIA["positive_cases"][0]
    good = expected_document(FIXTURE, case)
    check_document(good, FIXTURE, case)
    query = {"type": "sewing_group", "data": {"id": 20}}
    length = dict(source_revision=42, length_unit="m", query=query, lengths_m=[0.1, 0.1], mismatch_b_minus_a_m=0.0)
    check_lengths(length, good, case, query)
    inspection = synthetic_inspection(good, case)
    check_inspection(inspection, good, case, True)
    schema = synthetic_schema()
    check_schema(schema)
    response = {"jsonrpc": "2.0", "id": "acceptance", "result": {
        "content": [{"type": "text", "text": json.dumps(length)}], "structuredContent": length,
        "isError": False, "resultType": "complete"}}
    unwrap(response, "acceptance", True, True)

    def reject(name, fn):
        try:
            fn()
        except (AssertionError, KeyError, TypeError, ValueError, StopIteration) as error:
            results.append({"id": name, "deliberately_wrong": True, "rejected": True,
                            "diagnosis": str(error)[:300]})
        else:
            results.append({"id": name, "deliberately_wrong": True, "rejected": False})

    for name, mutate in [
        ("curve_linear_t_endpoint", lambda d: d["project"]["sewing_groups"][0]["a"][0].update(t1=0.5)),
        ("straight_wrong_direction", lambda d: d["project"]["sewing_groups"][0]["b"][0].update(t1=1.0)),
        ("legacy_seam_substitution", lambda d: d["project"].update(sewing_groups=[])),
        ("wrong_revision", lambda d: d["project"].update(revision=43)),
        ("allocator_leak", lambda d: d["project"].update(next_id=22)),
        ("persistent_length_lock", lambda d: d["project"]["sewing_groups"][0].update(length_m=0.1)),
        ("unrelated_construction_changed", lambda d: d["project"]["points"][0].update(position=[0.0, 0.01])),
    ]:
        bad = copy.deepcopy(good)
        mutate(bad)
        reject(name, lambda bad=bad: check_document(bad, FIXTURE, case))
    for name, mutate in [
        ("stale_length_report", lambda d: d.update(source_revision=41)),
        ("millimetres_reported_as_metres", lambda d: d.update(lengths_m=[100.0, 100.0])),
        ("nonfinite_length", lambda d: d.update(lengths_m=[float("nan"), 0.1])),
        ("wrong_mismatch", lambda d: d.update(mismatch_b_minus_a_m=0.01)),
    ]:
        bad = copy.deepcopy(length)
        mutate(bad)
        reject(name, lambda bad=bad: check_lengths(bad, good, case, query))
    for name, mutate in [
        ("linear_t_midpoint_portion", lambda d: d["sewing_groups"][0]["portions"][1]["a"].update(t1=case["expected"]["a"]["t1"] / 2)),
        ("wrong_stitch_correspondence", lambda d: d["mesh"]["stitches"][2]["b"].update(vertices=[8, 8])),
        ("stale_rest_mesh", lambda d: d["mesh"].update(doc_revision=41)),
        ("detached_boundary_geometry", lambda d: d["mesh"]["uv"][2].__setitem__(0, 0.11)),
    ]:
        bad = copy.deepcopy(inspection)
        mutate(bad)
        reject(name, lambda bad=bad: check_inspection(bad, good, case, True))
    for name, mutate in [
        ("MCP_false_success", lambda d: d["result"].update(isError=True)),
        ("MCP_text_disagrees", lambda d: d["result"]["content"][0].update(text="{}")),
        ("MCP_wrong_id", lambda d: d.update(id="other")),
        ("MCP_missing_modern_complete", lambda d: d["result"].pop("resultType")),
    ]:
        bad = copy.deepcopy(response)
        mutate(bad)
        reject(name, lambda bad=bad: unwrap(bad, "acceptance", True, True))
    bad = copy.deepcopy(schema)
    next(t for t in bad["tools"] if t["name"] == "project_execute")["inputSchema"]["properties"]["commands"]["items"]["oneOf"][0]["properties"]["data"]["properties"]["a"]["additionalProperties"] = True
    reject("open_side_schema", lambda: check_schema(bad))
    reject("recovery_bytes_changed", lambda: preserved({"output.recovery": {"sha256": "old"}}, {"output.recovery": {"sha256": "changed"}}))
    reject("duplicate_output_JSON", lambda: loads('{"revision":42,"revision":43}'))
    reject("nonfinite_output_JSON", lambda: loads('{"length_m":NaN}'))
    require(all(r["rejected"] for r in results), "checker accepted a deliberate wrong output")
    return {"kind": "synthetic_evaluator_self_check", "status": "passed", "product_executed": False,
            "oracle_maximum_crosscheck_error_m": maximum, "wrong_outputs_rejected": len(results),
            "mutants": results, "limits": ["Invented structural outputs test assertions; no executable adapter invocation or actual mesh/GUI/solver acceptance."]}


def verify_freeze():
    manifest = loads((ROOT / "FREEZE.json").read_bytes())
    for entry in manifest["files"]:
        require(sha((ROOT / entry["path"]).read_bytes()) == entry["sha256"], f"frozen acceptance changed: {entry['path']}")
    return sha((ROOT / "FREEZE.json").read_bytes())


def evaluate(cli, binary_sha, out):
    freeze_sha = verify_freeze()
    require(sha(cli.read_bytes()) == binary_sha, "CLI does not match separately reviewed immutable binary hash")
    require(not out.exists(), "output directory must be new; prior attempts are never overwritten")
    out.mkdir(parents=True)
    observations = []

    def record(ident, fn):
        try:
            detail = fn()
            observations.append({"id": ident, "status": "passed", "detail": detail})
        except Exception as error:
            observations.append({"id": ident, "status": "failed", "error": f"{type(error).__name__}: {error}"})
        (out / "RESULT.json").write_bytes(encoded({"kind": "bounded_independent_engineering_adapter_acceptance",
            "freeze_sha256": freeze_sha, "cli_sha256": binary_sha, "contract_sha256": CRITERIA["contract_sha256"],
            "immutable_source_inputs": CRITERIA["immutable_source_commit"], "observations": observations,
            "separate_required_gates": CRITERIA["separate_required_gates"], "claim_limits": CRITERIA["claim_limits"]}))

    def schemas():
        results = []
        for modern in [True, False]:
            result, transcript = mcp(cli, out, "tools/list", {}, modern)
            check_schema(result)
            name = "modern" if modern else "legacy"
            (out / f"schema-{name}.json").write_bytes(encoded(result))
            (out / f"schema-{name}-transcript.json").write_bytes(encoded(transcript))
            results.append(name)
        return results
    record("closed_MCP_schemas", schemas)

    for case in CRITERIA["positive_cases"]:
        def positive(case=case):
            blobs = []
            source = variant(case["variant"])
            for channel in CRITERIA["channels"]:
                work = out / case["id"] / channel
                work.mkdir(parents=True)
                (work / "input.weftforge").write_bytes(encoded(source))
                report, transcript = execute(cli, work, channel, [case["command"]])
                (work / "execute-transcript.json").write_bytes(encoded(transcript))
                require(report["revision"] == source["project"]["revision"] + 1 and report["commands_applied"] == 1, "creation was not one revision/command")
                raw = (work / "output.weftforge").read_bytes()
                actual = loads(raw)
                group = check_document(actual, source, case)
                blobs.append(raw)
                query = {"type": "sewing_group", "data": {"id": group["id"]}}
                if channel == "cli_execute":
                    cp = run_process(cli, ["lengths", "output.weftforge", "sewing-group", str(group["id"])], work)
                    require(cp.returncode == 0 and not cp.stderr, "CLI length read failed")
                    length = loads(cp.stdout)
                else:
                    length, _ = mcp(cli, work, "tools/call", {"name": "project_lengths", "arguments": {"path": "output.weftforge", "query": query}}, channel.endswith(MODERN), True)
                    inspected, _ = mcp(cli, work, "tools/call", {"name": "project_inspect", "arguments": {"path": "output.weftforge"}}, channel.endswith(MODERN), True)
                    require(inspected == {"project": actual["project"], "units": {"length": "m", "mass": "kg", "time": "s"}, "up": "Y"}, "MCP authored inspection changed")
                check_lengths(length, actual, case, query)
                (work / "lengths.json").write_bytes(encoded(length))
            require(blobs[0] == blobs[1] == blobs[2], "CLI and modern/legacy MCP native output bytes differ")
            return {"channels": 3, "native_sha256": sha(blobs[0])}
        record(case["id"], positive)

    def mesh_probes():
        ids = ["initial_asymmetric", "disjoint_same_curve", "unequal_existing_proportional", "nonuniform_collinear", "legacy_nonlinear_preserved"]
        counts = {}
        for ident in ids:
            case = next(c for c in CRITERIA["positive_cases"] if c["id"] == ident)
            for resolution in CRITERIA["coarse_fine_resolutions_m"]:
                source = variant({**case["variant"], "resolution": resolution})
                work = out / "mesh-probes" / ident / str(resolution)
                work.mkdir(parents=True)
                (work / "input.weftforge").write_bytes(encoded(source))
                execute(cli, work, "cli_execute", [case["command"]])
                actual = loads((work / "output.weftforge").read_bytes())
                group = check_document(actual, source, case)
                cp = run_process(cli, ["inspect-sewing", "output.weftforge"], work)
                require(cp.returncode == 0 and not cp.stderr, "rest inspection failed")
                original_inspection = loads(cp.stdout)
                check_inspection(original_inspection, actual, case)
                (work / "initial-inspection.json").write_bytes(cp.stdout)
                # Existing explicit map knots expose exact quarter fractions through
                # public portion readout; original creation endpoints were checked above.
                refined = copy.deepcopy(group)
                refined["correspondence"] = [[f, f] for f in CRITERIA["fractions"]]
                execute(cli, work, "cli_execute", [{"type": "update_sewing_group", "data": {"group": refined}}], input_name="output.weftforge", output_name="refined.weftforge")
                refined_doc = loads((work / "refined.weftforge").read_bytes())
                expected = copy.deepcopy(actual)
                expected["project"]["revision"] += 1
                expected["project"]["sewing_groups"][0] = refined
                require(refined_doc == expected, "map probe changed source intervals or allocated an ID")
                cp = run_process(cli, ["inspect-sewing", "refined.weftforge"], work)
                require(cp.returncode == 0 and not cp.stderr, "refined rest inspection failed")
                inspection = loads(cp.stdout)
                check_inspection(inspection, refined_doc, case, True)
                (work / "quarter-inspection.json").write_bytes(cp.stdout)
                counts[f"{ident}/{resolution}"] = len(inspection["mesh"]["uv"])
        return counts
    record("coarse_fine_authored_mapping_and_legacy_t_semantics", mesh_probes)

    negatives = CRITERIA["negative_cases"] + [{**c, "commands": [], "variant": {}, "same_file": False, "category": "raw_schema"} for c in CRITERIA["raw_schema_cases"]]
    for case in negatives:
        def negative(case=case):
            transcripts = []
            for channel in CRITERIA["channels"]:
                for same_file in ([False, True] if case["same_file"] else [False]):
                    work = out / "negative" / case["id"] / channel / ("same" if same_file else "separate")
                    work.mkdir(parents=True)
                    source = variant(case["variant"])
                    (work / "input.weftforge").write_bytes(encoded(source))
                    output_name = "input.weftforge" if same_file else "output.weftforge"
                    if not same_file:
                        (work / output_name).write_bytes(encoded(variant({"revision": 39})))
                    (work / (output_name + ".recovery")).write_bytes(encoded(variant({"revision": 38})))
                    before = snapshot(work)
                    result, transcript = execute(cli, work, channel, case["commands"], False, output_name=output_name, raw_commands=case.get("raw"))
                    preserved(before, snapshot(work))
                    (work / "transcript.json").write_bytes(encoded(transcript))
                    if case["category"] == "metric_precision_composition":
                        require("precision" in result.get("error", result.get("message", "")).lower(), "composition did not reach the intended metric precision failure")
                    transcripts.append({"channel": channel, "same_file": same_file})
            return {"runs": transcripts, "category": case["category"]}
        record(case["id"], negative)

    def same_file_success():
        case = CRITERIA["positive_cases"][0]
        hashes = []
        for channel in CRITERIA["channels"]:
            work = out / "same-file-success" / channel
            work.mkdir(parents=True)
            original = encoded(FIXTURE)
            (work / "input.weftforge").write_bytes(original)
            (work / "input.weftforge.recovery").write_bytes(encoded(variant({"revision": 38})))
            execute(cli, work, channel, [case["command"]], output_name="input.weftforge")
            raw = (work / "input.weftforge").read_bytes()
            check_document(loads(raw), FIXTURE, case)
            require((work / "input.weftforge.recovery").read_bytes() == original, "successful same-file edit lost the exact prior recovery")
            hashes.append(sha(raw))
        require(len(set(hashes)) == 1, "same-file adapter output bytes differ")
        return hashes[0]
    record("same_file_success_and_recovery_rotation", same_file_success)

    for failure in CRITERIA["writer_failure_cases"]:
        def writer_failure(failure=failure):
            for channel in CRITERIA["channels"]:
                work = out / "writer" / failure / channel
                work.mkdir(parents=True)
                (work / "input.weftforge").write_bytes(encoded(FIXTURE))
                destination = "output.weftforge"
                recovery = work / "output.weftforge.recovery"
                recovery.write_bytes(encoded(variant({"revision": 38})))
                if failure == "invalid_existing_destination":
                    (work / destination).write_bytes(b"invalid prior native bytes must survive\n")
                elif failure == "destination_directory":
                    (work / destination).mkdir()
                    (work / destination / "keep").write_bytes(b"protected directory marker")
                elif failure == "destination_symlink":
                    (work / "protected.weftforge").write_bytes(encoded(variant({"revision": 39})))
                    (work / destination).symlink_to("protected.weftforge")
                elif failure == "recovery_directory":
                    (work / destination).write_bytes(encoded(variant({"revision": 39})))
                    recovery.unlink()
                    recovery.mkdir()
                    (recovery / "keep").write_bytes(b"protected recovery marker")
                elif failure == "absent_parent":
                    destination = "missing/output.weftforge"
                elif failure == "invalid_input":
                    (work / "input.weftforge").write_bytes(b"invalid source must survive\n")
                    (work / destination).write_bytes(encoded(variant({"revision": 39})))
                before = snapshot(work)
                _, transcript = execute(cli, work, channel, [CRITERIA["positive_cases"][0]["command"]], False, output_name=destination)
                preserved(before, snapshot(work))
                (work / "transcript.json").write_bytes(encoded(transcript))
            return {"channels": 3, "preserved": True}
        record(failure, writer_failure)
    record("immutable_cli_retained", lambda: require(sha(cli.read_bytes()) == binary_sha, "CLI changed during evaluation"))
    failed = [r["id"] for r in observations if r["status"] == "failed"]
    return {"bounded_adapter_status": "failed" if failed else "passed", "failed": failed,
            "full_contract_acceptance": "not_established_separate_required_gates", "observations": len(observations)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    self_parser = sub.add_parser("self-check")
    self_parser.add_argument("--out", type=Path, required=True)
    run_parser = sub.add_parser("evaluate")
    run_parser.add_argument("--cli", type=Path, required=True)
    run_parser.add_argument("--sha256", required=True)
    run_parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "self-check":
        require(not args.out.exists(), "self-check report already exists; preserve every attempt")
        try:
            result = self_check()
            code = 0
        except Exception as error:
            result = {"kind": "synthetic_evaluator_self_check", "status": "failed", "product_executed": False,
                      "error": f"{type(error).__name__}: {error}"}
            code = 1
        result["bindings"] = {name: sha((ROOT / name).read_bytes()) for name in
                              ["evaluate_physical_sewing.py", "fixture.weftforge", "CRITERIA.json", "SOURCE-INPUTS.json"]}
        args.out.write_bytes(encoded(result))
    else:
        result = evaluate(args.cli.resolve(), args.sha256, args.out.resolve())
        code = int(bool(result["failed"]))
        (args.out / "SUMMARY.json").write_bytes(encoded(result))
    print(json.dumps(result if args.mode == "evaluate" else {k: v for k, v in result.items() if k != "mutants"}, allow_nan=False))
    return code


if __name__ == "__main__":
    sys.exit(main())
