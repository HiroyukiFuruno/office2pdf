# LibreOffice VCL font substitution table

- **URL:** https://github.com/LibreOffice/core/blob/a11496e66e08c820df21e7985ba79e6a9bd6d7f0/officecfg/registry/data/org/openoffice/VCL.xcu
- **Inspected revision:** `a11496e66e08c820df21e7985ba79e6a9bd6d7f0` (2026-10-16)
- **Language:** XML configuration (read by LibreOffice's C++ VCL layer)
- **License:** MPL-2.0

## What it does

`VCL.xcu` holds LibreOffice's `FontSubstitutions` table. Each lowercased
family node lists `SubstFonts`, the faces tried in order when the family is
missing, plus the family's weight, width and type classes.

## Why relevant

- It is a maintained, cross-platform answer to "what replaces this missing
  Office face", independent of fontconfig.
- Tahoma lists `andalesansui;arialunicodems;lucidaunicode;albanyamt;albany;arial;…`.
  These are Arial-class faces. office2pdf follows that class with Liberation
  Sans, which the issue #2235 width measurement also favours over DejaVu Sans
  (2.9% against 13.2%).
- Verdana lists `segoe;lucidasans;albanyamt;albany;arial;…`. Its table entry
  here keeps DejaVu Sans instead, which is Verdana-like in width.

## When to consult

- Choosing a `table_entry` substitute in `render/font_subst.rs` for a face
  the table does not list yet.
- Checking whether a family's class (sans, serif, symbol) is settled
  upstream before relying on name heuristics.

Measure candidate widths against a Word export before adopting an entry.
LibreOffice's order reflects font availability on its platforms, not advance
compatibility.
