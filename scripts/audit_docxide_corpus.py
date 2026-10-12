#!/usr/bin/env python3
"""Convert and triage the pinned docxide-pdf corpus. Counts do not prove visual parity.

Corpus checkout: https://github.com/sverrejb/docxide-pdf at SOURCE_REVISION.
No fixtures or fonts are downloaded, copied into Git, or executed by this script.
"""
from __future__ import annotations

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time
import xml.etree.ElementTree as ET
import zipfile

from compare_text_layer import normalize

SOURCE_REVISION = '1bb140b662a69f8786015c1b532d37d6084d0934'
SOURCE_URL = 'https://github.com/sverrejb/docxide-pdf'
W = '{http://schemas.openxmlformats.org/wordprocessingml/2006/main}'
FEATURES = {'altChunk': 'html_import', 'customXml': 'custom_xml', 'pgBorders': 'page_borders',
            'lnNumType': 'line_numbering', 'framePr': 'text_frames', 'sym': 'symbol_runs',
            'lvlRestart': 'list_restart', 'lvlOverride': 'list_override', 'isLgl': 'legal_numbering'}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def discover_fixtures(root: Path) -> list[Path]:
    # Include missing references and upstream skipped cases: neither is a pass.
    return sorted(root.glob('tests/fixtures/*/*/input.docx'))


def scan_features(path: Path) -> list[str]:
    features: set[str] = set()
    with zipfile.ZipFile(path) as archive:
        for name in archive.namelist():
            if not name.startswith('word/') or not name.endswith('.xml'):
                continue
            document = ET.fromstring(archive.read(name))
            for node in document.iter():
                if node.tag.startswith(W) and node.tag[len(W):] in FEATURES:
                    features.add(FEATURES[node.tag[len(W):]])
                if node.tag == W + 'sectPr' and node.find(W + 'vAlign') is not None:
                    features.add('section_vertical_alignment')
    return sorted(features)


def text_findings(reference: str, current: str) -> dict:
    reference_normalized, current_normalized = normalize(reference), normalize(current)
    reference_tokens = Counter(re.findall(r'\w+', reference_normalized))
    current_tokens = Counter(re.findall(r'\w+', current_normalized))
    missing, extra = reference_tokens - current_tokens, current_tokens - reference_tokens
    # Token hashes identify repeat findings without publishing source-document text.
    token_ids = lambda counts: [hashlib.sha256(token.encode()).hexdigest()[:16] for token in sorted(counts)]
    return {'normalized_equal': reference_normalized == current_normalized,
            'reference_characters': len(reference_normalized), 'output_characters': len(current_normalized),
            'missing_token_count': sum(missing.values()), 'extra_token_count': sum(extra.values()),
            'missing_tokens': token_ids(missing), 'extra_tokens': token_ids(extra),
            'empty_output_text': bool(reference_normalized) and not current_normalized}


def pdf_details(path: Path) -> dict:
    information = subprocess.run(['pdfinfo', str(path)], capture_output=True, text=True, check=True).stdout
    fields = dict(line.split(':', 1) for line in information.splitlines() if ':' in line)
    text = subprocess.run(['pdftotext', str(path), '-'], capture_output=True, text=True, check=True).stdout
    return {'pages': int(fields['Pages'].strip()), 'creator': fields.get('Creator', '').strip(),
            'producer': fields.get('Producer', '').strip(), 'text': text}


def convert_fixture(source: Path, output: Path, converter: Path, timeout: int) -> dict:
    output.mkdir(parents=True, exist_ok=True)
    pdf = output / 'office2pdf.pdf'
    pdf.unlink(missing_ok=True)
    started = time.monotonic()
    row: dict = {'source_sha256': sha256(source)}
    try:
        row['features'] = scan_features(source)
        process = subprocess.Popen([str(converter), str(source), '-o', str(pdf)],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   text=True, start_new_session=True)
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.communicate()
            row.update(status='conversion_timeout', exit_code=124, error='conversion timeout')
            return row
        (output / 'converter.log').write_text(stdout + stderr, encoding='utf-8')
        row['exit_code'] = process.returncode
        if process.returncode != 0 or not pdf.exists():
            # Keep diagnostics locally, but do not publish document or machine paths.
            row.update(status='conversion_failed', error=stderr.replace(str(source), 'input.docx')[-1000:])
            return row
        row['output_sha256'] = sha256(pdf)
        current = pdf_details(pdf)
        row['output_pages'] = current['pages']
        reference_path = source.with_name('reference.pdf')
        if not reference_path.exists():
            row.update(status='missing_reference')
            return row
        reference = pdf_details(reference_path)
        row.update(reference_sha256=sha256(reference_path), reference_pages=reference['pages'],
                   reference_creator=reference['creator'], reference_producer=reference['producer'],
                   page_count_equal=reference['pages'] == current['pages'],
                   text=text_findings(reference['text'], current['text']), status='compared')
    except (OSError, ValueError, KeyError, ET.ParseError, zipfile.BadZipFile, subprocess.SubprocessError) as error:
        row.update(status='audit_failed', error=type(error).__name__)
    finally:
        row['elapsed_seconds'] = round(time.monotonic() - started, 3)
    return row


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--corpus-root', type=Path, required=True)
    parser.add_argument('--converter', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--case', help='one group/case; default audits all inputs including skipped cases')
    parser.add_argument('--jobs', type=int, default=4)
    parser.add_argument('--timeout', type=int, default=120)
    args = parser.parse_args()
    if args.jobs < 1 or args.timeout < 1:
        parser.error('--jobs and --timeout must be positive')
    root, converter, output = args.corpus_root.resolve(), args.converter.resolve(), args.output.resolve()
    revision = subprocess.run(['git', '-C', str(root), 'rev-parse', 'HEAD'], check=True,
                              capture_output=True, text=True).stdout.strip()
    if revision != SOURCE_REVISION:
        parser.error(f'expected corpus commit {SOURCE_REVISION}, found {revision}')
    fixtures = discover_fixtures(root)
    if args.case:
        fixtures = [path for path in fixtures if str(path.parent.relative_to(root / 'tests/fixtures')) == args.case]
    if not fixtures:
        parser.error('no matching input.docx fixtures')
    output.mkdir(parents=True, exist_ok=True)
    provenance = {'repository': SOURCE_URL, 'revision': revision, 'converter_sha256': sha256(converter),
                  'converter_version': subprocess.run([str(converter), '--version'], capture_output=True,
                                                      text=True, check=True).stdout.strip()}
    rows: list[dict] = []
    def convert(source: Path) -> dict:
        case = str(source.parent.relative_to(root / 'tests/fixtures'))
        row = convert_fixture(source, output / case, converter, args.timeout)
        row['case'] = case
        return row
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        for row in pool.map(convert, fixtures):
            rows.append(row)
            if len(rows) % 25 == 0 or len(rows) == len(fixtures):
                print(f'{len(rows)}/{len(fixtures)} audited', flush=True)
    statuses = Counter(row['status'] for row in rows)
    report = {'schema_version': 1, 'provenance': provenance,
              'scope': 'triage only: font availability, geometry, pixels, and model vision remain unaudited',
              'summary': {'inputs': len(rows), 'statuses': dict(statuses),
                          'page_count_differences': sum(row.get('page_count_equal') is False for row in rows),
                          'empty_text_outputs': sum(row.get('text', {}).get('empty_output_text', False) for row in rows)},
              'cases': rows}
    (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report['summary']))
    # Conversion/audit failures and absent references must not look like a clean run.
    return 0 if statuses['compared'] == len(rows) else 1


if __name__ == '__main__':
    raise SystemExit(main())
