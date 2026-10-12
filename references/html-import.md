# HTML/MHTML import libraries

- [Servo html5ever](https://github.com/servo/html5ever), MIT/Apache-2.0:
  WHATWG HTML parser. The RcDom example informs local read-only tree traversal.
  Real DOCX imports contain omitted closing tags and HTML entities, so an XML
  parser with patched void tags would reject valid content.
- [mailparse](https://github.com/staktrace/mailparse), 0BSD: MIME parsing,
  folded headers, quoted-printable/base64 transfer decoding, and charsets.
  MHTML embeds HTML in MIME parts; splitting boundary strings is insufficient.

`docx_html_import.rs` maps a bounded HTML/CSS subset to WordprocessingML.
`docx_alt_chunk.rs` expands internal import anchors before contextual scans and
DOCX parsing, preserving cursor alignment. Neither library executes scripts or
fetches resources. Unsupported images/row spans retain surrounding text with
warnings; external and unsupported-format imports are skipped with warnings.

The mixed literal/quoted UTF-8 workaround preserves raw 8-bit bytes before MIME
decoding. An opt-in upstream API is proposed in
[quoted-printable PR 30](https://github.com/staktrace/quoted-printable/pull/30);
the importer does not require an unpublished dependency revision.
