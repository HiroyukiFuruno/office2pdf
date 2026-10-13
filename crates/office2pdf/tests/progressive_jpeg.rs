#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read};

const DOCX: &[u8] = include_bytes!("../../../tests/fixtures/docx/progressive_dc_tables.docx");

fn assert_picture_colors(name: &str, dimensions: (u32, u32), samples: &[(u32, u32, [u8; 3])]) {
    let mut package = zip::ZipArchive::new(Cursor::new(DOCX)).unwrap();
    let mut jpeg: Vec<u8> = Vec::new();
    package
        .by_name(&format!("word/media/{name}"))
        .unwrap()
        .read_to_end(&mut jpeg)
        .unwrap();
    let picture = image::load_from_memory(&jpeg).unwrap().to_rgb8();
    assert_eq!(picture.dimensions(), dimensions);
    // Independent libjpeg-turbo measurements allow only integer-IDCT rounding.
    for &(x, y, expected) in samples {
        let actual = picture.get_pixel(x, y).0;
        for channel in 0..3 {
            assert!(
                actual[channel].abs_diff(expected[channel]) <= 2,
                "{name} pixel ({x}, {y}) channel {channel}: {}, expected {}",
                actual[channel],
                expected[channel]
            );
        }
    }
}

#[test]
fn progressive_picture_uses_chroma_dc_table_one() {
    assert_picture_colors(
        "noninterleaved-dc-table-1-17x19.jpg",
        (17, 19),
        &[
            (5, 5, [221, 30, 20]),
            (15, 5, [19, 199, 42]),
            (5, 16, [40, 20, 231]),
        ],
    );
}

#[test]
fn progressive_picture_uses_chroma_dc_table_two() {
    assert_picture_colors(
        "noninterleaved-dc-table-2-83x59.jpg",
        (83, 59),
        &[
            (5, 5, [221, 30, 20]),
            (18, 5, [20, 200, 39]),
            (31, 5, [42, 20, 229]),
            (44, 5, [230, 180, 21]),
            (57, 5, [21, 180, 220]),
            (70, 5, [241, 240, 246]),
            (5, 16, [43, 18, 231]),
            (57, 49, [222, 28, 16]),
        ],
    );
}

#[test]
fn progressive_picture_uses_chroma_dc_table_three() {
    assert_picture_colors(
        "noninterleaved-dc-table-3-160x120.jpg",
        (160, 120),
        &[
            (5, 5, [221, 30, 20]),
            (18, 5, [20, 200, 39]),
            (31, 5, [42, 20, 229]),
            (44, 5, [230, 180, 21]),
            (57, 5, [21, 180, 220]),
            (70, 5, [241, 240, 246]),
            (109, 71, [39, 19, 230]),
            (148, 115, [20, 200, 39]),
        ],
    );
}

#[test]
fn docx_with_separate_progressive_dc_scans_converts() {
    let result = office2pdf::convert_bytes(
        DOCX,
        office2pdf::config::Format::Docx,
        &office2pdf::config::ConvertOptions::default(),
    )
    .unwrap();
    assert!(result.pdf.starts_with(b"%PDF"));
}
