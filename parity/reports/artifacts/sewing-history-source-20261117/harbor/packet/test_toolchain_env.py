"""Prospective pure environment check; unexecuted in source-only draft preparation."""
import ast
import os
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch


def require(condition, message):
    if not condition:
        raise ValueError(message)


class ToolchainEnvironment(unittest.TestCase):
    def test_pins_all_tools_and_rejects_mixed_environment(self):
        tree = ast.parse(Path(__file__).with_name("source_inputs.py").read_text())
        function = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "clean_child_env")
        namespace = {"os": os, "Path": Path, "r": SimpleNamespace(require=require),
                     "ambient_build_env": lambda: {k:v for k,v in sorted(os.environ.items()) if k.startswith(("CARGO_BUILD_","CARGO_PROFILE_","CARGO_TARGET_"))}}
        exec(compile(ast.Module(body=[function], type_ignores=[]), "<only clean_child_env>", "exec"), namespace)
        toolchain = "/synthetic/1.94.1-aarch64-apple-darwin/bin"
        cargo_home = str(Path("/synthetic/cargo-home").resolve())
        child = {"RUSTC":toolchain+"/rustc", "RUSTDOC":toolchain+"/rustdoc", "RUSTUP_TOOLCHAIN":"1.94.1",
                 "PATH":toolchain+":/usr/bin:/bin", "RUSTFLAGS":"", "CARGO_ENCODED_RUSTFLAGS":"", "RUSTDOCFLAGS":"",
                 "RUSTC_WRAPPER":"", "RUSTC_WORKSPACE_WRAPPER":""}
        binding = {"parent_path":"/usr/bin:/bin", "cargo_home":cargo_home, "ambient_build_env":{}, "child_env":child}
        with patch.dict(os.environ, {"PATH":"/usr/bin:/bin","CARGO_HOME":cargo_home}, clear=True):
            result = namespace["clean_child_env"](binding)
            for key in child:
                self.assertEqual(result[key], child[key])
            for key, value in [("RUSTC","/mixed/rustc"),("RUSTDOC","/mixed/rustdoc"),("RUSTUP_TOOLCHAIN","stable"),
                               ("RUSTC_WRAPPER","/mixed/wrapper"),("RUSTC_WORKSPACE_WRAPPER","/mixed/wrapper"),
                               ("PATH","/mixed/bin:/usr/bin:/bin")]:
                with self.subTest(key=key), patch.dict(os.environ, {key:value}):
                    with self.assertRaises(ValueError):
                        namespace["clean_child_env"](binding)


if __name__ == "__main__":
    unittest.main()
