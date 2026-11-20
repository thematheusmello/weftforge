"""Run the release CLI and current stdio MCP against the pocket fixture.

Usage: python3 check_cutout_adapters.py WEFTFORGE_CLI pocket.svg imported.weftforge imported.obj
Optionally pass --work NEW_DIRECTORY to retain adapter outputs.
"""

import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Stdio:
    def __init__(self, executable, cwd):
        self.proc = subprocess.Popen(
            [executable, "mcp"],
            cwd=cwd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        self.seq = 0

    def request(self, method, **params):
        self.seq += 1
        params["_meta"] = {
            "io.modelcontextprotocol/protocolVersion": "2026-07-28",
            "io.modelcontextprotocol/clientCapabilities": {},
        }
        message = {"jsonrpc": "2.0", "id": self.seq, "method": method, "params": params}
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()
        response = json.loads(self.proc.stdout.readline())
        assert response["id"] == self.seq and "error" not in response, response
        return response["result"]

    def tool(self, name, **arguments):
        return self.request("tools/call", name=name, arguments=arguments)

    def close(self):
        self.proc.stdin.close()
        assert self.proc.wait(timeout=10) == 0
        self.proc.stdout.close()


def test_adapter(executable, mode, source, native, native_obj, path):
    path.mkdir()
    svg = "pocket.svg"
    primary = "pocket.weftforge"
    recovery = primary + ".recovery"
    obj = "pocket.obj"
    (path / svg).write_bytes(source.read_bytes())
    text = source.read_text()
    invalid = {
        "incomplete_xml": text.replace("</svg>", ""),
        "same_nonzero_winding": text.replace(
            'fill-rule="evenodd"', 'fill-rule="nonzero"'
        ),
        "later_cutout_outside": text.replace(
            "<path ", '<path d="M0 0L10 0 10 -10 0 -10Z"/>\n  <path ', 1
        ).replace("160 -240", "260 -240", 1),
    }

    def cli(*args, check=True):
        return subprocess.run(
            [executable, *args],
            cwd=path,
            capture_output=True,
            text=True,
            check=check,
            timeout=30,
        )

    server = Stdio(executable, path) if mode == "mcp" else None
    try:
        if server:
            server.request("server/discover")
            tools = {row["name"] for row in server.request("tools/list")["tools"]}
            assert {
                "project_create",
                "project_import_svg",
                "project_export_obj",
            } <= tools
            assert not server.tool("project_create", output=primary).get(
                "isError", False
            )
        else:
            cli("new", primary)
        blank = (path / primary).read_bytes()
        if server:
            result = server.tool(
                "project_import_svg", input=primary, output=primary, svg=svg
            )
            assert (
                not result.get("isError", False) and result["resultType"] == "complete"
            )
            imported = result["structuredContent"]
            assert not server.tool("project_export_obj", input=primary, output=obj).get(
                "isError", False
            )
        else:
            imported = json.loads(cli("import-svg", primary, primary, svg).stdout)
            cli("export-obj", primary, obj)
        assert imported["pieces_imported"] == 1 and imported["holes_imported"] == 1
        assert imported["metres_per_user_unit"] == 0.001
        assert (path / recovery).read_bytes() == blank and blank != (
            path / primary
        ).read_bytes()
        generated = json.loads((path / primary).read_bytes())
        expected = copy.deepcopy(native)
        revision = generated["project"]["revision"]
        assert (
            revision
            == imported["revision"]
            == json.loads(blank)["project"]["revision"] + 1
        )
        expected["project"]["revision"] = revision
        assert (
            generated == expected
            and (path / obj).read_bytes() == native_obj.read_bytes()
        )
        before = [digest(path / name) for name in [primary, recovery]]
        refusals = {}
        for name, bad in invalid.items():
            filename = name + ".svg"
            file = path / filename
            file.write_text(bad)
            if server:
                res = server.tool(
                    "project_import_svg", input=primary, output=primary, svg=filename
                )
                assert res["isError"] and res["resultType"] == "complete"
                detail = res["content"][0]["text"]
            else:
                res = cli("import-svg", primary, primary, filename, check=False)
                assert res.returncode != 0
                detail = res.stderr.strip()
            assert detail and file.read_text() == bad
            assert before == [digest(path / name) for name in [primary, recovery]]
            refusals[name] = {"reason": detail, "primary_and_recovery_byte_equal": True}
        assert (path / svg).read_bytes() == source.read_bytes()
        return {
            "revision": revision,
            "pieces_imported": imported["pieces_imported"],
            "holes_imported": imported["holes_imported"],
            "metres_per_user_unit": imported["metres_per_user_unit"],
            "complete_native_document_equal_except_revision": True,
            "native_obj_byte_equal": True,
            "primary_sha256": before[0],
            "recovery_sha256": before[1],
            "obj_sha256": digest(path / obj),
            "rejections": refusals,
        }
    finally:
        if server:
            server.close()


def run(executable, svg, project, obj, directory):
    doc = json.loads(project.read_bytes())
    paths = [svg, project, obj]
    before = list(map(digest, paths))
    results = {}
    for mode in ["cli", "mcp"]:
        results[mode] = test_adapter(executable, mode, svg, doc, obj, directory / mode)
    for key in ["primary_sha256", "recovery_sha256", "obj_sha256"]:
        assert results["cli"][key] == results["mcp"][key]
    assert before == list(map(digest, paths))
    return {
        "executable_sha256": digest(executable),
        "mcp_profile": "2026-07-28",
        "frozen_inputs_preserved": True,
        "adapters": results,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["executable", "svg", "project", "obj"]:
        parser.add_argument(name, type=Path)
    parser.add_argument("--work", type=Path)
    arg = parser.parse_args()
    exe = arg.executable.resolve()
    if arg.work:
        arg.work.mkdir()
        output = run(exe, arg.svg, arg.project, arg.obj, arg.work)
    else:
        with tempfile.TemporaryDirectory() as temporary:
            output = run(exe, arg.svg, arg.project, arg.obj, Path(temporary))
    print(json.dumps(output, indent=2))
