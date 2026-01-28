import { Arch, createRustWorkflow } from '@dougefresh/ci';

const prompt = `

## kiro Documentation

There are additional markdown documents about kiro-cli configuration in directory ./docs/kiro
If you need more context around kiro agent config refer to these files.


## Facet Documentation

Face is new and evolving. You will find updated documentation in directory ./docs/facet
If unclear how the crate / library works, refer to these documents.


`;
export default function () {
  return createRustWorkflow()
    .withRelease({
      debian: true,
      bin: true,
      publish: false,
      profile: 'release',
      os: [Arch.AMD64],
    })
    .semver(false)
    .additionalPrompt(prompt)
    .disableSanitizers()
    .extra('cli-test', 'bash -x ./scripts/test-ci.sh')
    .build();
}
