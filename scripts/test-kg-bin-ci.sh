#!/bin/bash

if [ -z "$CI" ]; then
  echo not running in CI >/dev/stderr
  exit 1
fi

set -euo pipefail
TARGET_DIR="$(cargo metadata --format-version=1 --no-deps | jq -r '.target_directory')"
KG="${TARGET_DIR}/release/kg"
cargo build --release

mkdir -p .kiro
cp -a ./fixtures/kiro/generators .kiro

$KG help
$KG --help
$KG init --force
$KG init --skeleton
$KG validate
$KG v
$KG v --debug
$KG v --debug --trace aws-test
$KG v --local
$KG v --global
$KG generate
$KG generate --global
$KG g
$KG diff
$KG schema manifest | jq . >/dev/null
$KG schema agent | jq . >/dev/null
$KG schema agent -m | jq . >/dev/null
$KG tree summary
$KG tree summary -f json | jq . >/dev/null
$KG tree details base
$KG validate | head -2 && $KG generate | head -2 && $KG diff | head -2 && $KG schema agent | head -2

# Verify schemas match checked-in files
$KG schema manifest >/tmp/manifest.json
$KG schema agent >/tmp/agent.json
(cd schemas && sha256sum manifest.json agent.json >/tmp/schemas.sha256)
(cd /tmp && sha256sum --check schemas.sha256)

rm -rf .kiro/generators .kiro/agents
mkdir -p ~/.kiro/generators
cp -a -v fixtures/kiro/generators/* ~/.kiro/generators/
$KG generate --global
if uname | grep -q Linux; then
  cargo deb
  sudo dpkg -i ./target/debian/kiro-generator_*.deb
  /usr/bin/kg --version
  /usr/bin/kg validate --global
  ls -lR /usr/share/doc/kiro-generator
  ls -l /etc/kg
  ls -l /usr/lib/systemd/user/kiro*
fi
