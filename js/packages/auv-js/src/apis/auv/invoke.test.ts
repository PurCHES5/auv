import type { Transport } from '../../transport/types'

import { describe, expect, it } from 'vitest'

import {
  ListDisplaysRequestSchema,
  ListDisplaysResponseSchema,
} from '../../gen/auv/api/driver/v1/display_pb'
import {
  MoveMouseRequestSchema,
  MoveMouseStreamResponseSchema,
} from '../../gen/auv/api/driver/v1/input_pb'
import {
  AuvConfigurationError,
  connect,
  createPairingToken,
  invokeServerStream,
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

describe.skipIf(process.platform === 'win32')('invoke against an authenticated AUV daemon', () => {
  it('pairs a Device and invokes a real Runner through the remote HTTP API', async () => {
    const daemon = await setupAuvDaemon()

    try {
      const owner = await connect({ endpoint: daemon.ownerSocket, local: true, transport: 'unix' })
      const token = await createPairingToken(owner, { ttlMs: 60_000 })
      await owner.close()

      const bootstrap = await connect({ endpoint: daemon.remoteEndpoint, transport: 'http' })
      const enrollment = await pairDevice(bootstrap, { deviceId: 'auv-js-integration', label: 'auv-js integration test', token })
      await bootstrap.close()

      const unauthenticated = await connect({ endpoint: daemon.remoteEndpoint, transport: 'http' })
      await expect(listDisplays(unauthenticated)).rejects.toMatchObject({ name: 'AuvHttpError', status: 401 })
      await unauthenticated.close()

      const paired = await connect({ credential: enrollment.credential, endpoint: daemon.remoteEndpoint, transport: 'http' })
      {
        const response = await listDisplays(paired)
        expect(response.displays.every(display => display.displayId.length > 0)).toBe(true)
        expect(response.displays.length).toBeGreaterThan(1)
      }
      {
        const streamed = []

        for await (const item of await moveMouse(paired))
          streamed.push(item)

        expect(streamed.length).toBeGreaterThan(1)
        const events = streamed.map(item => item.event.case)
        expect(events[0]).toBe('started')
        expect(events).toContain('progress')
        expect(events.at(-1)).toBe('completed')
      }

      await paired.close()
    }
    finally {
      await daemon.stop()
    }
  }, 600_000)
})

function listDisplays(connection: Awaited<ReturnType<typeof connect>>) {
  return invokeUnary(connection, {
    input: ListDisplaysRequestSchema,
    method: 'ListDisplays',
    output: ListDisplaysResponseSchema,
    request: {},
    runnerClass: 'auv.core.local',
    service: 'auv.api.driver.v1.DisplayService',
  })
}

function moveMouse(connection: Awaited<ReturnType<typeof connect>>) {
  return invokeServerStream(connection, {
    input: MoveMouseRequestSchema,
    method: 'MoveMouse',
    output: MoveMouseStreamResponseSchema,
    request: {
      plan: {
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
      },
    },
    runnerClass: 'auv.core.local',
    service: 'auv.api.driver.v1.InputService',
  })
}
