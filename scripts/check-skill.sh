#!/bin/bash
set -euo pipefail
(cd resources/kg-helper && find . -type f | xargs sha256sum) >docs/.well-known/agent-skills/skill.sha256
cd docs/.well-known/agent-skills/
cat skill.sha256
tar -xzf ./kg-helper.tar.gz
sha256sum --check ./skill.sha256
tarSha="$(jq -r '.skills[0].digest' index.json | cut -d : -f 2)"
echo "${tarSha} kg-helper.tar.gz" >skill.sha256
sha256sum --check skill.sha256
