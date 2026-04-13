use raylib::prelude::*;

pub struct Texture {
    texture: Texture2D,
    framebuffer: Box<[u8]>,
}

impl Texture {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let mut img = Image::gen_image_color(width, height, Color::BLACK);
        img.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);
        let texture = rl.load_texture_from_image(thread, &img).unwrap();
        let framebuffer = vec![0u8; (width * height * 3) as usize].into_boxed_slice();
        Self { texture, framebuffer }
    }

    pub fn update(&mut self) {
        if let Err(e) = self.texture.update_texture(&self.framebuffer) {
            eprintln!("texture update failed: {e:?}");
        }
    }
}

impl std::ops::Index<usize> for Texture {
    type Output = u8;

    #[inline(always)]
    fn index(&self, idx: usize) -> &Self::Output { &self.framebuffer[idx] }
}

impl std::ops::IndexMut<usize> for Texture {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output { &mut self.framebuffer[idx] }
}

impl std::convert::AsRef<raylib::ffi::Texture> for Texture {
    fn as_ref(&self) -> &raylib::ffi::Texture { &self.texture }
}
