{ nixpkgs ? import nix/nixpkgs }:

let

    rustChannel = nixpkgs.rustChannelOf {
        date = "2021-12-02";
        channel = "nightly";
    };

    rust = rustChannel.rust.override {
        targets = [
            "x86_64-pc-windows-gnu"
            "x86_64-unknown-linux-gnu"
        ];
    };

in
    nixpkgs.mkShell {

        # This lists tools that are required during build.
        # Do not include libraries to link with in this list;
        # those should be configured per target operating system.
        nativeBuildInputs = [
            nixpkgs.cacert
                # ^ Used by Cargo to connect with TLS servers.
            nixpkgs.pkgsCross.mingwW64.buildPackages.gcc
                # ^ Used for linking Windows binaries.
            nixpkgs.shaderc
                # ^ Used for compiling shaders to SPIR-V.
            nixpkgs.wineWowPackages.full
                # ^ Used for running Windows binaries.
            rust
        ];

        # For each target operating system we specify the library search path.
        # The build.rs script will pick the correct list depending on target.
        # We separate by ; because that is what WINEPATH is split on by Wine.
        LIBRARIES_linux = nixpkgs.lib.concatStringsSep ";" [
            "${nixpkgs.SDL2}/lib"
        ];
        LIBRARIES_windows = nixpkgs.lib.concatStringsSep ";" [
            "${nixpkgs.pkgsCross.mingwW64.SDL2}/bin"
            "${nixpkgs.pkgsCross.mingwW64.windows.pthreads}/lib"
        ];

    }
