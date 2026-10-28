"""Exercise releases against disposable local repositories, without publishing."""

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


class ReleaseTest(unittest.TestCase):
    def git(self, *args, cwd=None, check=True):
        return subprocess.run(
            ["git", *args], cwd=cwd or self.repo,
            text=True, capture_output=True, check=check,
        ).stdout.strip()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.repo = root / "repo"
        self.remote = root / "remote.git"
        self.repo.mkdir()
        self.git("init", "--bare", str(self.remote))
        self.git("init", "-b", "main")
        self.git("config", "user.name", "Release Test")
        self.git("config", "user.email", "release@example.invalid")
        self.git("config", "commit.gpgsign", "false")
        self.git("config", "tag.gpgsign", "false")
        (self.repo / "scripts").mkdir()
        shutil.copyfile(Path(__file__).with_name("release.sh"), self.repo / "scripts/release.sh")
        self.manifest = '[package]\nname = "linux_virtual_joystick"\nversion = "0.1.4"\n'
        self.lock = (
            'version = 3\n\n[[package]]\nname = "dependency"\nversion = "0.1.4"\n'
            '\n[[package]]\nname = "linux_virtual_joystick"\nversion = "0.1.4"\n'
        )
        (self.repo / "Cargo.toml").write_text(self.manifest)
        (self.repo / "Cargo.lock").write_text(self.lock)
        self.git("add", ".")
        self.git("commit", "-m", "Initial")
        self.git("tag", "2.0.0")
        self.git("remote", "add", "origin", str(self.remote))
        self.git("push", "-u", "origin", "main", "--tags")
        self.original_head = self.git("rev-parse", "HEAD")

    def release(self, version="2.0.1"):
        return subprocess.run(
            ["bash", "scripts/release.sh"], cwd=self.repo,
            input=version + "\n", text=True, capture_output=True,
        )

    def test_release_updates_only_app_version_and_pushes_commit_and_tag(self):
        result = self.release()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Current app version: 0.1.4", result.stdout)
        self.assertIn("Latest release tag: 2.0.0", result.stdout)
        self.assertEqual((self.repo / "Cargo.toml").read_text(), self.manifest.replace("0.1.4", "2.0.1"))
        self.assertEqual((self.repo / "Cargo.lock").read_text(), self.lock.replace(
            'name = "linux_virtual_joystick"\nversion = "0.1.4"',
            'name = "linux_virtual_joystick"\nversion = "2.0.1"',
        ))
        self.assertEqual(self.git("status", "--porcelain"), "")
        self.assertEqual(self.git("log", "-1", "--format=%s"), "Release 2.0.1")
        self.assertEqual(self.git("cat-file", "-t", "2.0.1"), "tag")
        head = self.git("rev-parse", "HEAD")
        self.assertEqual(self.git("rev-parse", "refs/heads/main", cwd=self.remote), head)
        self.assertEqual(self.git("rev-parse", "refs/tags/2.0.1^{}", cwd=self.remote), head)

    def test_dirty_tree_is_preserved(self):
        note = self.repo / "work-in-progress.txt"
        note.write_text("Uncommitted work\n")
        result = self.release()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Commit or stash", result.stderr)
        self.assertEqual(note.read_text(), "Uncommitted work\n")
        self.assertEqual(self.git("rev-parse", "HEAD"), self.original_head)

    def test_invalid_old_and_reused_versions_do_not_change_files(self):
        for version in ["bad", "2.01.0", "2.1.0-rc.01", "0.1.5", "2.0.0", "2.0.0+build"]:
            with self.subTest(version=version):
                result = self.release(version)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.git("status", "--porcelain"), "")
                self.assertEqual(self.git("rev-parse", "HEAD"), self.original_head)

    def test_prerelease_version_is_accepted(self):
        result = self.release("2.1.0-rc.1")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('version = "2.1.0-rc.1"', (self.repo / "Cargo.toml").read_text())

    def test_stable_release_sorts_after_prerelease(self):
        self.git("tag", "v2.1.0-rc.9")
        result = self.release("2.1.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Latest release tag: v2.1.0-rc.9", result.stdout)

    def test_prerelease_numbers_sort_numerically(self):
        self.git("tag", "2.1.0-rc.9")
        result = self.release("2.1.0-rc.10")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_commit_failure_restores_version_files(self):
        hook = self.repo / ".git/hooks/pre-commit"
        hook.write_text("#!/bin/sh\nexit 1\n")
        hook.chmod(0o755)
        result = self.release()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.git("status", "--porcelain"), "")
        self.assertEqual((self.repo / "Cargo.toml").read_text(), self.manifest)
        self.assertEqual((self.repo / "Cargo.lock").read_text(), self.lock)

    def test_rejected_tag_push_keeps_remote_branch_unchanged(self):
        hook = self.remote / "hooks/update"
        hook.write_text('#!/bin/sh\ncase "$1" in refs/tags/*) exit 1;; esac\n')
        hook.chmod(0o755)
        result = self.release()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("preserved locally", result.stderr)
        self.assertEqual(self.git("rev-parse", "refs/heads/main", cwd=self.remote), self.original_head)
        self.assertEqual(self.git("tag", "--list", "2.0.1", cwd=self.remote), "")
        self.assertEqual(self.git("tag", "--list", "2.0.1"), "2.0.1")

    def test_branch_behind_remote_is_rejected(self):
        self.git("commit", "--allow-empty", "-m", "Remote update")
        self.git("push", "origin", "main")
        self.git("reset", "--hard", self.original_head)
        result = self.release()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("behind or diverges", result.stderr)
        self.assertEqual(self.git("status", "--porcelain"), "")


if __name__ == "__main__":
    unittest.main()
