{
  description = "MeetCal web development environment";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [ "aarch64-darwin" "aarch64-linux" "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc cargo clippy rustfmt rust-analyzer cargo-audit
              trunk leptosfmt wasm-bindgen-cli binaryen pkg-config git just
            ];
            shellHook = ''echo "Nix dev shell: MeetCal web"'';
          };
        });
    };
}
