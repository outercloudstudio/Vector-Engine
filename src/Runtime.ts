import { Runtime as JsRuntime } from 'bridge-js-runtime'
import { TBaseModule } from 'bridge-js-runtime/dist/Runtime'
import path from 'path-browserify'

export class Runtime extends JsRuntime {
  protected directory: FileSystemDirectoryHandle | null = null

  constructor(
    directory: FileSystemDirectoryHandle,
    modules?: [string, TBaseModule][]
  ) {
    super(modules)

    this.directory = directory
  }

  async readFile(filePath: string) {
    console.warn('Runtime reading file "' + filePath + '"')

    const handle = await this.directory?.getFileHandle(filePath)

    console.log(this.directory)

    console.log(filePath)

    console.log(handle)

    if (!handle) return new File([], 'voidFile.error')

    console.log(await handle.getFile())

    return await handle.getFile()
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
