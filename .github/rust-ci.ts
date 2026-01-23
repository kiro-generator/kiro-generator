import { createRustWorkflow } from '@dougefresh/ci';
const prompt = `

## kiro Documentation

There are additional makrdown documents about kiro-cli configuration in directory ./docs/kiro
If you need more context around kiro agent config refer to these files.


## Facet Documentation

Face is new and evolving. You will find updated documentation in directory ./docs/facet
If unclear how the crate / library works, refer to these documents.


`;
export default function () {
  return createRustWorkflow()
    .semver(false)
    .additionalPrompt(prompt)
    .disableSanitizers()
    .extra('cli-test', 'bash -x ./scripts/test-ci.sh')
    .build();
}
