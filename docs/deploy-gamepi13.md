# Deploying gbeed on a Waveshare GamePi13

This guide covers running the `console` frontend on a **Waveshare GamePi13** (a Raspberry Pi with an SPI ST7789V display and a PWM speaker), booting directly into gbeed. This is a manual installation on top of an existing Rasbian Bookworm Lite ([Waveshare Image](https://www.waveshare.com/wiki/GamePi13#Pre-installed_Image)), using the [Debian armv6l cross-compiled binary](../README.md#how-to-build-for-armv6l-debian-linux).

## Boot chain

The GamePi13 has no DRM/KMS device, so gbeed runs on the raylib **X11** backend (not DRM). At boot, tty1 auto-logs in, starts an X server, and runs gbeed inside it. `fbcp` mirrors the HDMI framebuffer (`fb0`) onto the SPI display (`fb1`), and audio is routed to the PWM speaker.

| Stage | Component          | Role                                                       |
|-------|--------------------|------------------------------------------------------------|
| 1     | `config.txt`       | Loads overlays: SPI display (`fb1`) + PWM audio (GPIO18)   |
| 2     | `getty@tty1`       | Passwordless autologin                                     |
| 3     | `~/.bash_profile`  | On tty1 only: `FRAMEBUFFER=/dev/fb0` ; `fbcp &` ; `startx` |
| 4     | `~/.xinitrc`       | `exec gbeed` — X renders to `fb0`                          |
| 5     | `fbcp`             | Mirrors `fb0` -> `fb1` (SPI panel)                         |
| 6     | `/etc/asound.conf` | Default sink -> `hw:1,0` (Headphones/PWM) -> speaker       |

## Install the binary

Cross-compile with `just cross-build-debian` (produces `./gbeed-debian`, armv6l / X11-GLFW / glibc 2.28), copy it to the Pi, and install it on the `PATH`:

```sh
sudo install -m 755 gbeed-debian /usr/local/bin/gbeed
```

## Overlays and `/boot/firmware/config.txt`

Two Device Tree overlays are needed, both under `/boot/firmware/overlays/`: `waveshare13` (SPI display driver, creates `/dev/fb1`) and `audremap18`.
These overlays can be downloaded from [Waveshare GamePi13 wiki](https://www.waveshare.com/wiki/GamePi13)

The relevant lines in `/boot/firmware/config.txt`:

```ini
dtparam=audio=on
dtparam=spi=on
dtoverlay=waveshare13
dtoverlay=audremap18

# keep commented: KMS breaks fbcp/DispmanX
# dtoverlay=vc4-kms-v3d

disable_fw_kms_setup=1
hdmi_force_hotplug=1
hdmi_group=2
hdmi_mode=87

hdmi_cvt 480 480 60 6 0 0 0
hdmi_drive=2
```

> **Important:** `vc4-kms-v3d` must stay commented. KMS disables the legacy firmware framebuffer that `fbcp` captures through DispmanX, which leaves the SPI panel black.

## Passwordless autologin on tty1

Create the systemd drop-in `/etc/systemd/system/getty@tty1.service.d/autologin.conf`:

```ini
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin daniqss --noclear %I $TERM
```

The empty `ExecStart=` resets the original command before overriding it. Apply with:

```sh
sudo systemctl daemon-reload && sudo systemctl restart getty@tty1
```

## Start X on login: `~/.bash_profile`

Once tty1 logs in, the login shell starts the display stack. The `tty1` guard prevents X from launching over SSH sessions:

```sh
if [ "$(tty)" = "/dev/tty1" ]; then
    export FRAMEBUFFER=/dev/fb0
    fbcp &
    startx
fi
```

X (fbdev driver) renders to `fb0`; `fbcp` mirrors `fb0 -> fb1` onto the SPI panel.

## Run gbeed inside X: `~/.xinitrc`

```sh
#!/bin/sh
xset -dpms
xset s off
xset s noblank
exec /usr/local/bin/gbeed
```

The `xset` calls disable screen blanking and DPMS. `exec` replaces the shell with gbeed, so X exits when gbeed does.

## Audio

This system runs raw ALSA (no PulseAudio/PipeWire). `aplay -l` reports two cards: `card 0` is HDMI (unused, no monitor) and **`card 1` is `Headphones`** — the analog/PWM output that reaches the speaker via `audremap18`.

gbeed (raylib/miniaudio) opens the ALSA *default* device, so the default must point at card 1. Create `/etc/asound.conf`:

```ini
pcm.!default {
    type plug
    slave.pcm "hw:1,0"
}
ctl.!default {
    type hw
    card 1
}
```

Test with `speaker-test -twav -l1` (without `-D`, so it uses the default), it should play through the speaker.

## The `libbcm_host` dependency
`fbcp` captures the framebuffer through DispmanX, which requires `libbcm_host.so.0` from the `libraspberrypi0` package. If that library goes missing (e.g. removed by an unrelated `apt` operation), `fbcp` fails to start and the SPI panel stays black even though gbeed is rendering fine to `fb0`. Pin the package so autoremove can't take it, and dry-run `apt` before installing anything:

```sh
sudo apt-mark manual libraspberrypi0
sudo apt-get install --dry-run <pkg>
```

## Troubleshooting
Verify the stack one layer at a time:

| Layer       | Command                                 | Expected                |
|-------------|-----------------------------------------|-------------------------|
| SPI panel   | `cat /dev/urandom \| sudo tee /dev/fb1` | colored noise on screen |
| fbcp        | `pgrep -a fbcp`                         | returns a PID           |
| libbcm_host | `ldconfig -p \| grep bcm_host`          | library listed          |
| X server    | `pgrep -a Xorg`                         | Xorg on `:0 vt1`        |
| audio       | `speaker-test -twav -l1`                | sound (without `-D`)    |
