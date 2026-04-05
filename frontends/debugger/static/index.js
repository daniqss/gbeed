function loadRomFile(file) {
  const reader = new FileReader();
  reader.onload = function (e) {
    const data = new Uint8Array(e.target.result);
    try {
      Module.FS.mkdirTree("/.glfw_dropped_files");
    } catch (e) {
      console.warn("Failed to create directory for the ROM: ", e);
    }
    const filePath = "/.glfw_dropped_files/" + file.name;
    Module.FS.writeFile(filePath, data);

    // check correct writing
    const written = Module.FS.readFile(filePath);
    if (written.length !== data.length) {
      console.error("ROM write mismatch:", written.length, "!=", data.length);
      return;
    }

    // load the ROM in the emulator
    Module.ccall("load_rom_from_js", null, ["string"], [filePath]);

    // hide button after loading first ROM
    document.getElementById("open-rom-btn").style.display = "none";
  };
  reader.readAsArrayBuffer(file);
}

// open file dialog when the button is clicked, and load the selected ROM
const romInput = document.getElementById("rom-input");
document.getElementById("open-rom-btn").addEventListener("click", () => {
  romInput.click();
});
romInput.addEventListener("change", () => {
  if (romInput.files.length > 0) {
    loadRomFile(romInput.files[0]);
    romInput.value = "";
  }
});

var Module = {
  canvas: document.getElementById("canvas"),
  onRuntimeInitialized: () => {
    window.addEventListener("beforeunload", () => {
      if (Module._save_game_wasm) {
        Module._save_game_wasm();
      }
    });

    window.addEventListener("pagehide", () => {
      if (Module._save_game_wasm) Module._save_game_wasm();
    });

    document.addEventListener("visibilitychange", () => {
      if (document.visibilityState === "hidden" && Module._save_game_wasm)
        Module._save_game_wasm();
    });
  },
};
