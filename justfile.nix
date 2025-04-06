{
  writeScriptBin,
  just,
  entr,
  wasm-bindgen-cli,
  binaryen,
  zip,
}:
writeScriptBin "just" ''
  #!/usr/bin/env -S ${just}/bin/just --working-directory . --justfile

  [private]
  default:
    @just --list --list-heading $'Available `just` commands:\n'

  # Build in debug mode
  build:
    cargo build --features bevy/dynamic_linking,bevy/file_watcher

  # Run in debug mode
  run:
    cargo run --features bevy/dynamic_linking,bevy/file_watcher

  # Watch files and execute `just run` on change
  watch:
    #!/bin/sh
    set -o pipefail

    trap "exit 0;" SIGINT

    while sleep 0.1; do
      find . -type f -name '*.rs' | ${entr}/bin/entr -dcr just run
    done

  # Build in release mode
  release:
    cargo build --release

  # Build in release mode for the browser
  www: wasm-bindgen

  # Build in release mode for the browser, optimized for size
  www-opt: wasm-opt

  [private]
  wasm-release:
    cargo build --profile wasm-release --target wasm32-unknown-unknown

  # Build `www-opt` and create a ZIP archive
  www-zip: www-opt
    cd www && ${zip}/bin/zip -r ../target.zip *

  [private]
  wasm-bindgen: wasm-release
    #!/bin/sh
    set -euxo pipefail

    target_directory=$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)
    package=$(cargo metadata --format-version 1 --no-deps | jq -r .packages[0].name)

    ${wasm-bindgen-cli}/bin/wasm-bindgen --out-name index \
      --out-dir www/target \
      --target web "$target_directory/wasm32-unknown-unknown/wasm-release/$package.wasm"

  [private]
  wasm-opt: wasm-bindgen
    ${binaryen}/bin/wasm-opt -Oz --output optimized.wasm www/target/index_bg.wasm
    mv optimized.wasm www/target/index_bg.wasm
''
