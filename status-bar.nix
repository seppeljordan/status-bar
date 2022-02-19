{ lib, rustPlatform }:
rustPlatform.buildRustPackage rec {
  pname = "status-bar";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "WuogSTbgkAvKvmUgNiQK2RE18f8fya2uFpVqF3eIaWA=";
  meta = with lib; { description = "A minimal status bar for swaywm"; };
}
