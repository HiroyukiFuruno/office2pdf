"""Contract tests for reproducible workspace dependency resolution."""

from __future__ import annotations

import re
import subprocess
import tomllib
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
LOCKFILE = PROJECT_ROOT / "Cargo.lock"
RESOLVING_COMMAND = re.compile(
    r"\b(?:cargo (?:build|check|clippy|metadata|test)|wasm-pack (?:build|test))\b"
)


class DependencyLockContractTest(unittest.TestCase):
    def test_workspace_lockfile_is_tracked_and_not_ignored(self) -> None:
        self.assertTrue(LOCKFILE.is_file(), "the workspace Cargo.lock must exist")

        tracked = subprocess.run(
            ["git", "ls-files", "--error-unmatch", "Cargo.lock"],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(tracked.returncode, 0, tracked.stderr)

        ignored = subprocess.run(
            ["git", "check-ignore", "--no-index", "--quiet", "Cargo.lock"],
            cwd=PROJECT_ROOT,
            check=False,
        )
        self.assertNotEqual(ignored.returncode, 0, "Cargo.lock must not be ignored")

    def test_workspace_resolves_the_graph_crates_io_users_build(self) -> None:
        # [patch] never reaches a published crate, so a patched workspace tests
        # code that library users and `cargo install` never build: 0.9.0's
        # crates.io build used upstream docx-rs and rejected minimal DOCX
        # packages the patched workspace passed, and zune-jpeg stayed patched
        # to an unreleased decoder after that (#2220).
        manifest = tomllib.loads((PROJECT_ROOT / "Cargo.toml").read_text())
        self.assertEqual(
            manifest.get("patch", {}),
            {},
            "depend on a published crate instead of a [patch] fork",
        )

        lock = tomllib.loads(LOCKFILE.read_text())
        for package in lock["package"]:
            # Workspace members carry no source.
            if "source" not in package:
                continue
            self.assertEqual(
                package["source"],
                "registry+https://github.com/rust-lang/crates.io-index",
                f"{package['name']} {package['version']} must resolve from crates.io",
            )

    def test_workflow_dependency_commands_refuse_lockfile_updates(self) -> None:
        for relative_path in (
            ".github/workflows/ci.yml",
            ".github/workflows/release.yml",
        ):
            for line_number, line in enumerate(
                (PROJECT_ROOT / relative_path).read_text().splitlines(), start=1
            ):
                command = line.strip()
                if command.startswith(("#", "echo ")):
                    continue
                if RESOLVING_COMMAND.search(command):
                    self.assertIn(
                        "--locked",
                        command,
                        f"{relative_path}:{line_number} can update Cargo.lock: {command}",
                    )

    def test_publish_verification_keeps_checking_the_consumer_graph(self) -> None:
        workflow = (PROJECT_ROOT / ".github/workflows/ci.yml").read_text()
        without_lock = workflow.index("rm Cargo.lock")
        publish = workflow.index("cargo publish --dry-run -p office2pdf --allow-dirty")
        self.assertLess(without_lock, publish)
        self.assertNotIn("cargo publish --locked --dry-run", workflow)

    def test_documented_visual_builds_use_the_reviewed_resolution(self) -> None:
        agent_rules = (PROJECT_ROOT / "AGENTS.md").read_text()
        business = (PROJECT_ROOT / "tests/golden_mocks/business/README.md").read_text()
        readme = (PROJECT_ROOT / "README.md").read_text()

        self.assertIn(
            "cargo test --locked -p office2pdf --test artifact_generator",
            agent_rules,
        )
        self.assertIn("cargo build --locked -p office2pdf-cli", business)
        wasm_builds = [
            line
            for line in readme.splitlines()
            if line.startswith("wasm-pack build crates/office2pdf")
        ]
        self.assertEqual(len(wasm_builds), 2)
        self.assertTrue(all("--locked" in line for line in wasm_builds))


class ForkPublishWorkflowTest(unittest.TestCase):
    """The forks office2pdf depends on reach crates.io only through this workflow.

    A `[patch.crates-io]` entry never applies to a published crate, so 0.9.0
    built from crates.io used upstream docx-rs and rejected the minimal DOCX
    packages its fork tolerates. The forks are published under their own names
    through this workflow; this pins how, because it holds the crates.io token.
    """

    WORKFLOW = PROJECT_ROOT / ".github/workflows/publish-fork.yml"
    FORKS = {
        "office2pdf-docx-rs": "developer0hye/docx-rs",
        "office2pdf-umya-spreadsheet": "developer0hye/umya-spreadsheet",
    }

    def setUp(self) -> None:
        self.text = self.WORKFLOW.read_text()

    def test_it_runs_only_on_manual_dispatch(self) -> None:
        trigger = self.text[self.text.index("\non:") : self.text.index("\npermissions:")]
        self.assertIn("workflow_dispatch:", trigger)
        for event in ("push:", "pull_request:", "schedule:", "workflow_run:"):
            self.assertNotIn(event, trigger)

    def test_it_publishes_only_the_fork_crates_from_their_own_repositories(self) -> None:
        options = re.search(r"options:\n((?:\s+- .+\n)+)", self.text)
        self.assertIsNotNone(options)
        self.assertEqual(
            sorted(re.findall(r"- (\S+)", options.group(1))), sorted(self.FORKS)
        )
        for package, repository in self.FORKS.items():
            self.assertRegex(
                self.text, rf"{re.escape(package)}\)\s+repository={re.escape(repository)}"
            )

    def test_it_publishes_an_exact_reviewed_commit(self) -> None:
        validation = self.text.index("^[0-9a-f]{40}$")
        checkout = self.text.index("uses: actions/checkout")
        self.assertLess(validation, checkout, "validate the ref before checking it out")
        self.assertIn('cargo +stable publish -p "${PACKAGE}"', self.text)
        self.assertNotIn("--allow-dirty", self.text)


if __name__ == "__main__":
    unittest.main()
