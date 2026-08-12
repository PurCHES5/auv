import type { AuvConnection, ConnectOptions } from '../transport/connection'
import type { Transport } from '../transport/types'

import {

  connectTransport,
} from '../transport/connection'
import { createHttpTransport } from '../web/http'
import { createGrpcTransport, createUnixSocketTransport } from './grpc'

const DEFAULT_HTTP_ENDPOINT = 'http://127.0.0.1:9847'

/** Connects with HTTP, gRPC, Unix socket, or a caller-provided transport. */
export async function connect(options: ConnectOptions = {}): Promise<AuvConnection> {
  return connectTransport(resolveTransport(options), options)
}

function resolveTransport(options: ConnectOptions): Transport {
  if (options.transport !== undefined && typeof options.transport !== 'string') {
    return options.transport
  }
  switch (options.transport ?? 'http') {
    case 'http':
      return createHttpTransport({ endpoint: options.endpoint ?? DEFAULT_HTTP_ENDPOINT })
    case 'grpc':
      return createGrpcTransport({ endpoint: options.endpoint })
    case 'unix':
      // TODO(js-unix-discovery): select a default socket only when the daemon
      // publishes a platform-defined owner socket location for SDK consumers.
      if (options.endpoint === undefined)
        throw new Error('unix transport requires an endpoint path')
      return createUnixSocketTransport({ path: options.endpoint })
  }
}
