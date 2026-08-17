import type { TestContext } from 'vitest'

import { spawnSync } from 'node:child_process'
import {
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  realpath,
  rm,
  writeFile,
} from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { pathToFileURL } from 'node:url'

import { expect, it } from 'vitest'

const packageVersion = '0.0.6'

interface DiagnosticReport {
  header?: {
    glibcVersionRuntime?: string
  }
}

interface FixtureOptions {
  executableContents?: string
  platformPackage?: boolean
}

it('binaryPath resolves the executable from the host optional package', async (t) => {
  if (isMusl())
    t.skip('the published Linux target requires glibc')

  const fixture = await createFixture(t, { platformPackage: true })
  const result = runBinaryPath(fixture)

  expect(result.status).toBe(0)
  expect(result.stderr).toBe('')
  expect(result.stdout.trim()).toBe(
    await realpath(join(fixture, 'node_modules', packageName(), executablePath())),
  )
})

it('binaryPath explains when optional dependencies were omitted', async (t) => {
  if (isMusl())
    t.skip('the published Linux target requires glibc')

  const fixture = await createFixture(t)
  const result = runBinaryPath(fixture)

  expect(result.status).toBe(1)
  expect(result.stderr).toContain('AuvBinaryError')
  expect(result.stderr).toContain('without omitting optional dependencies')
})

it('the auv shim forwards arguments and exit status', async (t) => {
  if (process.platform === 'win32' || isMusl()) {
    t.skip('the fixture executable is a POSIX shell script')
  }

  const fixture = await createFixture(t, {
    executableContents: '#!/bin/sh\nprintf "sidecar:%s\\n" "$*"\nexit 23\n',
    platformPackage: true,
  })
  await mkdir(join(fixture, 'bin'))
  await copyFile(
    new URL('../bin/auv.js', import.meta.url),
    join(fixture, 'bin', 'auv.js'),
  )

  const result = spawnSync(
    process.execPath,
    [join(fixture, 'bin', 'auv.js'), 'serve', '--no-discovery'],
    { encoding: 'utf8' },
  )

  expect(result.status).toBe(23)
  expect(result.stdout).toBe('sidecar:serve --no-discovery\n')
  expect(result.stderr).toBe('')
})

async function createFixture(
  t: TestContext,
  { executableContents = 'auv fixture', platformPackage = false }: FixtureOptions = {},
): Promise<string> {
  const fixture = await mkdtemp(join(tmpdir(), 'auv-cli-test-'))
  t.onTestFinished(() => rm(fixture, { force: true, recursive: true }))

  await mkdir(join(fixture, 'dist'))
  await copyFile(
    new URL('../dist/binary.js', import.meta.url),
    join(fixture, 'dist', 'binary.js'),
  )
  await writeFile(
    join(fixture, 'package.json'),
    JSON.stringify({ name: '@auv-js/cli', type: 'module', version: packageVersion }),
  )

  if (platformPackage) {
    const packageRoot = join(fixture, 'node_modules', packageName())
    const executable = join(packageRoot, executablePath())
    await mkdir(dirname(executable), { recursive: true })
    await writeFile(
      join(packageRoot, 'package.json'),
      JSON.stringify({ name: packageName(), version: packageVersion }),
    )
    await writeFile(executable, executableContents)
    await chmod(executable, 0o755)
  }

  return fixture
}

function executablePath(): string {
  return process.platform === 'win32' ? 'bin/auv.exe' : 'bin/auv'
}

function isMusl(): boolean {
  const report = process.report?.getReport?.() as DiagnosticReport | undefined
  return (
    process.platform === 'linux'
    && report?.header?.glibcVersionRuntime === undefined
  )
}

function packageName(): string {
  const packages: Record<string, string> = {
    'darwin-arm64': '@auv-js/cli-darwin-arm64',
    'darwin-x64': '@auv-js/cli-darwin-x64',
    'linux-arm64': '@auv-js/cli-linux-arm64-gnu',
    'linux-x64': '@auv-js/cli-linux-x64-gnu',
    'win32-x64': '@auv-js/cli-win32-x64-msvc',
  }
  const packageName = packages[`${process.platform}-${process.arch}`]
  if (!packageName) {
    throw new Error(`No test package for ${process.platform}/${process.arch}`)
  }
  return packageName
}

function runBinaryPath(fixture: string) {
  const script = `
    const { binaryPath } = await import(process.argv[1])
    console.log(binaryPath())
  `
  return spawnSync(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      script,
      pathToFileURL(join(fixture, 'dist', 'binary.js')).href,
    ],
    {
      encoding: 'utf8',
      env: { ...process.env, NODE_OPTIONS: '', NODE_PATH: '' },
    },
  )
}
