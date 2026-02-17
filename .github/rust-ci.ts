import { type Clippy, type Coverage, createRustWorkflow, Os } from '@dougefresh/ci';

export default function () {
  const clippy: Partial<Clippy> = {
    matrix: {
      os: [Os.LINUX_ARM64],
      toolchains: ['stable'],
      features: ['default'],
    },
  };

  const coverage: Partial<Coverage> = {
    matrix: {
      os: [Os.LINUX_ARM64],
      toolchains: ['stable'],
      features: ['default'],
    },
  };
  if (process.env.GITHUB_REF?.includes('main') || process.env.CONTEXT?.includes('ALL_OS')) {
    clippy.matrix?.os.push(Os.LINUX_AMD64, Os.MAC);
    coverage.matrix?.os.push(Os.LINUX_AMD64, Os.MAC);
  }
  return createRustWorkflow()
    .enableMdBook()
    .disableDocCheck()
    .clippy(clippy)
    .coverage(coverage)
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
    .extraJob('kg-bin-test', {
      run: 'bash ./scripts/test-kg-bin-ci.sh',
      cache: {
        cargoTools: ['cargo-deb'],
      },
      matrix: {
        os: [Os.LINUX_AMD64],
        toolchains: ['stable'],
        features: ['default'],
      },
    })
    .build();
}
