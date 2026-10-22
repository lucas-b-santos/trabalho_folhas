use std::print;

extern crate bmp;

fn main() {
    let img = bmp::open("src/IMG_20230504_090659.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    print!("Image dimensions: {}x{}", img.get_width(), img.get_height());
}
