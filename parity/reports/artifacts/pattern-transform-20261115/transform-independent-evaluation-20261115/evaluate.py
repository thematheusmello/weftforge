#!/usr/bin/env python3
"""Frozen, stdlib-only acceptance; no production math or model imports."""
import copy
import hashlib
import json
import math
from pathlib import Path
import re
import select
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
PLAN = json.loads((ROOT / "plan.json").read_text())
FIXTURE = json.loads((ROOT / "fixture.weftforge").read_text())
TOL = PLAN["tolerances"]


def require(ok, message):
    if not ok:
        raise AssertionError(message)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits(number):
    return struct.pack(">d", float(number))


def exact(old, new, path="root"):
    if isinstance(old, dict):
        require(isinstance(new, dict) and old.keys() == new.keys(), path + ": keys")
        for key in old:
            exact(old[key], new[key], path + "." + key)
    elif isinstance(old, list):
        require(isinstance(new, list) and len(old) == len(new), path + ": array")
        for index, pair in enumerate(zip(old, new)):
            exact(*pair, path + "[" + str(index) + "]")
    elif isinstance(old, float) or isinstance(new, float):
        require(bits(old) == bits(new), path + ": f64 bits changed")
    else:
        require(old == new, path + ": value changed")


def error(a, b):
    require(len(a) == len(b), "vector dimensions")
    require(all(math.isfinite(x) for x in [*a, *b]), "nonfinite result")
    return max(abs(x - y) for x, y in zip(a, b))


def near(a, b, tolerance, message):
    value = error(a, b)
    require(value <= tolerance, f"{message}: {value} > {tolerance}")
    return value


def multiply(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3))
             for j in range(3)] for i in range(3)]


def vector(matrix, value):
    return [sum(row[k] * value[k] for k in range(3)) for row in matrix]


def rotation(euler):
    x, y, z = euler
    cx, cy, cz = math.cos(x), math.cos(y), math.cos(z)
    sx, sy, sz = math.sin(x), math.sin(y), math.sin(z)
    rx = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]]
    ry = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]]
    rz = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]]
    return multiply(multiply(rz, ry), rx)


def world(placement, uv):
    return [a + b for a, b in zip(vector(rotation(placement["rotation"]),
                                       [*uv, 0.0]), placement["translation"])]


def transform(uv, command, oracle=None):
    d, p, angle = (command[key] for key in ("translation", "pivot", "angle"))
    if angle == 0.0:
        return [uv[i] if d[i] == 0.0 else uv[i] + d[i] for i in range(2)]
    if oracle == "quarter":
        return [1.25 - uv[1], uv[0]]
    x, y = uv[0] - p[0], uv[1] - p[1]
    c, s = (0.0, 1.0) if oracle == "quarter_general" else (math.cos(angle), math.sin(angle))
    return [p[0] + c*x - s*y + d[0], p[1] + s*x + c*y + d[1]]


def by_id(project, kind, object_id):
    return next(item for item in project[kind] if item["id"] == object_id)


def curve_at(project, curve, t):
    a = by_id(project, "points", curve["start"])["position"]
    b = by_id(project, "points", curve["end"])["position"]
    if curve["controls"] is None:
        return [(1.0-t)*a[i] + t*b[i] for i in range(2)]
    c, d = curve["controls"]
    return [(1.0-t)**3*a[i] + 3*(1.0-t)**2*t*c[i]
            + 3*(1.0-t)*t*t*d[i] + t**3*b[i] for i in range(2)]


def source_for(case):
    document = copy.deepcopy(FIXTURE)
    project = document["project"]
    piece = by_id(project, "pieces", 2)
    for case_key, placement_key in [("rotation", "rotation"),
                                    ("placement_translation", "translation")]:
        if case_key in case:
            piece["placement"][placement_key] = case[case_key]
    if "grain_angle" in case:
        piece["grain_angle"] = case["grain_angle"]
    if "coordinate_offset" in case:
        delta = case["coordinate_offset"]
        for point in project["points"]:
            if point["id"] in {3, 4, 5, 6}:
                point["position"] = [point["position"][i] + delta[i] for i in range(2)]
        first = by_id(project, "curves", 7)
        first["controls"] = [[uv[i] + delta[i] for i in range(2)] for uv in first["controls"]]
    return document


def grade(old_document, new_document, command, case):
    require(new_document.get("format") == "weftforge" and new_document.get("version") == 3,
            "native v3 envelope")
    old, new = old_document["project"], new_document["project"]
    require(new["revision"] == old["revision"] + 1, "exactly one transaction revision")
    old_piece, new_piece = (by_id(p, "pieces", command["piece"]) for p in (old, new))
    curve_ids = set(old_piece["boundary"])
    point_ids = {c[k] for c in old["curves"] if c["id"] in curve_ids for k in ("start", "end")}
    masked_old, masked_new = copy.deepcopy(old), copy.deepcopy(new)
    for project in (masked_old, masked_new):
        project["revision"] = None
        for point in project["points"]:
            if point["id"] in point_ids:
                point["position"] = None
        for curve in project["curves"]:
            if curve["id"] in curve_ids and by_id(old, "curves", curve["id"])["controls"] is not None:
                curve["controls"] = None
        piece = by_id(project, "pieces", command["piece"])
        piece["placement"] = piece["grain_angle"] = None
    exact(masked_old, masked_new, "authored_invariants")
    tolerance = case.get("geometry_tolerance_m", TOL["geometry_m"])
    oracle = case.get("oracle")
    maxima = dict(coordinate_m=0.0, polynomial_m=0.0, world_m=0.0)
    for kind in ("points", "curves"):
        for item in old[kind]:
            if kind == "points":
                if item["id"] not in point_ids:
                    continue
                before = [item["position"]]
                after = [by_id(new, kind, item["id"])["position"]]
            else:
                if item["id"] not in curve_ids or item["controls"] is None:
                    continue
                before = item["controls"]
                after = by_id(new, kind, item["id"])["controls"]
                require(isinstance(after, list) and len(after) == 2, "cubic representation retained")
            for uv, actual in zip(before, after):
                expected = transform(uv, command, oracle)
                if command["angle"] == 0.0:
                    exact(expected, actual, "single-addition coordinate")
                maxima["coordinate_m"] = max(maxima["coordinate_m"], near(
                    expected, actual, tolerance, "authored endpoint/control"))
    offset = case.get("coordinate_offset", [0.0, 0.0])
    for curve_id in old_piece["boundary"]:
        for t in PLAN["sample_t"]:
            before = ([offset[0]+0.75*t, offset[1]+0.75*t*t]
                      if curve_id == 7 and not case.get("source_is_transformed")
                      else curve_at(old, by_id(old, "curves", curve_id), t))
            after = curve_at(new, by_id(new, "curves", curve_id), t)
            maxima["polynomial_m"] = max(maxima["polynomial_m"], near(
                transform(before, command, oracle), after, tolerance, "independent curve sample"))
            maxima["world_m"] = max(maxima["world_m"], near(
                world(old_piece["placement"], before), world(new_piece["placement"], after),
                tolerance, "arranged curve sample"))
    a, anew = (rotation(p["placement"]["rotation"]) for p in (old_piece, new_piece))
    expected_a = multiply(a, rotation([0.0, 0.0, -command["angle"]]))
    maxima["rotation_matrix"] = near(sum(expected_a, []), sum(anew, []),
                                      TOL["rotation_matrix_component"], "right composition")
    pivot_delta = [command["pivot"][i] + command["translation"][i] for i in range(2)]
    expected_translation = [x-y for x, y in zip(world(old_piece["placement"], command["pivot"]),
                                               vector(expected_a, [*pivot_delta, 0.0]))]
    if command["angle"] == 0.0:
        exact(old_piece["placement"]["rotation"], new_piece["placement"]["rotation"],
              "translation retains Euler bits")
        expected_translation = [x-y for x,y in zip(old_piece["placement"]["translation"],
                                                   vector(a, [*command["translation"], 0.0]))]
        exact(old_piece["grain_angle"], new_piece["grain_angle"], "translation retains grain bits")
    near(expected_translation, new_piece["placement"]["translation"], tolerance, "placement origin")
    old_grain, grain = (p["grain_angle"] for p in (old_piece, new_piece))
    old_warp = [math.cos(old_grain), math.sin(old_grain), 0.0]
    actual_warp = [math.cos(grain), math.sin(grain), 0.0]
    expected_warp = vector(rotation([0.0, 0.0, command["angle"]]), old_warp)
    if command["angle"] != 0.0:
        require(-math.pi <= grain <= math.pi, "canonical rotated grain")
    maxima["grain"] = near(expected_warp, actual_warp, TOL["grain_component"], "material warp")
    maxima["world_warp"] = near(vector(a, old_warp), vector(anew, actual_warp),
                                 TOL["world_warp_component"], "world warp")
    return maxima


def primitive(t):
    return 0.375*t*math.sqrt(1+4*t*t) + 0.1875*math.asinh(2*t)


def interval_length(project, interval):
    curve = by_id(project, "curves", interval["curve"])
    if curve["id"] in (7, 16):
        return abs(primitive(interval["t1"]) - primitive(interval["t0"]))
    require(curve["controls"] is None, "declared analytic/straight metric only")
    a, b = (by_id(project, "points", curve[k])["position"] for k in ("start", "end"))
    return math.dist(a, b) * abs(interval["t1"] - interval["t0"])


QUERIES = [{"type":"interval", "data":{"curve":7,"t0":0.875,"t1":0.125}},
           {"type":"seam", "data":{"id":20}}, {"type":"sewing_group", "data":{"id":21}}]


def grade_lengths(project, reports):
    values, maximum = [], 0.0
    for query, report in zip(QUERIES, reports):
        exact(query, report["query"], "length query")
        require(report["source_revision"] == project["revision"] and report["length_unit"] == "m",
                "length revision/metres")
        if query["type"] == "interval":
            expected = [interval_length(project, query["data"])]
            actual = [report["length_m"]]
        else:
            kind = "seams" if query["type"] == "seam" else "sewing_groups"
            item = by_id(project, kind, query["data"]["id"])
            expected = [sum(interval_length(project, interval) for interval in
                            ([item[side]] if kind == "seams" else item[side])) for side in ("a", "b")]
            actual = report["lengths_m"]
            near([actual[1]-actual[0]], [report["mismatch_b_minus_a_m"]], 1e-15, "signed mismatch")
        maximum = max(maximum, near(expected, actual, TOL["analytic_length_m"], "analytic sewing length"))
        values.extend(actual)
    return values, maximum


def grade_mesh(document, report):
    exact(document, report["source"], "mesh native source")
    project, mesh = document["project"], report["mesh"]
    require(mesh["doc_revision"] == project["revision"], "fresh rest revision")
    require(len(mesh["positions"]) == len(mesh["uv"]) == len(mesh["piece_ids"]), "mesh lengths")
    maximum = 0.0
    for uv, position, piece_id in zip(mesh["uv"], mesh["positions"], mesh["piece_ids"]):
        piece = by_id(project, "pieces", piece_id)
        maximum = max(maximum, near(world(piece["placement"], uv), position,
                                    TOL["geometry_m"], "fresh UV world embedding"))
    for boundary in mesh["boundary"]:
        piece = by_id(project, "pieces", boundary["piece"])
        require(boundary["curve"] in piece["boundary"], "boundary ownership")
        expected = curve_at(project, by_id(project, "curves", boundary["curve"]), boundary["t"])
        near(expected, mesh["uv"][boundary["vertex"]], TOL["geometry_m"], "UV authored association")
    require({stitch["seam"] for stitch in mesh["stitches"]} == {20, 21}, "sewing IDs in fresh mesh")
    require(all(len(tri) == 3 and all(0 <= v < len(mesh["uv"]) for v in tri)
                for tri in mesh["triangles"]), "triangle indices")
    return {"vertices":len(mesh["uv"]), "triangles":len(mesh["triangles"]), "world_error_m":maximum}


def check_schema(result):
    tools = result["tools"]
    require(len(tools) == 9, "existing tool count")
    schema = next(tool for tool in tools if tool["name"] == "project_execute")["inputSchema"]
    variants = schema["properties"]["commands"]["items"]["oneOf"]
    matches = [v for v in variants if v["properties"]["type"].get("const") == "transform_piece"]
    require(len(matches) == 1, "one shared transform schema variant")
    variant, = matches
    require(variant["additionalProperties"] is False and set(variant["required"]) == {"type", "data"},
            "closed tagged command schema")
    data = variant["properties"]["data"]
    require(data["additionalProperties"] is False and set(data["required"]) ==
            set(data["properties"]) == {"piece", "translation", "angle", "pivot"}, "required transform fields")
    piece = data["properties"]["piece"]
    require(piece["type"] == "integer" and piece["minimum"] == 1, "nonzero schema piece ID")
    require(data["properties"]["angle"]["type"] == "number", "numeric angle schema")
    for key in ("translation", "pivot"):
        value = data["properties"][key]
        require(value["type"] == "array" and value["minItems"] == value["maxItems"] == 2
                and value["items"]["type"] == "number", "fixed two-number " + key)


class Adapter:
    def __init__(self, binary, route, directory, journal):
        self.binary, self.route, self.directory, self.journal = binary, route, directory, journal
        self.process, self.request_id = None, 0
        if route != "cli":
            self.process = subprocess.Popen([str(binary), "mcp"], cwd=directory,
                                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                if route == "mcp-legacy":
                    response = self.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{},
                                                            "clientInfo":{"name":"independent-transform", "version":"1"}})
                    require(response["result"]["protocolVersion"] == "2025-11-25", "legacy handshake")
                    self.send({"jsonrpc":"2.0", "method":"notifications/initialized"})
                else:
                    require(self.request("server/discover", {})["result"]["resultType"] == "complete",
                            "current discovery")
                check_schema(self.request("tools/list", {})["result"])
            except Exception:
                self.close()
                raise

    def record(self, value):
        self.journal.write(json.dumps({"route":self.route, **value}, allow_nan=False) + "\n")
        self.journal.flush()

    def send(self, value):
        raw = value if isinstance(value, str) else json.dumps(value, separators=(",", ":"), allow_nan=False)
        self.process.stdin.write((raw + "\n").encode())
        self.process.stdin.flush()

    def request(self, method, params, raw_commands=None):
        self.request_id += 1
        if self.route == "mcp-current":
            params = {**params, "_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28",
                                        "io.modelcontextprotocol/clientCapabilities":{}}}
        request = {"jsonrpc":"2.0", "id":self.request_id, "method":method, "params":params}
        if raw_commands is not None:
            request["params"]["arguments"]["commands"] = "RAW_COMMANDS_PLACEHOLDER"
            request = json.dumps(request, separators=(",", ":")).replace('"RAW_COMMANDS_PLACEHOLDER"', raw_commands)
        self.send(request)
        require(select.select([self.process.stdout], [], [], 30)[0], "MCP response timeout")
        line = self.process.stdout.readline()
        require(line, "MCP exited without response")
        response = json.loads(line)
        self.record({"request":request, "response":response})
        require(response.get("id") == self.request_id, "matching JSON-RPC ID")
        if self.route == "mcp-current" and "result" in response:
            require(response["result"].get("resultType") == "complete", "current complete result")
        return response

    def cli(self, arguments):
        result = subprocess.run([str(self.binary), *arguments], cwd=self.directory,
                                capture_output=True, text=True, timeout=60)
        self.record({"argv":[self.binary.name, *arguments], "exit_code":result.returncode,
                     "stdout":result.stdout, "stderr":result.stderr})
        return result

    def call(self, name, arguments, raw_commands=None, success=True):
        response = self.request("tools/call", {"name":name, "arguments":arguments}, raw_commands)
        if not success:
            rejected = "error" in response or response.get("result", {}).get("isError") is True
            require(rejected, "invalid MCP command unexpectedly succeeded")
            detail = response.get("error", response.get("result", {}).get("structuredContent", {}))
            require(detail, "useful MCP rejection")
            return detail
        require("result" in response and response["result"].get("isError") is False, "MCP tool success")
        result = response["result"]
        data = result["structuredContent"]
        exact(data, json.loads(result["content"][0]["text"]), "MCP structured/text agreement")
        return data

    def inspect(self, path, document):
        if self.route == "cli":
            result = self.cli(["lengths", path, "interval", "7", "0.875", "0.125"])
            require(result.returncode == 0, "CLI load/reopen precondition")
            report = json.loads(result.stdout)
            require(report["source_revision"] == document["project"]["revision"]
                    and report["length_unit"] == "m", "CLI reopened revision/metres")
            near([abs(primitive(0.875)-primitive(0.125))], [report["length_m"]],
                 TOL["analytic_length_m"], "CLI reopened analytic curve")
        else:
            result = self.call("project_inspect", {"path":path})
            exact(document["project"], result["project"], "MCP/save reopen exactness")
            exact({"length":"m", "mass":"kg", "time":"s"}, result["units"], "SI units")
            require(result["up"] == "Y", "Y up")

    def lengths(self, path):
        reports = []
        for query in QUERIES:
            if self.route == "cli":
                data, kind = query["data"], query["type"]
                tail = ([str(data["curve"]), str(data["t0"]), str(data["t1"])] if kind == "interval"
                        else [str(data["id"])])
                result = self.cli(["lengths", path, kind.replace("_", "-"), *tail])
                require(result.returncode == 0, "CLI authored length success")
                reports.append(json.loads(result.stdout))
            else:
                reports.append(self.call("project_lengths", {"path":path, "query":query}))
        return reports

    def execute(self, source, destination, commands_path, raw_commands, success):
        if self.route == "cli":
            result = self.cli(["execute", source, destination, commands_path])
            require((result.returncode == 0) == success, "CLI execute success/rejection")
            require(success or result.stderr.strip(), "useful CLI rejection")
            return json.loads(result.stdout) if success else {"error":result.stderr.strip()}
        return self.call("project_execute", {"input":source, "output":destination}, raw_commands, success)

    def close(self):
        if self.process:
            self.process.stdin.close()
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
                raise AssertionError("MCP did not exit after stdin closed")
            stderr = self.process.stderr.read().decode()
            self.record({"server_exit_code":self.process.returncode, "server_stderr":stderr})
            require(self.process.returncode == 0, "MCP clean shutdown")


def raw_commands(case):
    return case.get("raw_commands") or json.dumps([
        {"type":"transform_piece", "data":data} for data in case.get("commands", [case.get("command")])],
        separators=(",", ":"), allow_nan=False)


def frozen_files():
    freeze = json.loads((ROOT / "FREEZE.json").read_text())
    for name, digest in freeze["sha256"].items():
        require(sha(ROOT / name) == digest, "frozen artifact changed: " + name)
    return freeze


def run(binding_path):
    frozen_files()
    binding = json.loads(binding_path.read_text())
    require(binding.get("authorized") is True, "root authorization required")
    require(binding["root_reviewed_freeze_sha256"] == sha(ROOT / "FREEZE.json"), "root reviewed exact freeze")
    require(re.fullmatch(r"[0-9a-f]{40}", binding["source_commit"]), "exact candidate source commit")
    binary = Path(binding["binary"]).resolve()
    require(sha(binary) == binding["binary_sha256"], "exact candidate binary")
    directory = ROOT / "run-output"
    directory.mkdir(exist_ok=False)
    results = {"source_commit":binding["source_commit"], "binary_sha256":sha(binary),
               "freeze_sha256":sha(ROOT / "FREEZE.json"), "acceptance":"implementation only; reference unobserved",
               "cases":[], "failures":[]}
    journal = (directory / "journal.jsonl").open("w")
    for route in PLAN["routes"]:
        adapter = None
        try:
            adapter = Adapter(binary, route, directory, journal)
            for success, cases in [(True, PLAN["success"]), (False, PLAN["invalid"])]:
                for case in cases:
                    name = case["name"]
                    case_dir = directory / route / name
                    case_dir.mkdir(parents=True)
                    source_doc = source_for(case)
                    source_path, output_path = case_dir / "input.weftforge", case_dir / "output.weftforge"
                    write_json(source_path, source_doc)
                    source_path.chmod(0o444)
                    input_recovery = Path(str(source_path) + ".recovery")
                    input_recovery.write_bytes(b"READONLY INPUT RECOVERY SENTINEL\n")
                    input_recovery.chmod(0o444)
                    old_output = copy.deepcopy(FIXTURE)
                    old_output["project"]["revision"] = 13
                    by_id(old_output["project"], "pieces", 2)["grain_angle"] = -0.17
                    write_json(output_path, old_output)
                    output_recovery = Path(str(output_path) + ".recovery")
                    output_recovery.write_bytes(b"OLD OUTPUT RECOVERY SENTINEL\n")
                    unrelated = case_dir / "unrelated.destination"
                    unrelated.write_bytes(b"UNRELATED DESTINATION MUST RETAIN BYTES\n")
                    command_path = case_dir / "commands.json"
                    raw = raw_commands(case)
                    command_path.write_text(raw + "\n")
                    watched = [source_path, input_recovery, command_path, unrelated, output_path, output_recovery]
                    before = {p:p.read_bytes() for p in watched}
                    relative = lambda path: str(path.relative_to(directory))
                    record = {"route":route, "case":name, "expected_success":success}
                    try:
                        adapter.inspect(relative(source_path), source_doc)
                        if success:
                            old_lengths, old_metric_error = grade_lengths(source_doc["project"], adapter.lengths(relative(source_path)))
                        response = adapter.execute(relative(source_path), relative(output_path),
                                                   relative(command_path), raw, success)
                        record["execute"] = response
                        for path in watched[:4]:
                            require(path.read_bytes() == before[path], "unchanged read-only input/sidecar/commands/unrelated: " + path.name)
                        if success:
                            require(output_recovery.read_bytes() == before[output_path], "successful recovery is exact old primary")
                            saved_bytes = output_path.read_bytes()
                            saved = json.loads(saved_bytes)
                            require(response["revision"] == source_doc["project"]["revision"]+1
                                    and response["commands_applied"] == 1, "adapter one-command transaction report")
                            record["geometry"] = grade(source_doc, saved, case["command"], case)
                            adapter.inspect(relative(output_path), saved)
                            new_lengths, new_metric_error = grade_lengths(source_doc["project"] | {"revision":saved["project"]["revision"]},
                                                                         adapter.lengths(relative(output_path)))
                            record["length_errors_m"] = {"before_analytic":old_metric_error, "after_analytic":new_metric_error,
                                                         "before_after":near(old_lengths, new_lengths, TOL["before_after_length_m"], "length retained")}
                            if case.get("mesh"):
                                inspected = adapter.cli(["inspect-sewing", relative(output_path)])
                                require(inspected.returncode == 0, "fresh mesh inspection")
                                mesh_report = json.loads(inspected.stdout)
                                write_json(case_dir / "fresh-mesh.json", mesh_report)
                                record["fresh_mesh"] = grade_mesh(saved, mesh_report)
                            if case.get("inverse"):
                                c, s = math.cos(case["command"]["angle"]), math.sin(case["command"]["angle"])
                                du, dv = case["command"]["translation"]
                                inverse = {**case["command"], "angle":-case["command"]["angle"],
                                           "translation":[-c*du-s*dv, s*du-c*dv]}
                                inverse_path, inverse_commands = case_dir / "inverse.weftforge", case_dir / "inverse.commands.json"
                                inverse_raw = json.dumps([{"type":"transform_piece", "data":inverse}], allow_nan=False)
                                inverse_commands.write_text(inverse_raw + "\n")
                                inverse_before = output_path.read_bytes()
                                recovery_before = output_recovery.read_bytes()
                                output_path.chmod(0o444)
                                inverse_response = adapter.execute(relative(output_path), relative(inverse_path), relative(inverse_commands), inverse_raw, True)
                                require(output_path.read_bytes() == inverse_before, "inverse input retained")
                                require(output_recovery.read_bytes() == recovery_before, "inverse input recovery retained")
                                require(not Path(str(inverse_path)+".recovery").exists(), "new inverse output has no old primary recovery")
                                restored = json.loads(inverse_path.read_text())
                                require(inverse_response["revision"] == saved["project"]["revision"]+1
                                        and inverse_response["commands_applied"] == 1, "one inverse transaction")
                                record["inverse"] = grade(saved, restored, inverse, {"source_is_transformed":True})
                                for point in source_doc["project"]["points"]:
                                    near(point["position"], by_id(restored["project"], "points", point["id"])["position"],
                                         TOL["geometry_m"], "nonright inverse endpoint")
                                adapter.inspect(relative(inverse_path), restored)
                        else:
                            for path in watched:
                                require(path.read_bytes() == before[path], "failure retained every destination byte: " + path.name)
                            adapter.inspect(relative(source_path), source_doc)
                        for path in watched[:4]:
                            require(path.read_bytes() == before[path], "final unchanged input/sidecar/commands/unrelated bytes")
                        require(source_path.stat().st_mode & 0o222 == 0, "read-only input mode retained")
                        if success:
                            require(output_path.read_bytes() == saved_bytes, "inspection/inverse retained saved primary")
                            require(output_recovery.read_bytes() == before[output_path], "final exact old-primary recovery")
                        else:
                            for path in watched:
                                require(path.read_bytes() == before[path], "final failure byte retention")
                        record["status"] = "passed"
                    except Exception as problem:
                        record["status"], record["failure"] = "failed", str(problem)
                        results["failures"].append({"route":route, "case":name, "error":str(problem)})
                    record["files"] = {relative(path):{"before_sha256":hashlib.sha256(before[path]).hexdigest(),
                                                       "after_sha256":sha(path) if path.is_file() else None}
                                       for path in watched}
                    results["cases"].append(record)
                    write_json(directory / "RESULT.json", results)
        except Exception as problem:
            results["failures"].append({"route":route, "phase":"adapter/precondition", "error":str(problem)})
        finally:
            if adapter is not None:
                try:
                    adapter.close()
                except Exception as problem:
                    results["failures"].append({"route":route, "phase":"shutdown", "error":str(problem)})
    journal.close()
    require(sha(binary) == binding["binary_sha256"], "candidate binary retained")
    frozen_files()
    results["status"] = "passed" if not results["failures"] else "failed"
    write_json(directory / "RESULT.json", results)
    print(json.dumps({"status":results["status"], "cases":len(results["cases"]), "failures":results["failures"]}, indent=2))
    return bool(results["failures"])


def synthetic_layout(old, command, euler, oracle="quarter"):
    new = copy.deepcopy(old)
    project = new["project"]
    for point in project["points"]:
        if point["id"] in {3,4,5,6}:
            point["position"] = transform(point["position"], command, oracle)
    by_id(project, "curves", 7)["controls"] = [transform(q, command, oracle) for q in by_id(project, "curves", 7)["controls"]]
    old_piece, piece = by_id(old["project"], "pieces", 2), by_id(project, "pieces", 2)
    piece["placement"]["rotation"] = euler
    matrix = rotation(euler)
    pnew = [command["pivot"][i]+command["translation"][i] for i in range(2)]
    piece["placement"]["translation"] = [x-y for x,y in zip(world(old_piece["placement"], command["pivot"]), vector(matrix, [*pnew,0.0]))]
    unit = vector(rotation([0.0,0.0,command["angle"]]), [math.cos(old_piece["grain_angle"]), math.sin(old_piece["grain_angle"]),0.0])
    piece["grain_angle"] = (old_piece["grain_angle"] if command["angle"] == 0.0 else math.atan2(unit[1], unit[0]))
    project["revision"] += 1
    return new


def selfcheck():
    checks = []
    for euler, uv, expected in [([math.pi/2,0,0],[0,1],[0,0,1]), ([0,math.pi/2,0],[1,0],[0,0,-1]),
                                ([0,0,math.pi/2],[1,0],[0,1,0]), ([math.pi/2]*3,[1,1],[0,1,-1])]:
        near(vector(rotation(euler), [*uv,0.0]), expected, 1e-15, "independent matrix axis check")
    checks.append("independent XYZ axis and composition checks")
    case = next(c for c in PLAN["success"] if c["name"] == "quarter_identity_placement")
    old, command = source_for(case), case["command"]
    good = synthetic_layout(old, command, [0.0,0.0,-math.pi/2])
    grade(old, good, command, case)
    exact(PLAN["independent_expected"]["quarter_controls"], by_id(good["project"], "curves", 7)["controls"])
    for point_id, expected in PLAN["independent_expected"]["quarter_points"].items():
        exact(expected, by_id(good["project"], "points", int(point_id))["position"])
    checks.append("accepted independent exact quarter oracle")
    mutations = {
        "wrong_quarter_turn":lambda d: by_id(d["project"], "points", 4)["position"].__setitem__(0, 2.0),
        "missing_placement_compensation":lambda d: by_id(d["project"], "pieces", 2).__setitem__("placement", copy.deepcopy(by_id(old["project"], "pieces", 2)["placement"])),
        "unchanged_grain":lambda d: by_id(d["project"], "pieces", 2).__setitem__("grain_angle", 0.37),
        "millimetre_units":lambda d: by_id(d["project"], "points", 4).__setitem__("position", [500.0,750.0]),
        "changed_map":lambda d: d["project"]["seams"][0]["correspondence"][1].__setitem__(1, 0.25),
        "extra_revision":lambda d: d["project"].__setitem__("revision", 39),
    }
    for name, mutate in mutations.items():
        bad = copy.deepcopy(good)
        mutate(bad)
        try:
            grade(old, bad, command, case)
        except AssertionError:
            checks.append("rejected " + name)
        else:
            raise AssertionError("selfcheck failed to reject " + name)
    tilted_case = next(c for c in PLAN["success"] if c["name"] == "output_gimbal_plus")
    tilted = source_for(tilted_case)
    correct = synthetic_layout(tilted, command, [0.0, math.pi/2, 0.21-math.pi/2])
    grade(tilted, correct, command, tilted_case)
    bad = copy.deepcopy(correct)
    by_id(bad["project"], "pieces", 2)["placement"]["rotation"] = [math.pi/2,0.0,0.21-math.pi/2]
    try:
        grade(tilted, bad, command, tilted_case)
    except AssertionError:
        checks.append("rejected naive Z-only compensation on a tilted piece")
    else:
        raise AssertionError("selfcheck accepted naive tilted Z subtraction")
    for targeted in PLAN["success"]:
        if targeted["name"].startswith("output_"):
            original = source_for(targeted)
            expected = synthetic_layout(original, targeted["command"], [0.0,targeted["rotation"][0],0.21-math.pi/2])
            grade(original, expected, targeted["command"], targeted)
    checks.append("accepted both signs of exact/near output gimbal matrices")
    translation = next(c for c in PLAN["success"] if c["name"] == "translate_u_bypass_pivot")
    translated = synthetic_layout(FIXTURE, translation["command"], [0.31,-0.47,0.21], None)
    grade(FIXTURE, translated, translation["command"], translation)
    by_id(translated["project"], "points", 3)["position"][1] = 0.0
    try:
        grade(FIXTURE, translated, translation["command"], translation)
    except AssertionError:
        checks.append("rejected lost signed-zero bits on an unchanged translation axis")
    else:
        raise AssertionError("selfcheck accepted lost signed zero")
    nonright = next(c for c in PLAN["success"] if c["name"] == "nonright")["command"]
    c, s = math.cos(nonright["angle"]), math.sin(nonright["angle"])
    du, dv = nonright["translation"]
    inverse = {**nonright, "angle":-nonright["angle"], "translation":[-c*du-s*dv,s*du-c*dv]}
    for uv in ([0.0,0.0],[0.75,0.75],[-0.25,1.25],[0.5,0.25]):
        near(uv, transform(transform(uv, nonright), inverse), 1e-15, "nonright inverse oracle")
    simple = source_for({"rotation":[0.0,0.0,0.0]})
    forward = synthetic_layout(simple, nonright, [0.0,0.0,-nonright["angle"]], None)
    grade(simple, forward, nonright, {})
    backward = synthetic_layout(forward, inverse, [0.0,0.0,0.0], None)
    grade(forward, backward, inverse, {"source_is_transformed":True})
    checks.append("non-right-angle inverse oracle")
    overflow = next(c for c in PLAN["invalid"] if c["name"] == "control_only_result_out_of_bounds")["commands"][0]
    endpoints = [transform(p["position"], overflow) for p in FIXTURE["project"]["points"] if p["id"] in {3,4,5,6}]
    controls = [transform(p, overflow) for p in by_id(FIXTURE["project"], "curves", 7)["controls"]]
    require(all(abs(v) <= 1000 for p in endpoints for v in p) and any(abs(v) > 1000 for p in controls for v in p),
            "control overflow must leave endpoints supported")
    checks.append("control-only overflow precondition")
    report = {"status":"passed", "candidate_executed":False, "checks":checks,
              "success_rows":len(PLAN["success"]), "invalid_rows":len(PLAN["invalid"]),
              "planned_primary_cases":len(PLAN["routes"])*(len(PLAN["success"])+len(PLAN["invalid"]))}
    write_json(ROOT / "selfcheck.json", report)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    if sys.argv[1:] == ["selfcheck"]:
        selfcheck()
    elif len(sys.argv) == 3 and sys.argv[1] == "run":
        sys.exit(run(Path(sys.argv[2])))
    else:
        sys.exit("usage: evaluate.py selfcheck | run ROOT_AUTHORIZED_BINDING.json")
