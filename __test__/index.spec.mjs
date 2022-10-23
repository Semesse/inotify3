import test from 'ava'
import fs from 'fs'

import { Inotify, WatchMask, EventFlags } from '../dist/index.js'

test('inotify', async (t) => {
  const i = new Inotify()
  i.once('event', (path, event, flags) => {
    console.log(path, event, flags)
    t.is(path, 'Cargo.toml')
    t.is(event & WatchMask.IN_ACCESS, 1)
  })
  i.on('error', (err) => {
    console.error(err)
  })

  console.log('start listening for events')
  const wd = i.watch('.', WatchMask.IN_ACCESS)

  fs.promises.readFile('./Cargo.toml')
  await new Promise((r) => setTimeout(r, 1000))

  console.log('stop listening for events')
  i.unwatch(wd)

  console.log('close inotify instance')
  i.close()
  t.throws(() => i.watch('.', WatchMask.IN_ALL_EVENTS), { instanceOf: Error })
})
