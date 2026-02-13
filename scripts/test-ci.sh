#!/bin/bash

if [ -z "$CI" ]; then
  echo not running in CI >/dev/stderr
  exit 1
fi

set -e
KG=./target/debug/kg
cargo build

mkdir -p .kiro
cp -a ./data/kiro/generators .kiro

$KG help
$KG --help
$KG init
$KG validate
$KG v
$KG v --debug
$KG v --trace aws-test --debug
$KG v --local
$KG v --global
$KG generate
$KG g
$KG diff
$KG schema manifest | jq . >/dev/null
$KG schema agent | jq . >/dev/null
$KG schema manifest | jq -e '.description | contains("manifest TOML files")' >/dev/null
$KG schema agent | jq -e '.description | contains("agent TOML files")' >/dev/null

$KG bootstrap
cargo deb
