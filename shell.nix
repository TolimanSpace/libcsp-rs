{ pkgs ? import <nixpkgs> { } }:
with pkgs;
let
  libsocketcan = stdenv.mkDerivation rec {
    pname = "libsocketcan";
    version = "r101.b464485";

    src = fetchgit {
      url = "https://github.com/lalten/libsocketcan";
      rev = "b464485";
      sha256 = "sha256-MgHdiVju327plJnSBABA2ugzLZETQpPvUg7JluaIJQU=";
    };

    nativeBuildInputs = [ autoreconfHook ];

    postPatchPhase = ''
      patchShebangs .
      ./autogen.sh
      ./configure --prefix=$out
    '';

    buildPhase = ''
      cp README.md README
      make
    '';

    installPhase = ''
      make install
    '';

    # meta = with stdenv.lib; {
    #   description = "Allows control of some basic functions in SocketCAN from userspace";
    #   homepage = "http://lalten.github.io/libsocketcan/";
    #   license = licenses.lgpl21;
    #   platforms = platforms.linux;
    # };
  };
in
let
  libcsp = stdenv.mkDerivation rec {
    name = "libcsp";
    version = "1.6-patched";
    src = fetchFromGitHub {
      owner = "TolimanSpace";
      repo = "libcsp";
      rev = "140463b";
      sha256 = "sha256-2WBy2UjR+ulNoKfsz7V/v8Bp8O4T72ZjpCP8f5NcC7M=";
    };

    buildInputs = [
      zeromq
      python3
      libyaml
      libsocketcan
    ];

    nativeBuildInputs = [
      pkg-config
    ];

    patchPhase = ''
      # Configure `waf` to use Nix's python path rather than the system one. This is only needed for Nix.
      patchShebangs ./waf
    '';

    configurePhase = ''
      # Configure the CSP library compilation with all relevant features.
      ./waf configure --enable-shlib --prefix=$out --install-csp --enable-if-zmqhub --enable-can-socketcan --with-driver-usart=linux
    '';

    buildPhase = ''
      # Build the CSP library
      ./waf build
    '';

    installPhase = ''
      # ./waf install # This appears to be buggy, not getting all install files. Instead, do it manually:

      # Copy the library files
      mkdir -p $out/lib
      cp build/libcsp.so $out/lib
      cp build/libcsp.a $out/lib

      # Copy the header files
      mkdir -p $out/include/csp
      cp -r include/csp/* $out/include/csp
      cp -r build/include/csp/* $out/include/csp

      # Write a pkgconfig file so that the library can be detected in the environment
      mkdir -p $out/lib/pkgconfig
      cat > $out/lib/pkgconfig/libcsp.pc <<EOF
      prefix=$out
      exec_prefix=$out
      libdir=$out/lib
      includedir=$out/include

      Name: libcsp
      Description: CSP library
      Version: ${version}
      Libs: -L''${libdir} -lcsp
      Cflags: -I''${includedir}
      EOF
    '';
  };

in
mkShell {
  nativeBuildInputs = [
    pkg-config
    clang-tools
  ];

  buildInputs = [
    llvmPackages_16.clang
    zeromq
    rustup
    libcsp
  ];

  LIBCLANG_PATH = "${llvmPackages_16.libclang.lib}/lib";
}
