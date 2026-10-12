"""Corpus triage must include failed/missing inputs and never reuse stale PDFs."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from audit_docxide_corpus import discover_fixtures, scan_features, text_findings, convert_fixture

W = 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'


class CorpusAuditTest(unittest.TestCase):
    def package(self, path, body):
        with zipfile.ZipFile(path, 'w') as archive:
            archive.writestr('word/document.xml', f'<w:document xmlns:w="{W}"><w:body>{body}</w:body></w:document>')

    def test_discovery_does_not_drop_cases_without_reference_pdfs(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for case in ('case1', 'case2'):
                folder = root / 'tests' / 'fixtures' / 'cases' / case
                folder.mkdir(parents=True)
                self.package(folder / 'input.docx', '')
            (root / 'tests/fixtures/cases/case1/reference.pdf').touch()
            self.assertEqual(len(discover_fixtures(root)), 2)

    def test_feature_census_distinguishes_section_and_cell_alignment(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'source.docx'
            self.package(path, '<w:tbl><w:tr><w:tc><w:tcPr><w:vAlign w:val="center"/></w:tcPr></w:tc></w:tr></w:tbl><w:sectPr><w:pgBorders/><w:lnNumType/></w:sectPr><w:altChunk/>')
            features = scan_features(path)
            self.assertIn('page_borders', features)
            self.assertIn('line_numbering', features)
            self.assertIn('html_import', features)
            self.assertNotIn('section_vertical_alignment', features)

    def test_font_encoding_difference_is_not_missing_content(self):
        finding = text_findings('ofﬁce report', 'office report')
        self.assertTrue(finding['normalized_equal'])
        self.assertEqual(finding['missing_tokens'], [])

    def test_dropped_html_body_is_reported_even_when_output_pdf_exists(self):
        finding = text_findings('Import heading\nImportant terms\nFinal clause', '')
        self.assertFalse(finding['normalized_equal'])
        self.assertGreater(finding['missing_token_count'], 0)
        self.assertTrue(finding['empty_output_text'])

    def test_word_order_difference_is_reported_separately_from_loss(self):
        finding = text_findings('First table Body marker', 'Body marker First table')
        self.assertFalse(finding['normalized_equal'])
        self.assertEqual(finding['missing_token_count'], 0)
        self.assertEqual(finding['extra_token_count'], 0)

    def test_converter_failure_removes_stale_output_and_records_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / 'input.docx'
            self.package(source, '<w:p><w:r><w:t>Invoice</w:t></w:r></w:p>')
            output = root / 'audit'
            output.mkdir()
            (output / 'office2pdf.pdf').write_bytes(b'stale result')
            fake = root / 'converter'
            fake.write_text(f'#!{sys.executable}\nimport sys\nprint("unsupported package", file=sys.stderr)\nsys.exit(2)\n')
            fake.chmod(0o755)
            row = convert_fixture(source, output, fake, 2)
            self.assertEqual(row['status'], 'conversion_failed')
            self.assertEqual(row['exit_code'], 2)
            self.assertFalse((output / 'office2pdf.pdf').exists())
            self.assertIn('unsupported package', row['error'])

    def test_timeout_is_not_an_empty_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / 'input.docx'
            self.package(source, '')
            fake = root / 'converter'
            fake.write_text(f'#!{sys.executable}\nimport time\ntime.sleep(60)\n')
            fake.chmod(0o755)
            row = convert_fixture(source, root / 'audit', fake, 1)
            self.assertEqual(row['status'], 'conversion_timeout')


if __name__ == '__main__':
    unittest.main()
