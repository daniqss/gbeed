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
  };
''
