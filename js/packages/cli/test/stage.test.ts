import { access, mkdir, mkdtemp, readFile, rm, stat, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { x } from 'tinyexec'
import { describe, expect, it } from 'vitest'

const targets = [
  ['darwin-arm64', 'auv-aarch64-apple-darwin.tar.gz', 'auv-binding.darwin-arm64.node', 'auv'],
  ['darwin-x64', 'auv-x86_64-apple-darwin.tar.gz', 'auv-binding.darwin-x64.node', 'auv'],
  ['linux-arm64-gnu', 'auv-aarch64-unknown-linux-gnu.tar.gz', 'auv-binding.linux-arm64-gnu.node', 'auv'],
  ['linux-x64-gnu', 'auv-x86_64-unknown-linux-gnu.tar.gz', 'auv-binding.linux-x64-gnu.node', 'auv'],
  ['win32-x64-msvc', 'auv-x86_64-pc-windows-msvc.zip', 'auv-binding.win32-x64-msvc.node', 'auv.exe'],
] as const

async function createArchive(
  fixture: string,
  archive: string,
  executable: string,
  packageDir: string,
  kind: 'notarized' | 'release',
): Promise<void> {
  const artifactDir = join(fixture, kind === 'notarized' ? `notarized-${archive}` : archive)

  const payloadDir = join(artifactDir, 'payload')
  const archivePath = join(artifactDir, archive)
  await mkdir(payloadDir, { recursive: true })
  await writeFile(join(payloadDir, executable), `${kind}:${packageDir}`)

  if (archive.endsWith('.zip')) {
    await x('zip', ['-j', archivePath, join(payloadDir, executable)])
  }
  else {
    await x('tar', ['-czf', archivePath, '-C', payloadDir, executable])
  }
}

describe('', () => {
  it('platform staging verifies archives and adds sidecars beside NAPI bindings', async (t) => {
    if (process.platform === 'win32') {
      t.skip('release assembly runs on Ubuntu')
    }

    const fixture = await mkdtemp(join(tmpdir(), 'auv-cli-artifacts-'))
    const createdBindings: string[] = []

    t.onTestFinished(async () => {
      await rm(fixture, { force: true, recursive: true })
      await rm(new URL('../LICENSE', import.meta.url), { force: true })
      for (const [packageDir, , binding, executable] of targets) {
        await rm(new URL(`../npm/${packageDir}/bin/${executable}`, import.meta.url), {
          force: true,
        })
        await rm(new URL(`../npm/${packageDir}/LICENSE`, import.meta.url), {
          force: true,
        })
        if (createdBindings.includes(binding)) {
          await rm(new URL(`../npm/${packageDir}/${binding}`, import.meta.url), {
            force: true,
          })
        }
      }
    })

    for (const [packageDir, archive, binding, executable] of targets) {
      const bindingPath = new URL(`../npm/${packageDir}/${binding}`, import.meta.url)
      try {
        await access(bindingPath)
      }
      catch {
        await writeFile(bindingPath, 'NAPI fixture')
        createdBindings.push(binding)
      }

      await createArchive(fixture, archive, executable, packageDir, 'release')
      if (packageDir.startsWith('darwin')) {
        await createArchive(fixture, archive, executable, packageDir, 'notarized')
      }
    }

    await x('tsx', [new URL('../scripts/stage-platform-packages.ts', import.meta.url).pathname, fixture], { nodeOptions: { env: { ...process.env, AUV_RELEASE_TAG: 'v0.0.6' } } })

    for (const [packageDir, , , executable] of targets) {
      const sidecar = new URL(`../npm/${packageDir}/bin/${executable}`, import.meta.url)
      const expectedPrefix = packageDir.startsWith('darwin') ? 'notarized' : 'release'
      expect(await readFile(sidecar, 'utf8')).toBe(`${expectedPrefix}:${packageDir}`)
      expect((await stat(sidecar)).mode & 0o111).not.toBe(0)
      expect(await readFile(new URL(`../npm/${packageDir}/LICENSE`, import.meta.url), 'utf8')).toBe(await readFile(new URL('../../../../LICENSE', import.meta.url), 'utf8'))
    }
  })
})
