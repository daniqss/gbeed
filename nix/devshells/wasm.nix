{
  pkgs,
  rustToolchain,
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    just
    cmake
    clang
    emscripten
    python3
    rustToolchain
  ];

  env = {
    RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

    EMCC_CFLAGS = pkgs.lib.concatStringsSep " " [
      "-O3"
      "-sUSE_GLFW=3"
      "-sASSERTIONS=1"
      "-sWASM=1"
      "-sSTACK_SIZE=2097152"
      "-sGL_ENABLE_GET_PROC_ADDRESS=1"
      "-sEXPORTED_RUNTIME_METHODS=FS,HEAPU8,ccall,cwrap"
    ];
  };
}
