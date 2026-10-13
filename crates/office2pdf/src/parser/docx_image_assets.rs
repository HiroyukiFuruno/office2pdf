use std::io::Cursor;
use std::sync::LazyLock;

use typst::foundations::Bytes;
use typst::visualize::{ExchangeFormat, RasterImage};

use super::{ConvertWarning, DocxImageAsset, ImageFormat};

pub(super) fn normalize_png_preview(
    data: &[u8],
    relationship_id: &str,
    warnings: &mut Vec<ConvertWarning>,
) -> DocxImageAsset {
    // This shares the renderer's memoized decoder and preserves valid PNG color
    // profiles and metadata instead of unnecessarily re-encoding every picture.
    let decoded = RasterImage::plain(Bytes::new(data.to_vec()), ExchangeFormat::Png);
    let data = match decoded {
        Ok(_) => data.to_vec(),
        Err(error) => {
            let recovered = recover_png(data);
            let recovery = if recovered.is_some() {
                "retained decoded samples; unavailable samples are zero-filled"
            } else {
                "black placeholder; original drawing extent retained"
            };
            tracing::warn!(
                format = "DOCX",
                image_relationship_id = relationship_id,
                input_bytes = data.len(),
                reason = %error,
                recovery,
                "Recovered damaged PNG picture"
            );
            warnings.push(ConvertWarning::PartialElement {
                format: "DOCX".to_string(),
                element: "image".to_string(),
                detail: format!("PNG decoding failed: {error}; {recovery}"),
            });
            recovered.unwrap_or_else(|| BLACK_PLACEHOLDER.clone())
        }
    };
    DocxImageAsset {
        data,
        format: ImageFormat::Png,
    }
}

fn recover_png(data: &[u8]) -> Option<Vec<u8>> {
    let mut options = png::DecodeOptions::default();
    // A bad checksum need not destroy otherwise decodable image samples.
    options.set_ignore_checksums(true);
    options.set_ignore_text_chunk(true);
    let mut decoder = png::Decoder::new_with_options(Cursor::new(data), options);
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    let size = reader.output_buffer_size()?;
    // The decoder's limits do not cover the caller's output allocation.
    let allocation_limit = image::Limits::default().max_alloc?;
    if u64::try_from(size).ok()? > allocation_limit {
        return None;
    }
    let (color_type, bit_depth) = reader.output_color_type();
    let mut pixels: Vec<u8> = vec![0; size];
    // next_frame writes each decoded row/pass before reporting a damaged tail.
    // The initialized remainder is deterministic, including Adam7 holes.
    let _frame_result = reader.next_frame(&mut pixels);
    let mut info = reader.info().clone();
    info.color_type = color_type;
    info.bit_depth = bit_depth;
    info.interlaced = false;
    info.palette = None;
    info.trns = None;
    info.bkgd = None;
    info.sbit = None;
    info.animation_control = None;
    info.frame_control = None;
    let mut normalized: Vec<u8> = Vec::new();
    {
        let encoder = png::Encoder::with_info(&mut normalized, info).ok()?;
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&pixels).ok()?;
    }
    Some(normalized)
}

static BLACK_PLACEHOLDER: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut data: Vec<u8> = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut data, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("memory-backed PNG header");
        writer
            .write_image_data(&[0, 0, 0])
            .expect("one RGB pixel in a memory-backed PNG");
    }
    data
});
