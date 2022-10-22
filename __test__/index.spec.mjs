import test from 'ava'
import fs from 'fs'

import { Inotify, WatchMask, EventFlags } from '../dist/index.js'

test('inotify', async (t) => {
  const i = new Inotify()
  const wd = i.watch('.', WatchMask.IN_ALL_EVENTS)
  i.on('event', (path, event, flags) => {
    t.is(path, 'Cargo.toml')
    t.is(event & WatchMask.IN_ACCESS, 1)
  })

  console.log('start listening for events')
  fs.promises.readFile('./Cargo.toml')
  await new Promise((r) => setTimeout(r, 5000))
  console.log('stop listening for events')
  i.unwatch(wd)
  i.close()
  t.throws(() => i.watch('.', WatchMask.IN_ALL_EVENTS))
  await new Promise((r) => setTimeout(r, 50000))
  t.truthy(i)
})
