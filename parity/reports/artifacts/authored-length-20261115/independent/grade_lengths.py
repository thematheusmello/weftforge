#!/usr/bin/env python3
"""Frozen, independent analytic acceptance; run only a root-supplied candidate."""

import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
META = {
    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    "io.modelcontextprotocol/clientCapabilities": {},
}


def require(ok, message):
    if not ok:
        raise AssertionError(message)


def no_duplicates(pairs):
    value = {}
    for key, item in pairs:
        require(key not in value, f"duplicate output key: {key}")
        value[key] = item
    return value


def parse(text):
    def bad_constant(value):
        raise AssertionError(f"nonfinite JSON: {value}")
    return json.loads(text, object_pairs_hook=no_duplicates, parse_constant=bad_constant)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def frozen():
    entries = {}
    for line in (ROOT / "SHA256SUMS").read_text().splitlines():
        digest, name = line.split("  ", 1)
        require(sha((ROOT / name).read_bytes()) == digest, f"changed frozen artifact: {name}")
        entries[name] = digest
    require(set(entries) == {"PLAN.md", "fixture.weftforge", "criteria.json", "grade_lengths.py"}, "incomplete freeze")
    return entries


def number(value):
    require(type(value) in (int, float) and math.isfinite(value), "length is not finite numeric JSON")
    return value


def grade(payload, case, criteria):
    expected = case["expected"]
    require(type(payload) is dict and payload.keys() == expected.keys(), "unexpected payload fields")
    for key in ("source_revision", "length_unit", "query"):
        require(payload[key] == expected[key], f"wrong {key}")
    require(type(payload["source_revision"]) is int, "revision is not an integer")
    if "length_m" in expected:
        actual = number(payload["length_m"])
        require(actual > 0 and abs(actual - expected["length_m"]) <= criteria["length_tolerance_m"], "wrong interval length")
    else:
        actual = payload["lengths_m"]
        require(type(actual) is list and len(actual) == 2, "pair is not ordered A/B")
        for got, want in zip(actual, expected["lengths_m"]):
            require(number(got) > 0 and abs(got - want) <= criteria["length_tolerance_m"], "wrong raw side length")
        mismatch = number(payload["mismatch_b_minus_a_m"])
        require(mismatch == actual[1] - actual[0], "mismatch is not exact returned B minus A")
        require(abs(mismatch - expected["mismatch_b_minus_a_m"]) <= criteria["mismatch_tolerance_m"], "wrong independent signed mismatch")


def text_and_structured(result):
    texts = [part["text"] for part in result.get("content", []) if part.get("type") == "text"]
    require(len(texts) == 1 and "structuredContent" in result, "missing text/structured payload")
    payload = result["structuredContent"]
    require(parse(texts[0]) == payload, "MCP text/structured JSON differs")
    return payload


def grade_error(response, expected_code=None):
    if "error" in response:
        error = response["error"]
        require(error.get("code") in (-32602, -32700), "unexpected RPC error class")
        require(bool(error.get("message")), "RPC error has no reason")
        if expected_code is not None:
            require(error["code"] == expected_code, "wrong parse error code")
    else:
        require(expected_code is None, "malformed JSON was not a parse error")
        result = response.get("result", {})
        require(result.get("isError") is True, "invalid input became success")
        payload = text_and_structured(result)
        require(type(payload) is dict and bool(payload.get("error")), "tool error has no reason")
        require(not any(key.startswith("length") for key in payload), "error contains stale length")


def grade_schema(result):
    tools = [tool for tool in result.get("tools", []) if tool.get("name") == "project_lengths"]
    require(len(tools) == 1, "project_lengths is not uniquely advertised")
    tool = tools[0]
    for key, value in (("readOnlyHint", True), ("destructiveHint", False), ("openWorldHint", False)):
        require(tool.get("annotations", {}).get(key) is value, f"wrong {key}")
    schema = tool["inputSchema"]

    def resolve(node):
        while "$ref" in node:
            reference = node["$ref"]
            require(reference.startswith("#/"), "external schema reference")
            node = schema
            for part in reference[2:].split("/"):
                node = node[part.replace("~1", "/").replace("~0", "~")]
        return node

    def closed(node, fields):
        node = resolve(node)
        require(node.get("type") == "object" and node.get("additionalProperties") is False, "object schema is open")
        require(set(node.get("required", [])) == fields and set(node.get("properties", {})) == fields, "wrong required schema fields")
        return node["properties"]

    properties = closed(schema, {"path", "query"})
    require(resolve(properties["path"]).get("type") == "string", "path schema is not string")
    variants = resolve(properties["query"]).get("oneOf", [])
    require(len(variants) == 3, "query schema does not have three oneOf variants")
    seen = set()
    for variant in variants:
        fields = closed(variant, {"type", "data"})
        tag = resolve(fields["type"])
        kind = tag.get("const", tag.get("enum", [None])[0])
        require(kind in ("interval", "seam", "sewing_group") and kind not in seen, "wrong query variant")
        require("const" in tag or tag.get("enum") == [kind], "query tag admits another value")
        seen.add(kind)
        data = closed(fields["data"], {"curve", "t0", "t1"} if kind == "interval" else {"id"})
        ident = resolve(data["curve" if kind == "interval" else "id"])
        require(ident.get("type") == "integer" and (type(ident.get("minimum")) in (int, float) and ident["minimum"] == 1 or type(ident.get("exclusiveMinimum")) in (int, float) and ident["exclusiveMinimum"] == 0), "ID schema is not positive integer")
        for key in ("t0", "t1") if kind == "interval" else ():
            endpoint = resolve(data[key])
            require(endpoint.get("type") == "number" and endpoint.get("minimum") == 0 and endpoint.get("maximum") == 1, "endpoint schema is not bounded [0,1]")


def reversal_relations(outputs):
    for prefix in ("straight", "parabola"):
        require(outputs[prefix + "-partial"]["length_m"] == outputs[prefix + "-reversed"]["length_m"], "reversed metric differs")


def self_check(criteria):
    require(criteria["length_tolerance_m"] == 1e-8 and criteria["mismatch_tolerance_m"] == 2e-8, "tolerance freeze changed")
    cases = criteria["success_cases"]
    primitive = lambda t: t * math.sqrt(1 + 4 * t * t) / 2 + math.asinh(2 * t) / 4
    for case in cases:
        grade(case["expected"], case, criteria)
        query = case["query"]
        if query["type"] == "interval":
            data = query["data"]
            want = .5 * abs(data["t1"] - data["t0"]) if data["curve"] == 25 else abs(primitive(data["t1"]) - primitive(data["t0"]))
            require(abs(want - case["expected"]["length_m"]) < 1e-15, "frozen analytic expectation differs")
    pair_oracles = ([primitive(.8) - primitive(.2), .4], [primitive(.94) - primitive(.66) + primitive(.37) - primitive(.05), .9])
    for case, oracle in zip(cases[-2:], pair_oracles):
        require(all(abs(a - b) < 1e-15 for a, b in zip(case["expected"]["lengths_m"], oracle)), "frozen raw sum differs")
    outputs = {case["label"]: case["expected"] for case in cases}
    reversal_relations(outputs)
    require(abs(outputs["parabola-left"]["length_m"] + outputs["parabola-right"]["length_m"] - primitive(1)) < 1e-15, "analytic additivity differs")
    mutations = [(0, "length_m", cases[0]["expected"]["length_m"] + 4e-8), (8, "mismatch_b_minus_a_m", -cases[8]["expected"]["mismatch_b_minus_a_m"]), (0, "length_unit", "mm"), (0, "source_revision", 136), (0, "length_m", float("nan"))]
    for index, key, value in mutations:
        bad = copy.deepcopy(cases[index]["expected"])
        bad[key] = value
        try:
            grade(bad, cases[index], criteria)
        except AssertionError:
            continue
        raise AssertionError(f"self-check accepted bad {key}")
    try:
        text_and_structured({"content": [{"type": "text", "text": json.dumps(cases[0]["expected"])}], "structuredContent": {}})
    except AssertionError:
        pass
    else:
        raise AssertionError("self-check accepted adapter divergence")
    project = parse((ROOT / "fixture.weftforge").read_text())["project"]
    ids = [obj["id"] for key in ("points", "curves", "pieces", "materials", "seams", "sewing_groups") for obj in project[key]]
    require(len(ids) == len(set(ids)) and min(ids) > 0 and max(ids) < project["next_id"] == 33 and project["revision"] == 137, "fixture ID/revision mismatch")
    curves = {curve["id"]: curve for curve in project["curves"]}
    for piece in project["pieces"]:
        boundary = piece["boundary"]
        require(all(curves[a]["end"] == curves[b]["start"] for a, b in zip(boundary, boundary[1:] + boundary[:1])), "fixture boundary is not closed")
    require(curves[15]["controls"] == [[1/3, 0.], [2/3, 1/3]] and 2/3 > 0 and .3*.4/2 > 0, "analytic CCW fixture differs")


def snapshot(directory):
    return {str(path.relative_to(directory)): path.read_bytes() if path.is_file() else None for path in directory.rglob("*")}


def hashes(files):
    return {name: sha(data) if data is not None else None for name, data in files.items()}


def run_candidate(binary, source, directory, criteria, artifact_hashes):
    transcript = []
    report = {"status": "failed", "candidate_binary": binary.name, "binary_sha256": sha(binary.read_bytes()), "source_identity_supplied_by_root": source, "frozen_artifacts": artifact_hashes}
    fixture = (ROOT / "fixture.weftforge").read_bytes()
    for name, data in {"input.weftforge": fixture, "input.weftforge.recovery": fixture + b" \t\n", "invalid.weftforge": b'{"format":"weftforge","version":3,"project":', "invalid.weftforge.recovery": fixture, "unrelated.weftforge": b"unrelated destination retained\n", "unrelated.weftforge.recovery": b"unrelated recovery retained\n"}.items():
        (directory / name).write_bytes(data)

    def invoke(arguments, stdin=None):
        before = snapshot(directory)
        process = subprocess.run([str(binary), *arguments], input=stdin, text=True, capture_output=True, cwd=directory, timeout=30)
        after = snapshot(directory)
        transcript.append({"arguments": arguments, "stdin": stdin, "returncode": process.returncode, "stdout": process.stdout, "stderr": process.stderr, "before_sha256": hashes(before), "after_sha256": hashes(after)})
        require(before == after, "query changed file bytes or directory entries")
        return process

    def cli_cases(path):
        outputs = {}
        for case in criteria["success_cases"]:
            process = invoke(["lengths", path, *case["cli"]])
            require(process.returncode == 0, f"CLI failed {case['label']}: {process.stderr}")
            payload = parse(process.stdout)
            grade(payload, case, criteria)
            outputs[case["label"]] = payload
        reversal_relations(outputs)
        return outputs

    def call_line(ident, path, query, profile):
        meta = ',"_meta":' + json.dumps(META) if profile == "current" else ""
        arguments = '{"path":' + json.dumps(path) + ',"query":' + query + '}'
        return '{"jsonrpc":"2.0","id":' + str(ident) + ',"method":"tools/call","params":{"name":"project_lengths","arguments":' + arguments + meta + '}}'

    def mcp(path, profile, cli):
        params = {"_meta": META} if profile == "current" else {}
        first = {"jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": params} if profile == "current" else {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "independent-length-evaluator", "version": "1"}}}
        lines = [json.dumps(first)]
        operations = [("profile", 1, None)]
        if profile == "legacy":
            lines.append('{"jsonrpc":"2.0","method":"notifications/initialized"}')
        lines.append(json.dumps({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": params}))
        operations.append(("schema", 2, None))
        for case in criteria["success_cases"]:
            ident = len(operations) + 10
            lines.append(call_line(ident, path, json.dumps(case["query"]), profile))
            operations.append(("success", ident, case))
        for negative in criteria["negative_cases"]:
            if "query_json" in negative:
                ident = len(operations) + 10
                lines.append(call_line(ident, path, negative["query_json"], profile))
                operations.append(("error", ident, negative))
        case = criteria["success_cases"][0]
        for kind, query_path in (("error", "invalid.weftforge"), ("success", path)):
            ident = len(operations) + 10
            lines.append(call_line(ident, query_path, json.dumps(case["query"]), profile))
            operations.append((kind, ident, case if kind == "success" else {}))
        process = invoke(["mcp"], "\n".join(lines) + "\n")
        require(process.returncode == 0, f"MCP {profile} exited: {process.stderr}")
        responses = [parse(line) for line in process.stdout.splitlines()]
        require(len(responses) == len(operations), "MCP response count/stdout differs")
        for response, (kind, ident, data) in zip(responses, operations):
            require(response.get("jsonrpc") == "2.0", "wrong JSON-RPC envelope")
            require(response.get("id") == ident or (kind == "error" and data.get("rpc_error_code") == -32700 and response.get("id") is None), "wrong response ID/order")
            if kind == "error":
                grade_error(response, data.get("rpc_error_code"))
                continue
            require("result" in response and "error" not in response, "MCP success failed")
            result = response["result"]
            if profile == "current":
                require(result.get("resultType") == "complete", "missing current resultType")
            if kind == "schema":
                grade_schema(result)
            elif kind == "profile" and profile == "legacy":
                require(result.get("protocolVersion") == "2025-11-25", "wrong legacy negotiation")
            elif kind == "success":
                require(result.get("isError") is not True, "MCP query failed")
                payload = text_and_structured(result)
                grade(payload, data, criteria)
                require(payload == cli[data["label"]], "CLI/MCP parsed JSON differs")

    try:
        baseline = cli_cases("input.weftforge")
        for negative in criteria["negative_cases"]:
            if "cli" in negative:
                process = invoke(["lengths", "input.weftforge", *negative["cli"]])
                require(process.returncode != 0 and bool(process.stderr.strip()) and not process.stdout.strip(), f"CLI invalid input was not an explicit error: {negative['label']}")
        process = invoke(["lengths", "invalid.weftforge", *criteria["success_cases"][0]["cli"]])
        require(process.returncode != 0 and bool(process.stderr.strip()) and not process.stdout.strip(), "invalid primary was silently recovered")
        for profile in ("current", "legacy"):
            mcp("input.weftforge", profile, baseline)
        variant = parse(fixture)
        for resolution in (.006, .49):
            variant["project"]["resolution"] = resolution
            for index, piece in enumerate(variant["project"]["pieces"]):
                piece["placement"] = {"translation": [17.0 + index, -13.0, 9.0], "rotation": [-.91, .73 + index, -.34]}
            (directory / "isolation.weftforge").write_text(json.dumps(variant, allow_nan=False) + "\n")
            require(cli_cases("isolation.weftforge") == baseline, "placement/resolution changed authored JSON")
        for profile in ("current", "legacy"):
            mcp("isolation.weftforge", profile, baseline)
        report.update(status="passed", retained_file_sha256=hashes(snapshot(directory)), process_count=len(transcript))
    except Exception as error:
        report["failure"] = str(error)
        raise
    finally:
        (directory / "transcript.json").write_text(json.dumps(transcript, indent=2, allow_nan=False) + "\n")
        (directory / "report.json").write_text(json.dumps(report, indent=2, allow_nan=False) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    actions.add_parser("self-check")
    run = actions.add_parser("run")
    run.add_argument("--candidate", type=Path, required=True)
    run.add_argument("--source", required=True)
    run.add_argument("--run-id", required=True)
    arguments = parser.parse_args()
    artifacts = frozen()
    criteria = parse((ROOT / "criteria.json").read_text())
    self_check(criteria)
    if arguments.action == "self-check":
        print("PASS: frozen analytic fixture/criteria and bad length/sign/units/revision/nonfinite/adapter self-checks; no candidate executed")
        return
    require(re.fullmatch(r"[A-Za-z0-9_-]+", arguments.run_id), "unsafe run ID")
    require(re.fullmatch(r"[0-9a-fA-F]{7,40}", arguments.source), "root source identity must be a Git commit")
    binary = arguments.candidate.resolve(strict=True)
    directory = ROOT / "runs" / arguments.run_id
    directory.mkdir(parents=True, exist_ok=False)
    run_candidate(binary, arguments.source, directory, criteria, artifacts)
    print(f"PASS: independent CLI/MCP acceptance; runs/{arguments.run_id}/report.json")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
