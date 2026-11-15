"""Frozen independent adapter protocol reuse, plus bounded raw-process recording; no production geometry."""
import copy, hashlib, json, math, subprocess
from pathlib import Path
MODERN="2026-07-28"
LEGACY="2025-11-25"
TOL=1e-12

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

_unrecorded_process = run_process
AUDIT_ROOT = None
AUDIT_STARTED = None
def run_process(cli, args, cwd, stdin=None):
    if AUDIT_ROOT is None:
        return _unrecorded_process(cli, args, cwd, stdin)
    import time
    count = len(list(AUDIT_ROOT.glob("*-before.json")))
    require(count < 500 and time.monotonic() - AUDIT_STARTED < 900, "Prospective process/time cap reached")
    stem = AUDIT_ROOT / str(count + 1).zfill(4)
    Path(str(stem) + "-before.json").write_bytes(encoded(dict(args=args, cwd=cwd.relative_to(AUDIT_ROOT.parent).as_posix())))
    try:
        result = _unrecorded_process(cli, args, cwd, stdin)
    except subprocess.TimeoutExpired as error:
        Path(str(stem) + ".stdout").write_bytes(error.stdout or b"")
        Path(str(stem) + ".stderr").write_bytes(error.stderr or b"")
        Path(str(stem) + "-after.json").write_bytes(encoded(dict(status="timeout")))
        raise
    Path(str(stem) + ".stdout").write_bytes(result.stdout)
    Path(str(stem) + ".stderr").write_bytes(result.stderr)
    Path(str(stem) + "-after.json").write_bytes(encoded(dict(returncode=result.returncode)))
    return result
