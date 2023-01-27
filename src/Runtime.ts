import { Runtime as JsRuntime } from 'bridge-js-runtime'
import { TBaseModule } from 'bridge-js-runtime/dist/Runtime'
import path from 'path-browserify'

export class Runtime extends JsRuntime {
  protected directory: FileSystemDirectoryHandle | null = null

  constructor(
    rootDirectoryHandle: FileSystemDirectoryHandle | null,
    modules?: [string, TBaseModule][]
  ) {
    super(modules)

    this.directory = rootDirectoryHandle
  }

  reload(directory: FileSystemDirectoryHandle) {
    this.directory = directory
    this.clearCache()
  }

  async readFile(filePath: string) {
    if (this.directory == null) return new File([], 'voidFile.error')

    const readPath: string[] = filePath.split('/')

    if (readPath.length == 0) return new File([], 'voidFile.error')

    let singlePath = readPath.shift()!

    let handle =
      readPath.length >= 1
        ? await this.directory?.getDirectoryHandle(singlePath)
        : await this.directory?.getFileHandle(singlePath)

    while (readPath.length >= 1) {
      singlePath = readPath.shift()!

      handle =
        readPath.length >= 1
          ? await (<FileSystemDirectoryHandle>handle)?.getDirectoryHandle(
              singlePath
            )
          : await (<FileSystemDirectoryHandle>handle)?.getFileHandle(singlePath)
    }

    if (!handle) return new File([], 'voidFile.error')

    return await (<FileSystemFileHandle>handle).getFile()
  }

  run(filePath: string, env: any = {}, fileContent?: string) {
    return super.run(
      filePath,
      Object.assign(env, {
        require: (x: string) => this.require(x, path.dirname(filePath), env),
      }),
      fileContent
    )
  }
}
