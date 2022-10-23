import { EventEmitter } from 'events'
import { Inotify as NativeInotify, JsWatchDescriptor as WatchDescriptor } from '../lib'

/**
 * Set watch masks to register interested event types or set options for `watch()`.
 * See https://man7.org/linux/man-pages/man7/inotify.7.html
 */
export enum WatchMask {
  /**
   * File was accessed
   */
  IN_ACCESS = 0x00000001,

  /**
   * File or directory was modified
   */
  IN_MODIFY = 0x00000002,

  /**
   * File or directory attribute was changed, including
   * - permissions (e.g.,chmod(2))
   * - timestamps (e.g., utimensat(2))
   * - extended attributes (setxattr(2))
   * - link count (since Linux 2.6.25; e.g., for the target of link(2) and for unlink(2))
   * - user/group ID (e.g., chown(2))
   */
  IN_ATTRIB = 0x00000004,

  /**
   * File opened in writing mode was closed
   */
  IN_CLOSE_WRITE = 0x00000008,

  /**
   * File or directory not opened in writing mode was closed
   */
  IN_CLOSE_NOWRITE = 0x00000010,

  /**
   * File or directory was opened.
   */
  IN_OPEN = 0x00000020,

  /**
   * File or directory was moved out
   */
  IN_MOVED_FROM = 0x00000040,

  /**
   * File or directory was moved into watched directory
   */
  IN_MOVED_TO = 0x00000080,

  /**
   * File or directory was created
   * including links and UNIX sockets
   */
  IN_CREATE = 0x00000100,

  /**
   * File or child directory was deleted
   */
  IN_DELETE = 0x00000200,

  /**
   * Watched file or directory was deleted
   *
   * Cross filesystem move might trigger this type of event
   */
  IN_DELETE_SELF = 0x00000400,

  /**
   * Watched file or directory was moved
   */
  IN_MOVE_SELF = 0x00000800,

  /**
   * File or directory inside watched directory was moved (from or to)
   */
  IN_MOVE = IN_MOVED_FROM | IN_MOVED_TO,

  /**
   * File was closed
   */
  IN_CLOSE = IN_CLOSE_WRITE | IN_CLOSE_NOWRITE,

  /**
   * All kinds of events
   */
  IN_ALL_EVENTS = IN_ACCESS |
    IN_MODIFY |
    IN_ATTRIB |
    IN_CLOSE_WRITE |
    IN_CLOSE_NOWRITE |
    IN_OPEN |
    IN_MOVED_FROM |
    IN_MOVED_TO |
    IN_CREATE |
    IN_DELETE |
    IN_DELETE_SELF |
    IN_MOVE_SELF,

  /**
   * Watch given path only if it's a directory
   */
  IN_ONLYDIR = 0x01000000,

  /**
   * Do not follow symlinks
   */
  IN_DONT_FOLLOW = 0x02000000,

  /**
   * Ignore events from unlinked files or directories
   */
  IN_EXCL_UNLINK = 0x04000000,

  /**
   * Update existing mask instead of replacing it
   */
  IN_MASK_ADD = 0x20000000,

  /**
   * Remove watch after first event
   */
  IN_ONESHOT = 0x80000000,
}

export enum EventFlags {
  /**
   * Event is triggered on a directory
   */
  IN_ISDIR = 0x40000000,

  /**
   * The filesystem containing a watched path has been unmounted
   */
  IN_UNMOUNT = 0x00002000,

  /**
   * The event queue has overflowed
   */
  IN_Q_OVERFLOW = 0x00004000,

  /**
   * A watch was removed (explicitly or automatically), possible reasons include
   *
   * - `unwatch()`
   * - path was deleted
   * - filesystem containing path was unmounted
   * - `WatchMask.IN_ONESHOT` was set
   */
  IN_IGNORED = 0x00008000,
}

/**
 * Inotify filesystem event emitter. The only events are `'event'` and `'error'`
 *
 * ```typescript
 * inotify.on('event', (path: string | null, event: WatchMask | EventMask, cookie: number) => void)
 * ```
 */
class Inotify extends EventEmitter {
  private watcher?: NativeInotify
  constructor() {
    super()
    this.watcher = new NativeInotify()
    this.watcher.on((err, path, event, cookie) => {
      if (!this.watcher) return
      if (err) {
        this.emit('error', err)
        return
      }
      this.emit('event', path, event, cookie)
    })
  }
  /**
   * Watch the given path
   * @returns {WatchDescriptor} A descriptor indicating this watch. Note that this is a native object and cannot be extended.
   * The only usage of WatchDescriptor is to `unwatch()`
   */
  public watch(path: string, mask: WatchMask) {
    if (!this.watcher) throw new Error('Inotify instance was closed')
    return this.watcher.watch(path, mask)
  }
  /**
   * Unwatch the given path
   */
  public unwatch(wd: WatchDescriptor) {
    if (!this.watcher) throw new Error('Inotify instance was closed')
    return this.watcher.unwatch(wd)
  }

  /**
   * Close the inotify instance
   */
  public close() {
    // unref the watcher and it will automatically be dropped in rust side
    this.watcher = undefined
  }
}

export { Inotify, WatchDescriptor }
