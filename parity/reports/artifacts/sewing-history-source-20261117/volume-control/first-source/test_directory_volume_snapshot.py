"""Prospective synthetic tests only. Not executed during source-only draft preparation."""
import tempfile
from pathlib import Path
from types import SimpleNamespace
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent / "packet"))
import resources as r


class DirectoryVolumeSnapshot(unittest.TestCase):
    def test_temporary_files_and_existing_shallow_scope_count(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ".stage-060.obj.1-1.tmp").write_bytes(b"x" * 11)
            (root / "log").write_bytes(b"abc")
            (root / "nested").mkdir()
            (root / "nested/file").write_bytes(b"abcde")
            self.assertEqual(r.directory_bytes([root, root / "not-created"]), 19)
            self.assertEqual(r.directory_bytes([], shallow=[root]), 14)

    def test_rename_before_stat_restarts_the_snapshot(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            staged, final = root / ".stage.tmp", root / "stage.obj"
            staged.write_bytes(b"x" * 12)
            original, roots = Path.lstat, []

            def renamed(path):
                if path == root:
                    roots.append(path)
                if path == staged:
                    staged.rename(final)
                return original(path)

            with patch.object(Path, "lstat", renamed):
                self.assertEqual(r.directory_bytes([root]), 12)
            self.assertEqual(len(roots), 2)

    def test_rename_after_stat_retains_the_observed_size(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            staged, final = root / ".stage.tmp", root / "stage.obj"
            staged.write_bytes(b"x" * 12)
            original, samples = Path.lstat, []

            def renamed(path):
                entry = original(path)
                if path == staged:
                    samples.append(path)
                    staged.rename(final)
                return entry

            with patch.object(Path, "lstat", renamed):
                self.assertEqual(r.directory_bytes([root]), 12)
            self.assertEqual(len(samples), 1)

    def test_partial_observed_peak_is_conservative_after_retry(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            retained, staged, final = root / "a", root / "z.tmp", root / "z"
            retained.write_bytes(b"x" * 10)
            staged.write_bytes(b"x")
            original = Path.lstat

            def renamed(path):
                if path == staged:
                    staged.rename(final)
                    retained.unlink()
                return original(path)

            with patch.object(Path, "lstat", renamed):
                self.assertEqual(r.directory_bytes([root]), 10)
            self.assertEqual(r.directory_bytes([root]), 1)

    def test_persistent_disappearance_fails_after_exactly_three_attempts(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, calls = Path(temporary), []
            missing = root / "vanishing"
            missing.write_bytes(b"x")
            original = Path.lstat

            def vanished(path):
                if path == missing:
                    calls.append(path)
                    raise FileNotFoundError("synthetic disappearance")
                return original(path)

            with patch.object(Path, "lstat", vanished):
                with self.assertRaisesRegex(FileNotFoundError, "3 snapshot attempts"):
                    r.directory_bytes([root])
            self.assertEqual(len(calls), 3)

    def test_permission_and_symlink_errors_are_not_retried(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, calls = Path(temporary), []
            denied = root / "denied"
            denied.write_bytes(b"x")
            original = Path.lstat

            def forbidden(path):
                if path == denied:
                    calls.append(path)
                    raise PermissionError("synthetic permission failure")
                return original(path)

            with patch.object(Path, "lstat", forbidden):
                with self.assertRaises(PermissionError):
                    r.directory_bytes([root])
            self.assertEqual(len(calls), 1)
            (root / "link").symlink_to(denied)
            with self.assertRaisesRegex(ValueError, "Symlink volume entry"):
                r.directory_bytes([root])

    def test_shared_deadline_stops_before_another_snapshot(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, calls, expired = Path(temporary), [], [False]
            missing = root / "vanishing"
            missing.write_bytes(b"x")
            original = Path.lstat

            def vanished(path):
                if path == missing:
                    calls.append(path)
                    expired[0] = True
                    raise FileNotFoundError("synthetic disappearance")
                return original(path)

            def deadline(_):
                if expired[0]:
                    raise ValueError("synthetic deadline closed")

            with patch.object(Path, "lstat", vanished), patch.object(r, "deadline_check", deadline):
                with self.assertRaisesRegex(ValueError, "deadline closed"):
                    r.directory_bytes([root], deadline=1)
            self.assertEqual(len(calls), 1)

    def test_strict_artifact_enumeration_still_refuses_disappearance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, calls = Path(temporary), []
            missing = root / "vanishing"
            missing.write_bytes(b"x")
            original = Path.lstat

            def vanished(path):
                if path == missing:
                    calls.append(path)
                    raise FileNotFoundError("synthetic disappearance")
                return original(path)

            with patch.object(Path, "lstat", vanished):
                with self.assertRaises(FileNotFoundError):
                    r.files_below(root)
            self.assertEqual(len(calls), 1)

    def test_terminal_count_failure_returns_explicit_negative_after_cleanup(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            child = SimpleNamespace(pid=43210, returncode=None)
            usage = SimpleNamespace(ru_maxrss=0)
            count = Mock(side_effect=[0, FileNotFoundError("synthetic persistent volume failure")])
            with patch.object(r.subprocess, "Popen", return_value=child), patch.object(r.os, "wait4", return_value=(child.pid, 0, usage)), patch.object(r, "kill_group") as kill:
                result = r.monitor(["synthetic"], root, root / "stdout", root / "stderr",
                                   r.time.monotonic() + 60, count, reserve=0)
            kill.assert_called_once_with(child.pid)
            self.assertEqual(result["stop_reason"], "terminal_output_observation_unavailable")
            self.assertIsNone(result["terminal_phase_checks"]["output_bytes"])
            self.assertIn("FileNotFoundError", result["output_observation_error"])


if __name__ == "__main__":
    unittest.main()
