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
$KG v --debug --trace aws-test
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
ls -R ~/.kiro/skills

for f in SKILL.md assets/analysis.json; do
  if [ ! -f ~/.kiro/skills/kg-helper/$f ]; then
    echo "::error::$f missing" >/dev/stderr
    exit 1
  fi
done

(
  cd resources/kg-helper/references
  for f in *.md; do
    if [ ! -f ~/.kiro/skills/kg-helper/references/"$f" ]; then
      echo "::error::$f reference missing" >/dev/stderr
      exit 1
    fi
  done
)

cargo deb
