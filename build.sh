#! /usr/bin/env nix-shell
#! nix-shell -i bash

if test "$1" = release
then
  RELEASE=true
  WASM_PACK_FLAGS="--release"
else
  RELEASE=false
  WASM_PACK_FLAGS="--debug"
fi

wasm-pack \
  build \
  --target web \
  --no-typescript \
  --out-dir bin \
  --out-name index \
  $WASM_PACK_FLAGS \
  | sed 's/\x1b\[[0-9;]*m//g' # remove ANSI colors

rm -f bin/{.gitignore,package.json}

if test $RELEASE = true
then
  min=$(minify bin/index.js | tr -d "\n")
  echo -n "$min" > bin/index.js
  wasm-gc bin/index_bg.wasm
fi
