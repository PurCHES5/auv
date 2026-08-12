import type { Result } from 'tinyexec'

import type { AuvConnection } from '../node/index'

import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { Format, LogLevel, setGlobalFormat, setGlobalLogLevel, useLogg } from '@guiiai/logg'
import { x } from 'tinyexec'

import { connect, createPairingToken, listDevices, pairDevice } from '../node/index'
import { repositoryRoot } from './dir'

setGlobalFormat(Format.Pretty)
setGlobalLogLevel(LogLevel.Debug)

export interface AuvDaemonFixture {
  readonly ownerSocket: string
  readonly remoteEndpoint: string
  stop: () => Promise<void>
}

export interface PairedAuvDaemonFixture extends AuvDaemonFixture {
  readonly connection: AuvConnection
  readonly credential: string
  readonly deviceId: string
  readonly localDeviceId: string
}

/** Starts an isolated AUV daemon with local-owner and paired-bearer listeners. */
export async function setupAuvDaemon(): Promise<AuvDaemonFixture> {
  const log = useLogg('setup:auv-daemon').useGlobalConfig()

  const workspace = await repositoryRoot()
  const root = await mkdtemp(join(tmpdir(), 'auv-js-daemon-'))
  const ownerSocket = join(root, 'auv.sock')

  log.withFields({ root }).log('starting AUV daemon for testing')

  const daemon = x(join(workspace, 'target', 'debug', 'auv'), [
    'serve',
    '--listen',
    `unix://${ownerSocket}`,
    '--listen',
    'http://127.0.0.1:0',
    '--pairing-store',
    join(root, 'pairings.json'),
    '--store-root',
    join(root, 'store'),
    '--no-discovery',
  ], {
    nodeOptions: { cwd: workspace },
  })

  log.withFields({ ownerSocket }).log('AUV daemon started for testing')

  const readiness = daemonReadiness(daemon)

  const stop = async () => {
    daemon.kill('SIGINT')
    await daemon
    await readiness.output
    await rm(root, { force: true, recursive: true })

    log.withFields({ ownerSocket }).log('AUV daemon stopped')
  }

  try {
    const remoteEndpoint = await readiness.endpoint
    log.withFields({ endpoint: remoteEndpoint, ownerSocket }).log('AUV daemon ready for testing')

    return { ownerSocket, remoteEndpoint, stop }
  }
  catch (error) {
    await stop()
    throw error
  }
}

/** Starts an isolated daemon and returns an authenticated paired-Device connection. */
export async function setupPairedAuvDaemon(deviceId = 'auv-js-test-device'): Promise<PairedAuvDaemonFixture> {
  const log = useLogg('setup:auv-daemon-pairing').useGlobalConfig()

  const daemon = await setupAuvDaemon()
  try {
    log.debug('connecting to started AUV daemon for pairing and enrollment')
    const owner = await connect({ endpoint: daemon.ownerSocket, local: true, transport: 'unix' })

    log.log('creating pairing token as owner')
    const token = await createPairingToken(owner)
    log.withFields({ deviceId, token: redacted(token.value) }).log('enrolled successfully')

    await owner.close()

    log.withField('endpoint', daemon.remoteEndpoint).debug('connecting to started AUV daemon and pair for test Device')
    const bootstrap = await connect({ endpoint: daemon.remoteEndpoint, transport: 'http' })

    log.log('paring test Device')
    const enrollment = await pairDevice(bootstrap, { deviceId, label: 'auv-js test Device', token })
    log.log('paired successfully')

    await bootstrap.close()

    log.withField('endpoint', daemon.remoteEndpoint).debug('connecting to started AUV daemon with test Device only credential')
    const connection = await connect({ credential: enrollment.credential, endpoint: daemon.remoteEndpoint, transport: 'http' })

    log.withField('endpoint', daemon.remoteEndpoint).debug('testing with list devices')
    const localDevice = (await listDevices(connection)).find(device => device.local)
    if (localDevice === undefined)
      throw new Error('AUV daemon fixture did not expose its local Device')

    return {
      ...daemon,
      connection,
      credential: enrollment.credential,
      deviceId: enrollment.deviceId,
      localDeviceId: localDevice.id,
      async stop() {
        await connection.close()
        await daemon.stop()
      },
    }
  }
  catch (error) {
    await daemon.stop()
    throw error
  }
}

function daemonReadiness(daemon: Result): { endpoint: Promise<string>, output: Promise<string> } {
  let resolveEndpoint!: (endpoint: string) => void
  let rejectEndpoint!: (error: Error) => void
  const endpoint = new Promise<string>((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('AUV daemon readiness timed out')), 10_000)
    resolveEndpoint = (value) => {
      clearTimeout(timeout)
      resolve(value)
    }
    rejectEndpoint = (error) => {
      clearTimeout(timeout)
      reject(error)
    }
  })

  const output = (async () => {
    let collected = ''
    for await (const line of daemon) {
      collected += `${line}\n`
      const match = line.match(/auv serve: (http:\/\/\S+)/u)
      if (match !== null)
        resolveEndpoint(match[1]!)
    }
    if (daemon.exitCode !== undefined && daemon.exitCode !== 0)
      rejectEndpoint(new Error(`AUV daemon exited before readiness:\n${collected}`))
    return collected
  })()

  return { endpoint, output }
}
function redacted(str: string, startPad: number = 4, endPad: number = 0) {
  const maskLength = Math.max(0, str.length - startPad - endPad)

  return (
    str.slice(0, startPad)
    + '•'.repeat(maskLength)
    + (endPad > 0 ? str.slice(-endPad) : '')
  )
}
