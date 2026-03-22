use crate::colors;
use crate::renderer::{TILES_PER_ROW, TILE_PIXEL_SIZE, TILE_TEXTURE_WIDTH};

use raylib::prelude::*;

pub struct Texture {
    pub texture: Texture2D,
    pub pixels: Vec<u8>,
}

impl Texture {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let mut img = Image::gen_image_color(width, height, colors::BACKGROUND);
        img.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);
        let texture = rl.load_texture_from_image(thread, &img).unwrap();
        let pixels = vec![0u8; (width * height * 3) as usize];
        Self { texture, pixels }
    }

    pub fn update(&mut self) {
        if let Err(e) = self.texture.update_texture(&self.pixels) {
            eprintln!("texture update failed: {e:?}");
        }
    }
}

pub fn update_bg_map(
    texture: &mut Texture,
    map_data: &[u8],
    tile_data: &[u8],
    is_mode_8000: bool,
    palette: u8,
) {
    let stride = 256_usize;

    for tile_y in 0..32_usize {
        for tile_x in 0..32_usize {
            let tile_number = map_data[tile_y * 32 + tile_x];

            let tile_base = if is_mode_8000 {
                (tile_number as usize) * 16
            } else {
                let signed_tile_number = tile_number as i8 as i32;
                (0x1000_i32 + signed_tile_number * 16) as usize
            };

            let pixel_x_base = tile_x * 8;
            let pixel_y_base = tile_y * 8;

            for row in 0..8_usize {
                let low_byte = tile_data[tile_base + row * 2];
                let high_byte = tile_data[tile_base + row * 2 + 1];

                for column in 0..8_usize {
                    let bit_index = 7 - column;
                    let color_id = (((high_byte >> bit_index) & 1) << 1) | ((low_byte >> bit_index) & 1);

                    let shade = (palette >> (color_id * 2)) & 0x03;
                    let color = colors::GB_PALETTE[shade as usize];
                    let index = ((pixel_y_base + row) * stride + (pixel_x_base + column)) * 3;

                    texture.pixels[index] = color.r;
                    texture.pixels[index + 1] = color.g;
                    texture.pixels[index + 2] = color.b;
                }
            }
        }
    }

    texture.update();
}

// decodes a 2bpp vram block into the tile texture (region 0/$8000, 1/$8800, 2/$9000)
pub fn update_tiles(texture: &mut Texture, data: &[u8]) {
    let stride = TILE_TEXTURE_WIDTH as usize;

    for tile_index in 0..128_usize {
        let tile_base_x = (tile_index % TILES_PER_ROW as usize) * TILE_PIXEL_SIZE as usize;
        let tile_base_y = (tile_index / TILES_PER_ROW as usize) * TILE_PIXEL_SIZE as usize;
        let data_base = tile_index * 16;

        for row in 0..8_usize {
            let low_byte = data[data_base + row * 2];
            let high_byte = data[data_base + row * 2 + 1];

            for column in 0..8_usize {
                let bit_index = 7 - column;
                let color_id = (((high_byte >> bit_index) & 1) << 1) | ((low_byte >> bit_index) & 1);

                let color = colors::GB_PALETTE[color_id as usize];
                let pixel_x = tile_base_x + column;
                let pixel_y = tile_base_y + row;
                let index = (pixel_y * stride + pixel_x) * 3;

                texture.pixels[index] = color.r;
                texture.pixels[index + 1] = color.g;
                texture.pixels[index + 2] = color.b;
            }
        }
    }

    texture.update();
}
