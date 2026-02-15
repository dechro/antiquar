{
  pkgs,
  lib,
  config,
  ...
}:
{
  devcontainer = {
    enable = true;
    settings = {
      "updateRemoteUserUID" = false;
      "containerUser" = "vscode";
      "remoteUser" = "vscode";
    };
  };
  languages.rust = {
    enable = true;
    mold.enable = false;
    channel = "nightly";
    components = [
      "rustc-codegen-cranelift"
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
    rustflags = "-C link-arg=-fuse-ld=wild -Z threads=0";

  };
  packages = [
    # pkgs.cmake
    pkgs.just
    pkgs.pkg-config
    pkgs.mesa-gl-headers
    pkgs.mesa
    pkgs.libGL
    pkgs.libGLU
    pkgs.libxkbcommon
    pkgs.fontconfig
    pkgs.wayland
    pkgs.libxcb
    pkgs.libx11
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.sccache
    pkgs.wild
    pkgs.clang
  ];
  env = {
    LD_LIBRARY_PATH = lib.makeLibraryPath [
      pkgs.mesa
      pkgs.libGL
      pkgs.libGLU
      pkgs.libxkbcommon
      pkgs.fontconfig
      pkgs.wayland
      pkgs.libxcb
      pkgs.libx11
      pkgs.vulkan-loader
    ];
    RUSTC_WRAPPER = "sccache";
  };
  enterShell = ''
    ulimit -n 16000
    unshare -Umr bash -c "mkdir target &>/dev/null; mount -t tmpfs -o size=8G,noatime tmpfs ./target"
  '';
}
