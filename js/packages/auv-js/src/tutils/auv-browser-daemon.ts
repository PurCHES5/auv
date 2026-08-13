import type { TestProject } from 'vitest/node'

import { setupPairedAuvDaemon } from './auv-daemon'

export interface BrowserAuvDaemonContext {
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
  const daemon = await setupPairedAuvDaemon('auv-js-browser-integration')

  project.provide('auvBrowserDaemon', {
    credential: daemon.credential,
    endpoint: daemon.remoteEndpoint,
  })

  return daemon.stop
}
