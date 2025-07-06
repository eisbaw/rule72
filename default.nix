{ lib, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "rule72";
  version = "0.1.0";

  # The crate lives in a subdirectory
  src = ./rule72;

  # Use the existing Cargo.lock for reproducible builds
  cargoLock.lockFile = ./rule72/Cargo.lock;

  doCheck = true;

  meta = with lib; {
    description = "Git commit-message reflow CLI tool";
    homepage = "https://github.com/eisbaw/rule72";
    license = licenses.asl20;
    maintainers = [ "Mark Ruvald Pedersen" ];
    mainProgram = "rule72";
  };
} 