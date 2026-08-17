import type { Buffer } from 'node:buffer'

import process from 'node:process'

import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import {
  chmod,
  copyFile,
  mkdir,
  readdir,
  readFile,
  rename,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises'
import { basename, dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const cliRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const repositoryRoot = resolve(cliRoot, '../../..')

interface PackageManifest {
  files?: string[]
  main: string
  name: string
  version: string
}

interface Target {
  archive: string
  executable: string
  packageDir: string
}

const targets: readonly Target[] = [
  {
    archive: 'auv-aarch64-apple-darwin.tar.gz',
    executable: 'auv',
    packageDir: 'darwin-arm64',
  },
  {
    archive: 'auv-x86_64-apple-darwin.tar.gz',
    executable: 'auv',
    packageDir: 'darwin-x64',
  },
  {
    archive: 'auv-aarch64-unknown-linux-gnu.tar.gz',
    executable: 'auv',
    packageDir: 'linux-arm64-gnu',
  },
  {
    archive: 'auv-x86_64-unknown-linux-gnu.tar.gz',
    executable: 'auv',
    packageDir: 'linux-x64-gnu',
  },
  {
    archive: 'auv-x86_64-pc-windows-msvc.zip',
    executable: 'auv.exe',
    packageDir: 'win32-x64-msvc',
  },
]

const artifactsRoot = resolve(process.argv[2] ?? join(cliRoot, 'artifacts'))
const rootManifest = JSON.parse(
  await readFile(join(cliRoot, 'package.json'), 'utf8'),
) as PackageManifest

verifyReleaseVersion(rootManifest.version)

const artifactFiles = await collectFiles(artifactsRoot)
for (const target of targets) {
  await stageTarget(target, artifactFiles, rootManifest.version)
}

await copyFile(join(repositoryRoot, 'LICENSE'), join(cliRoot, 'LICENSE'))
console.info(`Staged ${targets.length} AUV executables from ${artifactsRoot}`)

async function collectFiles(root: string): Promise<string[]> {
  const entries = await readdir(root, { withFileTypes: true })
  const nested = await Promise.all(
    entries.map(async (entry) => {
      const path = join(root, entry.name)
      return entry.isDirectory() ? collectFiles(path) : [path]
    }),
  )
  return nested.flat()
}

function extractExecutable(archive: string, executable: string): Buffer {
  const options = { encoding: 'buffer' as const, maxBuffer: 512 * 1024 * 1024 }
  if (archive.endsWith('.zip')) {
    return execFileSync('unzip', ['-p', archive, executable], options)
  }
  return execFileSync('tar', ['-xOf', archive, executable], options)
}

async function isFile(path: string): Promise<boolean> {
  try {
    return (await stat(path)).isFile()
  }
  catch (error) {
    if (
      error !== null
      && typeof error === 'object'
      && 'code' in error
      && error.code === 'ENOENT'
    ) {
      return false
    }
    throw error
  }
}

function preferNotarized(candidates: readonly string[]): string | undefined {
  return candidates.toSorted((left, right) => {
    const leftNotarized = left.includes('notarized-') ? 1 : 0
    const rightNotarized = right.includes('notarized-') ? 1 : 0
    return rightNotarized - leftNotarized
  })[0]
}

async function stageTarget(
  target: Target,
  artifactFiles: readonly string[],
  packageVersion: string,
): Promise<void> {
  const candidates = artifactFiles.filter(
    file => basename(file) === target.archive,
  )
  const archive = preferNotarized(candidates)
  if (!archive) {
    throw new Error(`Missing release archive ${target.archive}`)
  }

  await verifyChecksum(archive)

  const packageRoot = join(cliRoot, 'npm', target.packageDir)
  const manifestPath = join(packageRoot, 'package.json')
  const manifest = JSON.parse(
    await readFile(manifestPath, 'utf8'),
  ) as PackageManifest
  if (manifest.version !== packageVersion) {
    throw new Error(
      `${manifest.name} is ${manifest.version}, but @auv-js/cli is ${packageVersion}`,
    )
  }

  const bindingPath = join(packageRoot, manifest.main)
  if (!(await isFile(bindingPath))) {
    throw new Error(
      `Missing NAPI artifact ${manifest.main}; run napi artifacts before staging AUV executables`,
    )
  }

  const relativeExecutable = `bin/${target.executable}`
  if (!manifest.files?.includes(relativeExecutable)) {
    manifest.files = [...new Set([...(manifest.files ?? []), relativeExecutable])]
    await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)
  }

  const executable = extractExecutable(archive, target.executable)
  const destination = join(packageRoot, relativeExecutable)
  const temporary = `${destination}.tmp`
  await mkdir(dirname(destination), { recursive: true })
  try {
    await writeFile(temporary, executable, { mode: 0o755 })
    await chmod(temporary, 0o755)
    await rename(temporary, destination)
  }
  finally {
    await rm(temporary, { force: true })
  }

  await copyFile(join(repositoryRoot, 'LICENSE'), join(packageRoot, 'LICENSE'))
}

async function verifyChecksum(archive: string): Promise<void> {
  const checksumPath = `${archive}.sha256`
  const expected = (await readFile(checksumPath, 'utf8')).trim().split(/\s+/u)[0]
  if (!/^[a-f\d]{64}$/iu.test(expected)) {
    throw new Error(`Invalid SHA-256 file ${checksumPath}`)
  }

  const actual = createHash('sha256')
    .update(await readFile(archive))
    .digest('hex')
  if (actual.toLowerCase() !== expected.toLowerCase()) {
    throw new Error(`SHA-256 mismatch for ${archive}`)
  }
}

function verifyReleaseVersion(packageVersion: string): void {
  const releaseTag = process.env.AUV_RELEASE_TAG
  if (!releaseTag)
    return

  const tagVersion = releaseTag.replace(/^v/u, '')
  if (tagVersion !== packageVersion) {
    throw new Error(
      `Release tag ${releaseTag} does not match @auv-js/cli ${packageVersion}`,
    )
  }
}
