{
  description = "UTP Core Flake";

  # TODO: add the UTP cachix cache!

  inputs = {
    nixpkgs.url      = github:NixOS/nixpkgs/nixos-21.11;
    rust-overlay.url = github:oxalica/rust-overlay;
    flake-utils.url  = github:numtide/flake-utils;
    nur.url          = github:nix-community/NUR;
  };

  # TODO: cargo extensions (cargo-expand)
  # TODO: CI setup? (garnix)
  # TODO: expose targets, etc.

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nur }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # TODO: make a nixpkg of its own and upstream:
        gdb-tools = pkgs: with pkgs.python3Packages; buildPythonPackage rec {
          pname = "gdb-tools";
          version = "1.4";

          propagatedBuildInputs = [ arpeggio ];
          src = fetchPypi {
            inherit pname version;
            sha256 = "NYtmI+0qeVx58vy49CRMEZw1jzZOgwElHUVIE1VpNEc=";
          };

          format = "pyproject";

          meta = with pkgs.lib; {
            # maintainers = with maintainers; [ TODO ];
            # description = "TODO"
            # license = bsd clause 3
          };
        };

        overlays = [ (import rust-overlay) nur.overlay ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # `gdb` is broken on ARM macOS so we'll fallback to using x86_64 GDB
        # there (assuming Rosetta is installed: https://github.com/NixOS/nix/pull/4310).
        #
        # See: https://github.com/NixOS/nixpkgs/issues/147953
        gdbPkgs' = let
          pkgs' = if pkgs.stdenv.isDarwin && pkgs.stdenv.isAarch64 then
            (import nixpkgs { system = "x86_64-darwin"; inherit overlays; })
          else
            pkgs;
        in
          [ pkgs'.gdb pkgs'.nur.repos.mic92.gdb-dashboard (gdb-tools pkgs') ]
        ;

        # As per https://github.com/ut-utp/.github/wiki/Dev-Environment-Setup#embedded-development-setup
        # on Linux we need to expose `gdb` as `gdb-multiarch`
        # (to match other distros):
        gdbPkgs = if pkgs.stdenv.isLinux then
          let
            baseGdb = builtins.head gdbPkgs';
            gdbMultiarch = pkgs.stdenvNoCC.mkDerivation {
              pname = "gdb-multiarch";
              inherit (baseGdb) version meta;
              nativeBuildInputs = with pkgs; [ makeWrapper ];
              unpackPhase = "true";
              installPhase = ''
                mkdir -p $out/bin
                makeWrapper ${baseGdb}/bin/gdb $out/bin/gdb-multiarch
              '';
            };
          in
          [gdbMultiarch] ++ gdbPkgs'
        else
          gdbPkgs';

        rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
        llvm-tools-preview = builtins.head (builtins.filter (p: p.pname == "llvm-tools-preview") rust-toolchain.paths);

        # Should bump this manually rather than always grab the newest, I think.
        nightly-rust-toolchain = with pkgs.rust-bin; (nightly."2022-07-02".default);
        # nightly-rust-toolchain = with pkgs.rust-bin; (selectLatestNightlyWith (toolchain: toolchain.default));

        cargo-nightly = pkgs.writeShellScriptBin "cargo-nightly" ''
          export RUSTC="${nightly-rust-toolchain}/bin/rustc";
          export CARGO="${nightly-rust-toolchain}/bin/cargo";
          export RUSTDOC="${nightly-rust-toolchain}/bin/rustdoc";
          export RUSTFMT="${nightly-rust-toolchain}/bin/rustfmt";
          # TODO: clippy
          exec "$CARGO" "$@"
        '';
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            rust-toolchain
            cargo-nightly

            cargo-bloat cargo-asm cargo-expand
          ] ++ gdbPkgs;
        };
      }
    );
}
