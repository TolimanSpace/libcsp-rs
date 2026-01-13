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
    version = "2.0-patched";
    src = fetchFromGitHub {
      owner = "TolimanSpace";
      repo = "libcsp";
      rev = "236f869";
      sha256 = "sha256-rnmIDd9Km1ZMzdrsKVnwX5qrlrOfme+1la3hX1flx3k=
";
    };

    buildInputs = [
      zeromq
      python311
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
      cp build/libcsp.so $out/lib/
      cp build/libcsp.a $out/lib/

      # Copy the header files
      mkdir -p $out/include
      cp -r include/csp $out/include/
      if [ -d build/include ]; then
        cp -r build/include/. $out/include/
      fi
      if [ -f build/csp_autoconfig.h ]; then
        cp build/csp_autoconfig.h $out/include/
      fi

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
Libs: -L$out/lib -lcsp
Cflags: -I$out/include
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
    llvmPackages_latest.clang
    zeromq
    rustup
    libcsp
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${libcsp}/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LD_LIBRARY_PATH="${libcsp}/lib:$LD_LIBRARY_PATH"
  '';

  # Change llvmPackages_16 to a supported version, e.g., 17 or 18
  LIBCLANG_PATH = "${llvmPackages_latest.libclang.lib}/lib";
}
