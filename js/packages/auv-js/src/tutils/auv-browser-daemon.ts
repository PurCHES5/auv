import type { TestProject } from 'vitest/node'

import process from 'node:process'

import { setupPairedAuvDaemon } from './auv-daemon'

export type BrowserAuvDaemonContext
  = | { readonly available: false }
    | {
      readonly available: true
      readonly credential: string
      readonly endpoint: string
    }

declare module 'vitest' {
  export interface ProvidedContext {
    auvBrowserDaemon: BrowserAuvDaemonContext
  }
}

/** Starts and pairs the daemon before browser tests without bundling Node helpers. */
export async function setup(project: TestProject): Promise<() => Promise<void>> {
  if (process.platform === 'win32') {
    // TODO(windows-browser-fixture): pairing needs an owner-authenticated Unix
    // listener. Start a Windows fixture after an equivalent local owner
    // transport has an approved contract.
    project.provide('auvBrowserDaemon', { available: false })
    return () => Promise.resolve()
  }

  const daemon = await setupPairedAuvDaemon('auv-js-browser-integration')

  project.provide('auvBrowserDaemon', {
    available: true,
    credential: daemon.credential,
    endpoint: daemon.remoteEndpoint,
  })

  return daemon.stop
}
