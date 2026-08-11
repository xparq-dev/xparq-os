use fontdue::{Font, FontSettings};
use std::{env, fs, path::PathBuf};

const CELL_WIDTH: usize = 8;
const CELL_HEIGHT: usize = 16;
const GLYPH_COUNT: usize = 256;
const FONT_SIZE_PX: f32 = 13.0;
const BASELINE: i32 = 13;

fn main() {
    let mut args = env::args().skip(1);
    let source = PathBuf::from(args.next().expect("usage: font-rasterizer <RobotoMono.ttf> <atlas.bin>"));
    let output = PathBuf::from(args.next().expect("usage: font-rasterizer <RobotoMono.ttf> <atlas.bin>"));
    if args.next().is_some() {
        panic!("usage: font-rasterizer <RobotoMono.ttf> <atlas.bin>");
    }

    let bytes = fs::read(&source).expect("failed to read source TTF");
    let font = Font::from_bytes(bytes, FontSettings::default()).expect("failed to parse source TTF");
    let mut atlas = vec![0u8; GLYPH_COUNT * CELL_WIDTH * CELL_HEIGHT];

    for code in 0x20u8..=0x7e {
        rasterize_glyph(&font, code as char, code as usize, &mut atlas);
    }

    let replacement = atlas[(b'?' as usize) * CELL_WIDTH * CELL_HEIGHT
        ..(b'?' as usize + 1) * CELL_WIDTH * CELL_HEIGHT]
        .to_vec();
    for code in 0..GLYPH_COUNT {
        if !(0x20..=0x7e).contains(&code) {
            let start = code * CELL_WIDTH * CELL_HEIGHT;
            atlas[start..start + CELL_WIDTH * CELL_HEIGHT].copy_from_slice(&replacement);
        }
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("failed to create atlas directory");
    }
    fs::write(&output, &atlas).expect("failed to write alpha atlas");
    println!("Generated {} bytes at {}", atlas.len(), output.display());
}

fn rasterize_glyph(font: &Font, character: char, code: usize, atlas: &mut [u8]) {
    let (metrics, bitmap) = font.rasterize(character, FONT_SIZE_PX);
    let advance = metrics.advance_width.round() as i32;
    let cell_left = (CELL_WIDTH as i32 - advance) / 2;
    let dst_left = cell_left + metrics.xmin;
    let dst_top = BASELINE - metrics.height as i32 - metrics.ymin;
    let glyph_start = code * CELL_WIDTH * CELL_HEIGHT;

    for source_y in 0..metrics.height {
        for source_x in 0..metrics.width {
            let dst_x = dst_left + source_x as i32;
            let dst_y = dst_top + source_y as i32;
            if dst_x >= 0 && dst_x < CELL_WIDTH as i32 && dst_y >= 0 && dst_y < CELL_HEIGHT as i32 {
                atlas[glyph_start + dst_y as usize * CELL_WIDTH + dst_x as usize] =
                    bitmap[source_y * metrics.width + source_x];
            }
        }
    }
}
