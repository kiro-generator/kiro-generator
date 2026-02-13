import { Arch, createRustWorkflow } from '@dougefresh/ci';

export default function () {
  return createRustWorkflow()
    .enableMdBook()
    .withRelease({
      debian: true,
      bin: true,
      publish: false,
      profile: 'release',
      os: [Arch.AMD64],
    })
    .semver(false)
    .disableSanitizers()
    .extra('cli-test', 'bash -x ./scripts/test-ci.sh', {
      cargoTools: ['cargo-deb'],
    })

    .build();
}
