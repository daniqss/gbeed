# ST7789V initialisation sequence for the gamepi13 consumed by the kernel's panel-mipi-dbi driver
# `panel-mipi-dbi` is a generic driver.
# it knows how to push pixels over SPI but need us to tell it how to deal with the specific display.
{
  name = "panel";

  commands = [
    # sleep out
    {cmd = "11";}
    {delay = 120;}

    # COLMOD, 16 bit/pixel, RGB565
    {
      cmd = "3a";
      params = ["05"];
    }
    # PORCTRL
    {
      cmd = "b2";
      params = ["05" "05" "00" "33" "33"];
    }
    # GCTRL, VGH 13.26V, VGL -10.43V
    {
      cmd = "b7";
      params = ["75"];
    }
    # VDVVRHEN, take VDV and VRH from the commands below, not from NVM
    {
      cmd = "c2";
      params = ["01" "ff"];
    }
    # VRHS, VAP 4.1V + (VCOM + offset + 0.5 * VDV)
    {
      cmd = "c3";
      params = ["13"];
    }
    # VDVS — VDV 0V
    {
      cmd = "c4";
      params = ["20"];
    }
    # VCOMS, 0.9V
    {
      cmd = "bb";
      params = ["22"];
    }
    # VCMOFSET, VCOM offset 0V
    {
      cmd = "c5";
      params = ["20"];
    }
    # PWCTRL1, AVDD 6.8V, AVCL -4.8V, VDS 2.3V
    {
      cmd = "d0";
      params = ["a4" "a1"];
    }

    # display on
    {cmd = "29";}
    # INVON, the IPS panel is wired inverted
    {cmd = "21";}

    # MADCTL, memory access control to set orientation.
    # 0x00 is `rotate = 0` with RGB order, matching `waveshare13.dtbo`.
    # TODO: set MX/MY/MV here (0x40/0x80/0x20) if the image comes out mirrored or rotated
    {
      cmd = "36";
      params = ["00"];
    }

    # PVGAMCTRL
    {
      cmd = "e0";
      params = ["d0" "05" "0a" "09" "08" "05" "2e" "44" "45" "0f" "17" "16" "2b" "33"];
    }
    # NVGAMCTRL
    {
      cmd = "e1";
      params = ["d0" "05" "0a" "09" "08" "05" "2e" "43" "45" "0f" "16" "16" "2b" "33"];
    }
  ];
}
