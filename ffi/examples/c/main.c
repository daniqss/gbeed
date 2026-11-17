// Minimal gbeed host in C: runs a ROM, prints the serial output and dumps the last frame as a PGM image.
// Links the same against the std library (`just ffi example-c`) or the no_std one (`just ffi example-c-nostd`).
//
//   just ffi build
//   cc ffi/examples/c/main.c -Iffi/include -Ltarget/release -lgbeed -o gbeed-c
//   LD_LIBRARY_PATH=target/release ./gbeed-c rom.gb [frames]

#include <stdio.h>
#include <stdlib.h>

#include "gbeed.h"

typedef struct {
    unsigned frames_drawn;
    unsigned audio_flushes;
} Stats;

static uint8_t *read_file(const char *path, size_t *len) {
    FILE *file = fopen(path, "rb");
    if (!file) return NULL;

    fseek(file, 0, SEEK_END);
    *len = (size_t)ftell(file);
    rewind(file);

    uint8_t *data = malloc(*len);
    if (data && fread(data, 1, *len, file) != *len) {
        free(data);
        data = NULL;
    }

    fclose(file);
    return data;
}

static void on_transfer(void *user_data, uint8_t data) {
    (void)user_data;
    putchar(data);
    fflush(stdout);
}

static void update_screen(void *user_data) { ((Stats *)user_data)->frames_drawn++; }

static void flush_buffer(void *user_data) { ((Stats *)user_data)->audio_flushes++; }

static void write_pgm(const char *path, const uint8_t *framebuffer) {
    FILE *file = fopen(path, "wb");
    if (!file) return;

    fprintf(file, "P5\n%d %d\n255\n", GB_SCREEN_WIDTH, GB_SCREEN_HEIGHT);
    for (size_t i = 0; i < GB_SCREEN_WIDTH * GB_SCREEN_HEIGHT; i++) {
        // shade 0 is the lightest color
        fputc(255 - framebuffer[i] * 85, file);
    }

    fclose(file);
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <rom> [frames]\n", argv[0]);
        return 1;
    }

    unsigned frames = argc > 2 ? (unsigned)atoi(argv[2]) : 60 * 10;

    size_t rom_len;
    uint8_t *rom = read_file(argv[1], &rom_len);
    if (!rom) {
        fprintf(stderr, "could not read %s\n", argv[1]);
        return 1;
    }

    // the emulator lives in memory owned by the host, on embedded targets it can be a static buffer
    size_t align = gbeed_emu_align();
    size_t size = (gbeed_emu_size() + align - 1) / align * align;
    void *storage = aligned_alloc(align, size);

    GbeedError error;
    GbeedEmu *emu = gbeed_init(storage, size, rom, rom_len, NULL, 0, NULL, 0, &error);
    // the ROM is copied, it can be freed right away
    free(rom);

    if (!emu) {
        fprintf(stderr, "could not create the emulator: %s\n", gbeed_error_message(error));
        free(storage);
        return 1;
    }

    // write_pixel and push_sample are left NULL, the frame and the samples are read from the internal buffers
    Stats stats = {0};
    GbeedController controller = {
        .user_data = &stats,
        .renderer = {.update_screen = update_screen},
        .audio_player = {.flush_buffer = flush_buffer},
        .serial_listener = {.on_transfer = on_transfer},
    };
    gbeed_set_controller(emu, &controller);

    size_t samples = 0;
    for (unsigned i = 0; i < frames; i++) {
        error = gbeed_run_frame(emu);
        if (error != GBEED_ERROR_OK) {
            fprintf(stderr, "emulation error: %s\n", gbeed_error_message(error));
            break;
        }

        size_t len;
        gbeed_audio_samples(emu, &len);
        samples += len;
    }

    write_pgm("frame.pgm", gbeed_framebuffer(emu));
    printf("\n%u frames drawn, %u audio flushes, %zu samples, %zu bytes of emulator, last frame written to frame.pgm\n",
           stats.frames_drawn, stats.audio_flushes, samples, size);

    gbeed_deinit(emu);
    free(storage);
    return error == GBEED_ERROR_OK ? 0 : 1;
}
