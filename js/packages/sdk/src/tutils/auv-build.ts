import { Format, LogLevel, setGlobalFormat, setGlobalLogLevel, useLogg } from '@guiiai/logg'
import { x } from 'tinyexec'

import { repositoryRoot } from './dir'

setGlobalFormat(Format.Pretty)
setGlobalLogLevel(LogLevel.Debug)

export default async function setup(): Promise<void> {
  const log = useLogg('setup:auv-build').useGlobalConfig()

  const root = await repositoryRoot()
  log.withField('root', root).debug('building AUV CLI binary for testing')

  await x('cargo', ['build', '--quiet', '--package', 'auv-cli', '--bin', 'auv'], {
    nodeOptions: { cwd: root },
    throwOnError: true,
  })

  log.debug('AUV CLI binary built successfully')
}
