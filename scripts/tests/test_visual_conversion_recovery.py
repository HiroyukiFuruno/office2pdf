import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from scripts import check_visual_pr as contract
from scripts.tests.test_check_visual_pr import ROOT, visual_body


def recovery_body():
    return (
        visual_body()
        .replace('Evidence mode: `fix`', 'Evidence mode: `conversion-recovery`')
        .replace('- Before: `assets/bugfixes/issue-186/before.jpg`',
                 '- Before: None\n- Baseline failure report: `assets/bugfixes/issue-186/conversion-failure.json`')
        .replace('tests/fixtures/xlsx/pr_186_contributor_acceptance.xlsx',
                 'tests/fixtures/docx/report.docx')
        .replace('![Before](https://example.com/before.jpg)', '')
    )


class ConversionRecoveryContractTests(unittest.TestCase):
    def prepare(self, root):
        fixture = root / 'tests/fixtures/docx/report.docx'
        fixture.parent.mkdir(parents=True)
        shutil.copy2(ROOT / 'tests/fixtures/docx/docxide/truncated_png_report.docx', fixture)
        directory = root / 'assets/bugfixes/issue-186'
        directory.mkdir(parents=True)
        for name in ['gt.jpg', 'after.jpg']:
            shutil.copy2(ROOT / f'assets/bugfixes/issue-2106/{name}', directory / name)
        diagnostic = directory / 'conversion-failure.txt'
        diagnostic.write_text('Error: Typst compilation failed: failed to decode image (unexpected end of file)\n')
        receipt = {
            'schema_version': 1,
            'status': 'conversion_failed',
            'fixture': 'tests/fixtures/docx/report.docx',
            'source_sha256': hashlib.sha256(fixture.read_bytes()).hexdigest(),
            'baseline_revision': 'b605a5813b0adafadb49587f554a0b2a8d301859',
            'converter_sha256': '1' * 64,
            'exit_code': 1,
            'output_pdf_generated': False,
            'diagnostic_path': 'assets/bugfixes/issue-186/conversion-failure.txt',
            'diagnostic_sha256': hashlib.sha256(diagnostic.read_bytes()).hexdigest(),
        }
        path = directory / 'conversion-failure.json'
        path.write_text(json.dumps(receipt))
        return path, receipt

    def test_recovery_has_native_and_current_previews_without_fabricated_before(self):
        changed = ['assets/bugfixes/issue-186/after.jpg']
        self.assertEqual(contract.validate_pr_body(recovery_body(), changed), [])
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.prepare(root)
            self.assertEqual(contract.validate_evidence(changed, root, mode='conversion-recovery'), [])

    def test_real_failure_receipt_binds_source_and_diagnostic(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path, _ = self.prepare(root)
            self.assertEqual(contract.validate_conversion_failure(recovery_body(), [path.relative_to(root).as_posix()], root), [])

    def test_successful_baseline_cannot_omit_before_image(self):
        for key, value in [('exit_code', 0), ('exit_code', False), ('output_pdf_generated', True), ('status', 'compared')]:
            with self.subTest(key=key, value=value), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                path, receipt = self.prepare(root)
                receipt[key] = value
                path.write_text(json.dumps(receipt))
                self.assertTrue(contract.validate_conversion_failure(recovery_body(), [path.relative_to(root).as_posix()], root))

    def test_changed_source_and_changed_log_invalidate_baseline(self):
        for target in ['tests/fixtures/docx/report.docx', 'assets/bugfixes/issue-186/conversion-failure.txt']:
            with self.subTest(target=target), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                path, _ = self.prepare(root)
                with (root / target).open('ab') as stream:
                    stream.write(b'changed')
                self.assertTrue(contract.validate_conversion_failure(recovery_body(), [path.relative_to(root).as_posix()], root))

    def test_recovery_does_not_exempt_current_layout_or_cluster_reports(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.assertTrue(contract.validate_layout_audit(recovery_body(), ['assets/bugfixes/issue-186/after.jpg'], root))
            self.assertTrue(contract.validate_render_cluster_audits(recovery_body(), ['assets/bugfixes/issue-186/after.jpg'], root))

    def test_existing_fix_mode_still_requires_before(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.prepare(root)
            self.assertTrue(contract.validate_evidence(['assets/bugfixes/issue-186/after.jpg'], root))

    def test_recovery_rejects_fabricated_before_and_missing_current_images(self):
        for name in ['gt.jpg', 'after.jpg', 'before.jpg']:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                self.prepare(root)
                directory = root / 'assets/bugfixes/issue-186'
                if name == 'before.jpg':
                    shutil.copy2(directory / 'after.jpg', directory / name)
                else:
                    (directory / name).unlink()
                self.assertTrue(contract.validate_evidence(
                    ['assets/bugfixes/issue-186/after.jpg'], root,
                    mode='conversion-recovery'))

    def test_invalid_receipt_metadata_and_fixture_paths_are_rejected(self):
        for key, value in [
            ('schema_version', True), ('schema_version', 2),
            ('baseline_revision', 'main'), ('converter_sha256', 'unknown'),
            ('source_sha256', '0' * 64), ('diagnostic_sha256', '0' * 64),
            ('diagnostic_path', 'other/conversion-failure.txt'),
            ('fixture', '../outside.docx'), ('fixture', '/outside.docx'),
        ]:
            with self.subTest(key=key, value=value), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                path, receipt = self.prepare(root)
                receipt[key] = value
                path.write_text(json.dumps(receipt))
                body = recovery_body()
                if key == 'fixture':
                    body = body.replace('tests/fixtures/docx/report.docx', value)
                self.assertTrue(contract.validate_conversion_failure(
                    body, [path.relative_to(root).as_posix()], root))

    def test_missing_malformed_and_unchanged_failure_receipts_are_rejected(self):
        for contents in [None, 'not JSON', '[]', '{}']:
            with self.subTest(contents=contents), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                path, _ = self.prepare(root)
                if contents is None:
                    path.unlink()
                else:
                    path.write_text(contents)
                self.assertTrue(contract.validate_conversion_failure(
                    recovery_body(), [path.relative_to(root).as_posix()], root))
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.prepare(root)
            self.assertTrue(contract.validate_conversion_failure(recovery_body(), [], root))

    def test_failure_mode_cannot_be_used_for_text_layer_only_or_normal_fix(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path, _ = self.prepare(root)
            changed = [path.relative_to(root).as_posix()]
            self.assertTrue(contract.validate_conversion_failure(
                visual_body(), changed, root))
            self.assertTrue(contract.validate_conversion_failure(
                recovery_body() + '\n- Text-layer-only: Yes\n', changed, root))


if __name__ == '__main__':
    unittest.main()
