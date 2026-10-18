//! Extract and deobfuscate embedded fonts from PPTX/DOCX archives.
//!
//! OOXML files can embed fonts as obfuscated binary data. Archive parsing and
//! deobfuscation stay in memory on every target. Native conversion materializes
//! the faces in a temporary directory for Typst's font discovery, while WASM passes the
//! same bytes directly to Typst.

use std::io::Read;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::parser::xml_util::get_attr_str;

#[cfg(test)]
#[path = "embedded_fonts_tests.rs"]
mod tests;

// =============================================================================
// Types
// =============================================================================

/// Font file format detected by magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FontFileFormat {
    Ttf,
    Otf,
    Ttc,
}

impl FontFileFormat {
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    fn extension(self) -> &'static str {
        match self {
            Self::Ttf => "ttf",
            Self::Otf => "otf",
            Self::Ttc => "ttc",
        }
    }
}

/// A PPTX embedded font entry parsed from `presentation.xml`.
#[derive(Debug)]
struct PptxEmbeddedFontEntry {
    typeface: String,
    variants: Vec<FontVariantRef>,
}

/// A reference to a single font variant (regular, bold, italic, boldItalic).
#[derive(Debug)]
struct FontVariantRef {
    style: String,
    r_id: String,
}

/// A DOCX embedded font entry parsed from `word/fontTable.xml`.
#[derive(Debug)]
struct DocxEmbeddedFontEntry {
    font_name: String,
    variants: Vec<DocxFontVariantRef>,
}

/// A DOCX font variant reference including the fontKey for deobfuscation.
#[derive(Debug)]
struct DocxFontVariantRef {
    style: String,
    r_id: String,
    font_key: String,
}

/// One deobfuscated face carried by an OOXML document.
///
/// The document's declared family name is deliberately not kept: it is
/// untrusted text, and the face's own `name` table is what font discovery
/// reads.
#[derive(Debug)]
struct EmbeddedFontFace {
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    format: FontFileFormat,
    data: Vec<u8>,
}

/// Deobfuscated document font faces, kept in memory for target-independent use.
#[derive(Debug)]
pub(crate) struct EmbeddedFontData {
    faces: Vec<EmbeddedFontFace>,
}

impl EmbeddedFontData {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) fn font_bytes(&self) -> impl Iterator<Item = &[u8]> {
        self.faces.iter().map(|face| face.data.as_slice())
    }
}

/// Temporary directory containing extracted font files.
/// Cleaned up automatically when dropped.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct EmbeddedFontDir {
    path: PathBuf,
    font_count: usize,
}

#[cfg(not(target_arch = "wasm32"))]
impl EmbeddedFontDir {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.font_count == 0
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for EmbeddedFontDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Extract and deobfuscate embedded fonts from an OOXML archive in memory.
///
/// Returns `None` if:
/// - The format doesn't support embedded fonts (XLSX)
/// - No embedded fonts are declared in the document
/// - The ZIP cannot be opened
/// - Extraction fails silently (best-effort)
pub(crate) fn extract_embedded_font_data(
    data: &[u8],
    format: crate::config::Format,
) -> Option<EmbeddedFontData> {
    use crate::config::Format;

    let result = match format {
        Format::Pptx => extract_pptx_font_data(data),
        Format::Docx => extract_docx_font_data(data),
        Format::Xlsx => None,
    };

    if let Some(ref fonts) = result {
        tracing::info!(
            font_count = fonts.faces.len(),
            "extracted embedded font data from archive"
        );
    }

    result
}

/// Extract embedded fonts and materialize them for native font discovery.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn extract_embedded_fonts(
    data: &[u8],
    format: crate::config::Format,
) -> Option<EmbeddedFontDir> {
    let fonts = extract_embedded_font_data(data, format)?;
    let prefix = match format {
        crate::config::Format::Pptx => "office2pdf-pptx-fonts",
        crate::config::Format::Docx => "office2pdf-docx-fonts",
        crate::config::Format::Xlsx => return None,
    };
    let temp_dir = create_temp_font_dir(prefix)?;
    let mut font_count = 0;

    for (index, face) in fonts.faces.into_iter().enumerate() {
        // Name files by position, never by document text: a declared family
        // such as `../x` or an absolute path would otherwise steer the write
        // outside the directory (#2153).
        let path = temp_dir.join(format!("font-{index:04}.{}", face.format.extension()));
        if write_new_file(&path, &face.data).is_ok() {
            font_count += 1;
        }
    }

    if font_count == 0 {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return None;
    }

    tracing::info!(
        font_count,
        path = ?temp_dir,
        "materialized embedded fonts from archive"
    );

    Some(EmbeddedFontDir {
        path: temp_dir,
        font_count,
    })
}

// =============================================================================
// GUID parsing
// =============================================================================

/// Parse a font-obfuscation GUID into its 16-byte XOR key.
///
/// Input formats: `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` or without braces.
/// ECMA-376 Part 1 §17.8.1 reads the 32 hex digits as bytes in string order
/// and reverses them, as LibreOffice's `EmbeddedFontHandler` does. The
/// mixed-endian in-memory GUID layout, reversed or not, corrupts the sfnt
/// header or the first table record (issue #2200).
fn parse_guid_to_bytes(guid: &str) -> Option<[u8; 16]> {
    let s = guid.trim_start_matches('{').trim_end_matches('}');
    let groups: Vec<&str> = s.split('-').collect();
    let group_lengths = groups.iter().map(|group| group.len());
    if !group_lengths.eq([8, 4, 4, 4, 12]) {
        return None;
    }
    let hex = groups.concat();
    // `from_str_radix` would also accept a sign, and slicing needs ASCII.
    if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }

    let mut key = [0u8; 16];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * index..2 * index + 2], 16).ok()?;
    }
    key.reverse();
    Some(key)
}

// =============================================================================
// Deobfuscation
// =============================================================================

/// XOR the first 32 bytes of font data with the 16-byte key (repeated twice).
fn deobfuscate_font_data(data: &mut [u8], key: &[u8; 16]) {
    let len = std::cmp::min(32, data.len());
    for i in 0..len {
        data[i] ^= key[i % 16];
    }
}

/// Detect font format from magic bytes at the start of the data.
fn detect_font_format(data: &[u8]) -> Option<FontFileFormat> {
    if data.len() < 4 {
        return None;
    }
    if data[0..4] == [0x00, 0x01, 0x00, 0x00] {
        Some(FontFileFormat::Ttf)
    } else if &data[0..4] == b"OTTO" {
        Some(FontFileFormat::Otf)
    } else if &data[0..4] == b"ttcf" {
        Some(FontFileFormat::Ttc)
    } else {
        None
    }
}

/// Retain deobfuscated bytes, detecting their format from the magic.
///
/// Bytes without sfnt magic are still kept as TrueType, since a parser may
/// recover what detection does not recognise, but a missing magic usually
/// means the key or the part is wrong and the face will not load.
fn deobfuscated_face(data: Vec<u8>, family: &str, style: &str) -> EmbeddedFontFace {
    // The family is the document's own text; `?` escapes it in both log lines.
    let format = match detect_font_format(&data) {
        Some(format) => format,
        None => {
            tracing::warn!(
                family = ?family,
                style,
                "deobfuscated embedded font has no sfnt magic; the face may not load"
            );
            FontFileFormat::Ttf
        }
    };
    tracing::debug!(family = ?family, style, ?format, "deobfuscated embedded font face");
    EmbeddedFontFace { format, data }
}

// =============================================================================
// PPTX extraction
// =============================================================================

/// Parse `<p:embeddedFontLst>` from `presentation.xml`.
fn parse_pptx_embedded_font_list(xml: &str) -> Vec<PptxEmbeddedFontEntry> {
    let mut reader = Reader::from_str(xml);
    let mut entries: Vec<PptxEmbeddedFontEntry> = Vec::new();
    let mut in_embedded_font = false;
    let mut current_typeface: Option<String> = None;
    let mut current_variants: Vec<FontVariantRef> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"embeddedFont" {
                    in_embedded_font = true;
                    current_typeface = None;
                    current_variants = Vec::new();
                }
            }
            Ok(Event::Empty(ref e)) if in_embedded_font => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"font" => {
                        current_typeface = get_attr_str(e, b"typeface");
                    }
                    b"regular" | b"bold" | b"italic" | b"boldItalic" => {
                        if let Some(r_id) = get_attr_str(e, b"r:id") {
                            let style = String::from_utf8_lossy(local_name.as_ref()).to_string();
                            current_variants.push(FontVariantRef { style, r_id });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"embeddedFont" && in_embedded_font {
                    in_embedded_font = false;
                    if let Some(typeface) = current_typeface.take()
                        && !current_variants.is_empty()
                    {
                        entries.push(PptxEmbeddedFontEntry {
                            typeface,
                            variants: std::mem::take(&mut current_variants),
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    entries
}

/// Extract GUID from a PPTX font file path like `ppt/fonts/{GUID}.fntdata`.
fn extract_guid_from_font_path(path: &str) -> Option<String> {
    let filename = path.rsplit('/').next()?;
    let stem = filename.strip_suffix(".fntdata")?;
    // Verify it looks like a GUID: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
    if stem.starts_with('{') && stem.ends_with('}') && stem.len() == 38 {
        Some(stem.to_string())
    } else {
        None
    }
}

fn extract_pptx_font_data(data: &[u8]) -> Option<EmbeddedFontData> {
    let mut archive = crate::parser::open_zip(data).ok()?;

    // Read presentation.xml
    let pres_xml = {
        let mut file = archive.by_name("ppt/presentation.xml").ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;
        content
    };

    let font_entries = parse_pptx_embedded_font_list(&pres_xml);
    if font_entries.is_empty() {
        return None;
    }

    // Read presentation.xml.rels to resolve rId → target
    let rels_xml = {
        let mut file = archive.by_name("ppt/_rels/presentation.xml.rels").ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;
        content
    };
    let rels = crate::parser::xml_util::parse_rels_id_target(&rels_xml);

    let mut faces = Vec::new();

    for entry in &font_entries {
        for variant in &entry.variants {
            let Some(target) = rels.get(&variant.r_id) else {
                continue;
            };

            // Resolve path relative to ppt/
            let font_zip_path = if target.starts_with('/') {
                target.trim_start_matches('/').to_string()
            } else {
                format!("ppt/{target}")
            };

            // Extract GUID from filename
            let Some(guid_str) = extract_guid_from_font_path(&font_zip_path) else {
                continue;
            };
            let Some(key) = parse_guid_to_bytes(&guid_str) else {
                continue;
            };

            // Read font data from ZIP
            let mut font_data = Vec::new();
            {
                let mut file = match archive.by_name(&font_zip_path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                if file.read_to_end(&mut font_data).is_err() {
                    continue;
                }
            }

            // Deobfuscate
            deobfuscate_font_data(&mut font_data, &key);

            faces.push(deobfuscated_face(
                font_data,
                &entry.typeface,
                &variant.style,
            ));
        }
    }

    (!faces.is_empty()).then_some(EmbeddedFontData { faces })
}

// =============================================================================
// DOCX extraction
// =============================================================================

/// Parse `<w:font>` elements with embedded font references from `word/fontTable.xml`.
fn parse_docx_embedded_font_entries(xml: &str) -> Vec<DocxEmbeddedFontEntry> {
    let mut reader = Reader::from_str(xml);
    let mut entries: Vec<DocxEmbeddedFontEntry> = Vec::new();
    let mut in_font = false;
    let mut current_name: Option<String> = None;
    let mut current_variants: Vec<DocxFontVariantRef> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"font" {
                    in_font = true;
                    current_name = get_attr_str(e, b"w:name").or_else(|| get_attr_str(e, b"name"));
                    current_variants = Vec::new();
                }
            }
            Ok(Event::Empty(ref e)) if in_font => {
                let local_name = e.local_name();
                let style = match local_name.as_ref() {
                    b"embedRegular" => Some("regular"),
                    b"embedBold" => Some("bold"),
                    b"embedItalic" => Some("italic"),
                    b"embedBoldItalic" => Some("boldItalic"),
                    _ => None,
                };
                if let Some(style) = style {
                    let r_id = get_attr_str(e, b"r:id");
                    let font_key =
                        get_attr_str(e, b"w:fontKey").or_else(|| get_attr_str(e, b"fontKey"));
                    if let (Some(r_id), Some(font_key)) = (r_id, font_key) {
                        current_variants.push(DocxFontVariantRef {
                            style: style.to_string(),
                            r_id,
                            font_key,
                        });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"font" && in_font {
                    in_font = false;
                    if let Some(name) = current_name.take()
                        && !current_variants.is_empty()
                    {
                        entries.push(DocxEmbeddedFontEntry {
                            font_name: name,
                            variants: std::mem::take(&mut current_variants),
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    entries
}

fn extract_docx_font_data(data: &[u8]) -> Option<EmbeddedFontData> {
    let mut archive = crate::parser::open_zip(data).ok()?;

    // Read word/fontTable.xml
    let font_table_xml = {
        let mut file = archive.by_name("word/fontTable.xml").ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;
        content
    };

    let font_entries = parse_docx_embedded_font_entries(&font_table_xml);
    if font_entries.is_empty() {
        return None;
    }

    // Read word/_rels/fontTable.xml.rels
    let rels_xml = {
        let mut file = archive.by_name("word/_rels/fontTable.xml.rels").ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;
        content
    };
    let rels = crate::parser::xml_util::parse_rels_id_target(&rels_xml);

    let mut faces = Vec::new();

    for entry in &font_entries {
        for variant in &entry.variants {
            let Some(target) = rels.get(&variant.r_id) else {
                continue;
            };

            // Resolve path relative to word/
            let font_zip_path = if target.starts_with('/') {
                target.trim_start_matches('/').to_string()
            } else {
                format!("word/{target}")
            };

            // Parse GUID from fontKey attribute
            let Some(key) = parse_guid_to_bytes(&variant.font_key) else {
                continue;
            };

            // Read font data from ZIP
            let mut font_data = Vec::new();
            {
                let mut file = match archive.by_name(&font_zip_path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                if file.read_to_end(&mut font_data).is_err() {
                    continue;
                }
            }

            // Deobfuscate
            deobfuscate_font_data(&mut font_data, &key);

            faces.push(deobfuscated_face(
                font_data,
                &entry.font_name,
                &variant.style,
            ));
        }
    }

    (!faces.is_empty()).then_some(EmbeddedFontData { faces })
}

// =============================================================================
// Helpers
// =============================================================================

/// Create a fresh directory for one conversion's fonts.
///
/// `create_dir` fails on an existing path, so a directory or symlink planted
/// at the predicted name is never reused. The process id and counter keep
/// concurrent conversions apart when the clock is coarse.
#[cfg(not(target_arch = "wasm32"))]
fn create_temp_font_dir(prefix: &str) -> Option<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    let pid = std::process::id();
    let sequence = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("{prefix}-{pid}-{nanos}-{sequence}"));
    std::fs::create_dir(&path).ok()?;
    Some(path)
}

/// Write a file that must not exist yet, refusing to follow or replace one.
#[cfg(not(target_arch = "wasm32"))]
fn write_new_file(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(data)
}
