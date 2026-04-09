#!/bin/bash
#

jq --rawfile p resources/agents/prompt.md '.prompt = $p' resources/agents/kg-helper.json >/tmp/.$$.json
mv /tmp/.$$.json ./resources/agents/kg-helper.json
