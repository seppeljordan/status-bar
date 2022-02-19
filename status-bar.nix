{ lib, rustPlatform }:
rustPlatform.buildRustPackage rec {
  pname = "status-bar";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "QkKrGWN1wx5DW+vV5ngGBD91IReqg0adMydDeOfYD6A=";
  meta = with lib; { description = "A minimal status bar for swaywm"; };
}
