{ pkgs ? import <nixpkgs> { } }:
with pkgs; mkShell {
  nativeBuildInputs = [
    pkg-config
    clang-tools
  ];

  buildInputs = [
    llvmPackages_16.clang
    zeromq
    rustup
  ];

  LIBCLANG_PATH = "${llvmPackages_16.libclang.lib}/lib";
}
