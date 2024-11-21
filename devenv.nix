{ pkgs, lib, config, inputs, ... }:

# See: https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md#nix

let
  # Application dependencies
  deps = with pkgs; [
      zstd
    ] ++ lib.optionals stdenv.hostPlatform.isLinux [
      alsa-lib # Needed for audio
      udev # Needed for pkg-config
      vulkan-loader
      libxkbcommon wayland # To use the wayland feature
      xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
    ] ++ lib.optionals stdenv.hostPlatform.isDarwin [
      darwin.apple_sdk_11_0.frameworks.Cocoa
      rustPlatform.bindgenHook
    ];
in
{
  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    pkg-config
  ] ++ deps;

  # Necessary for linux development
  # In the long term, do something similar to: https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/ju/jumpy/package.nix
  env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps;

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [
      "cargo"
      "rust-src"
      "rustc"
      # Needed for development on nixos
      # Path for your editor should be "./.devenv/profile/bin/rust-analyzer"
      "rust-analyzer"
    ];
  };

  # https://devenv.sh/processes/
  processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/scripts/
  enterShell = ''
    echo "Welcome to Harmony!"
    rustc --version
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/pre-commit-hooks/
  pre-commit.hooks = {
    # TODO: add linting
  };

  # See full reference at https://devenv.sh/reference/options/
}
