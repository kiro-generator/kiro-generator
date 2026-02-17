import { type Clippy, createRustWorkflow, Os } from '@dougefresh/ci';

export default function () {
  const clippy: Partial<Clippy> = {
    matrix: {
      os: [Os.LINUX_ARM64],
      toolchains: ['stable'],
      features: ['default'],
    },
  };
  if (process.env.GITHUB_REF_NAME && process.env.GITHUB_REF.includes('main')) {
    clippy.matrix?.os.push(Os.LINUX_AMD64, Os.MAC);
  }
  return createRustWorkflow()
    .enableMdBook()
    .disableDocCheck()
    .clippy(clippy)
    .withRelease({
      debian: true,
      bin: true,
      publish: false,
      profile: 'release',
      os: [Os.LINUX_ARM64, Os.LINUX_AMD64, Os.MAC],
      homebrew: {
        if: true,
        repo: 'homebrew-kiro-generator',
      },
    })
    .semver(false)
    .disableSanitizers()
    .extra('cli-test', 'bash ./scripts/test-ci.sh', {
      cargoTools: ['cargo-deb'],
    })

    .build();
}
