import { type Clippy, type Coverage, createRustWorkflow, Os } from '@dougefresh/ci';

export default function () {
  const disabled: boolean = process.env._GIT_MSG?.includes('[ci:disable]') || false;
  const clippy: Partial<Clippy> = {
    if: !disabled,
    matrix: {
      os: [Os.LINUX_ARM64],
      toolchains: ['stable'],
      features: ['default'],
    },
  };

  const coverage: Partial<Coverage> = {
    if: !disabled,
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
  let pipeline = createRustWorkflow()
    .enableMdBook()
    .disableDocCheck()
    .clippy(clippy)
    .coverage(coverage)
    .withRelease({
      debian: true,
      bin: {
        name: 'kg',
        linux: {
          arm64: true,
          amd64: true,
        },
        mac: true,
        win: false,
      },
      publish: false,
      profile: 'release',
      assets: [
        {
          glob: 'schemas/*.json',
          archiveName: 'schemas.tar.gz',
        },
        {
          glob: 'resources/kg-helper/**',
          archiveName: 'skill.tar.gz',
        },
      ],
    })
    .semver(false)
    .disableSanitizers()
    .extraJob('kg-bin-test', {
      if: !disabled,
      run: 'bash ./scripts/test-kg-bin-ci.sh',
      cache: {
        cargoTools: ['cargo-deb'],
      },
      matrix: {
        os: [Os.LINUX_AMD64],
        toolchains: ['stable'],
        features: ['default'],
      },
    });

  if (disabled) {
    return pipeline
      .disableAi()
      .disableCargoSort()
      .disableDocCheck()
      .disableHack()
      .disableDependencies()
      .disableFmt()
      .build();
  }
  return pipeline.build();
}
