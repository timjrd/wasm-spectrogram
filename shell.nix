let
  zeros = "0000000000000000000000000000000000000000000000000000";
  fetch = {url, rev, sha256 ? zeros}: fetchTarball {
    url = "${url}/archive/${rev}.tar.gz";
    inherit sha256;
  };
  nixpkgs = fetch {
    url = https://github.com/NixOS/nixpkgs-channels;
    rev = "cb2cdab71368885ce6408b3ad7cfcf544a8c38a0";
    sha256 = "02vyx2ccrfqxz7ndlfww1ivqbq1qlmglq5690r6nvmylcm976dqw";
  };
  nixpkgs-mozilla = fetch {
    url = https://github.com/mozilla/nixpkgs-mozilla;
    rev = "d46240e8755d91bc36c0c38621af72bf5c489e13";
    sha256 = "0icws1cbdscic8s8lx292chvh3fkkbjp571j89lmmha7vl2n71jg";
  };
in

with import nixpkgs {
  overlays = [(import nixpkgs-mozilla)];
};

stdenv.mkDerivation {
  name = "spectrogram";
  buildInputs = [
    caddy
    minify
    wasm-pack
    wasm-gc
    ((rustChannelOf {
      channel = "nightly";
      date    = "2019-20-11";
      sha256  = "0xwvfpp2lzrbqkmjblvakrr3kvw8l6yyq07lfmbx8g60kbw003l4";
    }).rust.override {
      targets = ["wasm32-unknown-unknown"];
    })
  ];
}
