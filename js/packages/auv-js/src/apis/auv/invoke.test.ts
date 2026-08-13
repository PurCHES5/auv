import type { Transport } from '../../transport/types'

import { afterAll, describe, expect, it } from 'vitest'

import {
  ListDisplaysRequestSchema,
  ListDisplaysResponseSchema,
} from '../../gen/auv/api/driver/v1/display_pb'
import {
  AuvConfigurationError,
  connect,
  createAuv,
  createPairingToken,
  invokeUnary,
  pairDevice,
} from '../../node/index'
import { setupAuvDaemon } from '../../tutils/auv-daemon'

describe('typed remote invoke', () => {
  it('rejects explicit Device placement on a local-only connection before dispatch', async () => {
    let dispatched = false
    const transport: Transport = {
      close() {},
      async connect() {},
      async duplex() { throw new Error('unexpected dispatch') },
      async unary() {
        dispatched = true
        throw new Error('unexpected dispatch')
      },
    }
    const connection = await connect({ local: true, transport })

    await expect(invokeUnary(connection, {
      deviceId: 'remote-device',
      input: ListDisplaysRequestSchema,
      method: 'ListDisplays',
      output: ListDisplaysResponseSchema,
      request: {},
      runnerClass: 'auv.core.local',
      service: 'auv.api.driver.v1.DisplayService',
    })).rejects.toEqual(new AuvConfigurationError('local connection cannot select an explicit Device'))
    expect(dispatched).toBe(false)
  })
})

describe.skipIf(process.platform === 'win32')('invoke against an authenticated AUV daemon', async () => {
  const daemon = await setupAuvDaemon()

  const owner = await connect({ endpoint: daemon.ownerSocket, local: true, transport: 'unix' })
  const token = await createPairingToken(owner, { ttlMs: 60_000 })
  await owner.close()

  const bootstrap = await connect({ endpoint: daemon.remoteEndpoint, transport: 'http' })
  const enrollment = await pairDevice(bootstrap, { deviceId: 'auv-js-integration', label: 'auv-js integration test', token })
  await bootstrap.close()

  afterAll(async () => {
    await daemon.stop()
  })

  it('rejects unauthenticated requests to the remote HTTP API', async () => {
    const unauthenticated = await connect({ endpoint: daemon.remoteEndpoint, transport: 'http' })
    const auv = createAuv(unauthenticated).runner({ runnerClass: 'auv.core.local' })
    await expect(auv.displays.list()).rejects.toMatchObject({ name: 'AuvHttpError', status: 401 })
    await unauthenticated.close()
  })

  it('pairs a Device and invokes a real Runner through the remote HTTP API', async () => {
    const paired = await connect({ credential: enrollment.credential, endpoint: daemon.remoteEndpoint, transport: 'http' })

    {
      const auv = createAuv(paired).runner({ runnerClass: 'auv.core.local' })
      const displays = await auv.displays.list()
      expect(displays.every(display => display.displayId.length > 0)).toBe(true)
      expect(displays.length).toBeGreaterThan(1)
    }

    await paired.close()
  }, 600_000)

  it('pairs a Device and invokes mouseMove to the remote HTTP API', async () => {
    const paired = await connect({ credential: enrollment.credential, endpoint: daemon.remoteEndpoint, transport: 'http' })

    const auv = createAuv(paired).runner({ runnerClass: 'auv.core.local' })
    const streamed = []

    for await (const item of await auv.input.moveMouse({
      curve: {
        segments: [{
          control1: { x: 0, y: 0 },
          control2: { x: 0, y: 0 },
          end: { x: 0, y: 0 },
        }],
        start: { x: 0, y: 0 },
      },
      mapping: { height: 1, width: 1 },
      options: {
        duration: { nanos: 50_000_000, seconds: 0n },
        sampleRateHz: 60,
      },
      start: { source: { case: 'current', value: {} } },
    }))
      streamed.push(item)

    expect(streamed.length).toBeGreaterThan(1)
    const events = streamed.map(item => item.event.case)
    expect(events[0]).toBe('started')
    expect(events).toContain('progress')
    expect(events.at(-1)).toBe('completed')

    await paired.close()
  })
})
