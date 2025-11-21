pub struct Sprite {
    xpos: u8,
    ypos: u8,
    tile_index: u8,
    // TODO: use u8?
    priority: bool,
    xflip: bool,
    yflip: bool,
    // mmm not sure about this one, true = palette 1, false = palette 0
    palette_number: bool,
}
