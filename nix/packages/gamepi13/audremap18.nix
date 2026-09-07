''
  /dts-v1/;
  /plugin/;

  / {
    compatible = "brcm,bcm2835";

    fragment@0 {
      target = <&audio_pins>;
      __overlay__ {
        brcm,pins = <18>;
        brcm,function = <2>; /* alt5 */
      };
    };

    fragment@1 {
      target = <&chosen>;
      __overlay__ {
        bootargs = "snd_bcm2835.enable_headphones=1";
      };
    };
  };
''
