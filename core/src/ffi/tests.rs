use super::*;
use crate::AudioPlayer;
use core::mem::MaybeUninit;
use std::boxed::Box;

// 32KiB of zeros is a valid ROM only cartridge full of `nop`s
static ROM: [u8; 0x8000] = [0; 0x8000];

/// Emulator storage, freed when dropped. Must outlive the handle built on it
struct Storage(Box<MaybeUninit<GbeedEmu>>);

impl Storage {
    fn new() -> Self { Self(Box::new_uninit()) }

    fn init(&mut self) -> *mut GbeedEmu {
        let mut error = GbeedError::Instruction;
        let emu = unsafe {
            gbeed_init(
                self.0.as_mut_ptr().cast(),
                gbeed_emu_size(),
                ROM.as_ptr(),
                ROM.len(),
                ptr::null(),
                0,
                ptr::null(),
                0,
                &mut error,
            )
        };

        assert_eq!(error, GbeedError::Ok);
        assert_eq!(emu, self.0.as_mut_ptr());
        emu
    }
}

#[test]
fn init_errors() {
    let mut storage = Storage::new();
    let ptr = storage.0.as_mut_ptr().cast::<c_void>();
    let size = gbeed_emu_size();
    let mut error = GbeedError::Ok;

    let init = |storage, storage_len, rom: &[u8], boot: Option<&[u8]>, error| unsafe {
        let (boot_ptr, boot_len) = boot.map_or((ptr::null(), 0), |boot| (boot.as_ptr(), boot.len()));
        gbeed_init(
            storage,
            storage_len,
            rom.as_ptr(),
            rom.len(),
            boot_ptr,
            boot_len,
            ptr::null(),
            0,
            error,
        )
    };

    assert!(init(ptr::null_mut(), size, &ROM, None, &mut error).is_null());
    assert_eq!(error, GbeedError::NullPointer);

    assert!(init(ptr, size - 1, &ROM, None, &mut error).is_null());
    assert_eq!(error, GbeedError::InvalidStorage);

    let misaligned = unsafe { ptr.cast::<u8>().add(1) }.cast();
    assert!(init(misaligned, size, &ROM, None, &mut error).is_null());
    assert_eq!(error, GbeedError::InvalidStorage);

    assert!(init(ptr, size, &ROM, Some(&[0; 10]), &mut error).is_null());
    assert_eq!(error, GbeedError::InvalidBootRom);

    let no_rom = unsafe {
        gbeed_init(
            ptr,
            size,
            ptr::null(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            0,
            &mut error,
        )
    };
    assert!(no_rom.is_null());
    assert_eq!(error, GbeedError::NullPointer);
}

#[test]
fn null_handle() {
    unsafe {
        assert_eq!(gbeed_run_frame(ptr::null_mut()), GbeedError::NullPointer);
        assert_eq!(gbeed_step(ptr::null_mut()), GbeedError::NullPointer);
        assert!(gbeed_framebuffer(ptr::null()).is_null());

        let mut len = 1;
        gbeed_audio_samples(ptr::null_mut(), &mut len);
        assert_eq!(len, 0);

        gbeed_deinit(ptr::null_mut());
    }
}

#[test]
fn default_buffers() {
    let mut storage = Storage::new();
    let emu = storage.init();

    unsafe {
        // the first frame returns right away, as the post boot cpu state already has more cycles than a frame
        for _ in 0..2 {
            assert_eq!(gbeed_run_frame(emu), GbeedError::Ok);
        }

        assert!(!gbeed_framebuffer(emu).is_null());

        let mut len = 0;
        assert!(!gbeed_audio_samples(emu, &mut len).is_null());
        assert!(len > 0 && len % 2 == 0);

        // drained
        gbeed_audio_samples(emu, &mut len);
        assert_eq!(len, 0);

        assert!(gbeed_save_data(emu, &mut len).is_null());
        assert_eq!(len, 0);

        gbeed_deinit(emu);
    }
}

#[derive(Default)]
struct Counters {
    pixels: usize,
    frames: usize,
    flushes: usize,
}

unsafe extern "C" fn write_pixel(user_data: *mut c_void, _: usize, _: usize, _: u8, _: u8) {
    unsafe { (*user_data.cast::<Counters>()).pixels += 1 };
}

unsafe extern "C" fn update_screen(user_data: *mut c_void) {
    unsafe { (*user_data.cast::<Counters>()).frames += 1 };
}

unsafe extern "C" fn flush_buffer(user_data: *mut c_void) {
    unsafe { (*user_data.cast::<Counters>()).flushes += 1 };
}

unsafe extern "C" fn playing_stereo(_: *mut c_void) -> bool { false }

#[test]
fn callbacks() {
    let mut storage = Storage::new();
    let emu = storage.init();
    let mut counters = Counters::default();

    let controller = GbeedController {
        user_data: (&mut counters as *mut Counters).cast(),
        renderer: GbeedRenderer {
            write_pixel: Some(write_pixel),
            update_screen: Some(update_screen),
            ..Default::default()
        },
        audio_player: GbeedAudioPlayer {
            playing_stereo: Some(playing_stereo),
            flush_buffer: Some(flush_buffer),
            ..Default::default()
        },
        ..Default::default()
    };

    unsafe {
        gbeed_set_controller(emu, &controller);
        for _ in 0..3 {
            assert_eq!(gbeed_run_frame(emu), GbeedError::Ok);
        }

        // value returning callbacks replace the default implementation
        assert!(!(*emu).controller.playing_stereo());

        // the buffers keep being filled alongside the callbacks
        let mut len = 0;
        gbeed_audio_samples(emu, &mut len);
        assert!(len > 0);

        gbeed_deinit(emu);
    }

    assert_eq!(counters.flushes, 3);
    assert!(counters.frames >= 2);
    assert!(counters.pixels >= 2 * buffers::FRAMEBUFFER_SIZE);
}
