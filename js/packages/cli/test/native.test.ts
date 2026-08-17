import { readFile } from 'node:fs/promises'

import { expect, it } from 'vitest'

import { nativePackageVersion } from '../src-js'

it('the NAPI binding and JavaScript package versions match', async () => {
  const manifest = JSON.parse(
    await readFile(new URL('../package.json', import.meta.url), 'utf8'),
  ) as { version: string }

  expect(nativePackageVersion()).toBe(manifest.version)
})
