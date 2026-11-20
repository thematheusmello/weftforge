"""Exercise the installed official MCP client against the same CLI binary and original exported garment surfaces."""

import argparse
import asyncio
import hashlib
import importlib.metadata
import json
from pathlib import Path
import shutil
import subprocess

from jsonschema import Draft202012Validator
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


async def measure(cli, root, case):
    case.mkdir(exist_ok=False)

    def run(*args):
        return subprocess.check_output([cli, *args], cwd=case)

    def arg_file(path):
        return str(path.resolve())

    project = case / "Empty.weftforge"
    run("new", project)
    report = {
        "binary_sha256": digest(cli),
        "official_sdk_version": importlib.metadata.version("mcp"),
        "cases": [],
    }
    references = root / "parity/reports/artifacts"
    with (case / "stderr.log").open("w") as stderr:
        async with stdio_client(
            StdioServerParameters(command=str(cli), args=["mcp"]), errlog=stderr
        ) as (stdin, stdout):
            async with ClientSession(stdin, stdout) as session:
                hello = await session.initialize()
                report["protocol_version"] = hello.protocolVersion
                response = await session.list_tools()
                toolmap = {v.name: v.inputSchema for v in response.tools}
                assert len(toolmap) == len(response.tools) == 15
                schema = toolmap["project_import_obj"]
                assert schema["additionalProperties"] is False
                execute = toolmap["project_execute"]
                variants = execute["properties"]["commands"]["items"]["oneOf"]
                (command_schema,) = [
                    v
                    for v in variants
                    if v["properties"]["type"]["const"] == "import_obj_patterns"
                ]
                for candidate in (schema, command_schema):
                    Draft202012Validator.check_schema(candidate)
                for label, source, pieces, holes, materials in (
                    (
                        "Harbor",
                        references / "native-obj-fabrics-20261120/Harbor.obj",
                        2,
                        0,
                        2,
                    ),
                    (
                        "Pocket",
                        references / "native-svg-cutout-20261120/Pocket edited.obj",
                        1,
                        1,
                        0,
                    ),
                ):
                    obj = case / (label + ".obj")
                    shutil.copyfile(source, obj)
                    library = next(
                        (
                            row.split(maxsplit=1)[1]
                            for row in obj.read_text().splitlines()
                            if row.startswith("mtllib ")
                        ),
                        None,
                    )
                    if library is not None:
                        shutil.copyfile(source.with_name(library), case / library)
                    expected = case / (label + " cli.weftforge")
                    sdk = case / (label + " sdk.weftforge")
                    cli_output = json.loads(
                        run("import-obj", project, expected, obj, "1")
                    )
                    args = {
                        "input": arg_file(project),
                        "output": arg_file(sdk),
                        "obj": arg_file(obj),
                        "metres_per_unit": 1,
                    }
                    Draft202012Validator(schema).validate(args)
                    val = await session.call_tool("project_import_obj", args)
                    assert not val.isError, val
                    fields = val.structuredContent
                    for k, expected_value in (
                        ("pieces_imported", pieces),
                        ("holes_imported", holes),
                        ("materials_imported", materials),
                        ("metres_per_unit", 1),
                        ("revision", 1),
                    ):
                        assert fields[k] == cli_output[k] == expected_value, (
                            label,
                            k,
                            fields,
                            cli_output,
                        )
                    assert sdk.read_bytes() == expected.read_bytes()
                    embedded = {"obj": obj.read_text(), "metres_per_unit": 1}
                    if library:
                        embedded["mtl"] = (case / library).read_text()
                    command = {"type": "import_obj_patterns", "data": embedded}
                    batch = {
                        "input": arg_file(project),
                        "output": arg_file(sdk),
                        "commands": [command],
                    }
                    Draft202012Validator(command_schema).validate(command)
                    val = await session.call_tool("project_execute", batch)
                    assert not val.isError and sdk.read_bytes() == expected.read_bytes()
                    previous = sdk.with_name(sdk.name + ".recovery")
                    assert previous.read_bytes() == sdk.read_bytes()
                    bad = {**args, "metres_per_unit": -1}
                    assert not Draft202012Validator(schema).is_valid(bad)
                    refused = await session.call_tool("project_import_obj", bad)
                    assert (
                        refused.isError
                        and previous.read_bytes()
                        == sdk.read_bytes()
                        == expected.read_bytes()
                    )
                    report["cases"].append(
                        {
                            "fixture": source.name,
                            "obj_sha256": digest(obj),
                            "mtl_sha256": digest(case / library) if library else None,
                            "revision": 1,
                            "pieces": pieces,
                            "holes": holes,
                            "imported_materials": materials,
                            "saved_sha256": digest(sdk),
                            "cli_mcp_file_inline_identical": True,
                            "primary_and_recovery_preserved_on_rejection": True,
                        }
                    )
    assert not (case / "stderr.log").read_text()
    (case / "SDK.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    for name in ("binary", "root", "case"):
        p.add_argument(name, type=lambda x: Path(x).resolve())
    opts = p.parse_args()
    asyncio.run(measure(opts.binary, opts.root, opts.case))
