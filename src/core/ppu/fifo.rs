use std::collections::VecDeque;

/// three clocks to fetch eight pixels
/// pauses in 4th clock unless fifo has space
#[derive(Debug, PartialEq)]
enum FetcherState {
    ReadTile,
    ReadData0,
    ReadData1,
    Idle,
    Push,
}

/// shifts a pixel each 4Mhz and send it to the lcd
/// pauses until the fifo has at least 8 pixels
/// this is required to mix background pixel with sprites pixels
#[derive(Debug)]
struct PixelFifo {
    fifo: VecDeque<u32>,
    cur_fetch_state: FetcherState,
    line_x: u8,
    pushed_x: u8,
    fetch_x: u8,
    bgw_fetch_data: [u8; 3],
    fetch_entry_data: [u8; 6],
    map_y: u8,
    map_x: u8,
    tile_y: u8,
    fifo_x: u8,
}

impl PixelFifo {
    fn new() -> Self {
        Self {
            fifo: VecDeque::with_capacity(16),
            cur_fetch_state: FetcherState::ReadTile,
            line_x: 0,
            pushed_x: 0,
            fetch_x: 0,
            bgw_fetch_data: [0; 3],
            fetch_entry_data: [0; 6],
            map_y: 0,
            map_x: 0,
            tile_y: 0,
            fifo_x: 0,
        }
    }
}
