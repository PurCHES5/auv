import { describe, expect, inject, it } from 'vitest'

import { connect, createAuv } from '../../web/index'

const daemon = inject('auvBrowserDaemon')

describe('typed remote invoke from a browser', () => {
  it('rejects unauthenticated requests to the remote HTTP API', async () => {
    const connection = await connect({ endpoint: daemon.endpoint, transport: 'http' })

    try {
      const auv = createAuv(connection).runner({ runnerClass: 'auv.core.local' })
      await expect(auv.displays.list()).rejects.toMatchObject({
        name: 'AuvHttpError',
        status: 401,
      })
    }
    finally {
      await connection.close()
    }
  })

  it('invokes a real Driver capability through browser fetch', async () => {
    const connection = await connect({
      credential: daemon.credential,
      endpoint: daemon.endpoint,
      transport: 'http',
    })

    try {
      const auv = createAuv(connection).runner({ runnerClass: 'auv.core.local' })
      const displays = await auv.displays.list()
      expect(displays.length).toBeGreaterThan(0)
      expect(displays.every(display => display.displayId.length > 0)).toBe(true)
    }
    finally {
      await connection.close()
    }
  }, 600_000)

  it('streams a real Driver capability through browser WebSocket', async () => {
    const connection = await connect({
      credential: daemon.credential,
      endpoint: daemon.endpoint,
      transport: 'http',
    })

    try {
      const auv = createAuv(connection).runner({ runnerClass: 'auv.core.local' })
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
    }
    finally {
      await connection.close()
    }
  })
})
