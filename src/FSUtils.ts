export async function doesFileExistOnFolderHandle(name: string, folder: any) {
    try {
        await folder.getFileHandle(name)
    } catch (err) {
        return false
    }

    return true
}

export async function getPermissions(fileHandle: any): Promise<boolean> {
    try {
        const options = {
            mode: 'readwrite'
        }

        if ((await fileHandle.queryPermission(options)) === 'granted') {
            return true
        }

        if ((await fileHandle.requestPermission(options)) === 'granted') {
            return true
        }

        return false
    } catch {
        await new Promise(r => setTimeout(r, 100))

        return await getPermissions(fileHandle)
    }
}

export async function getFileText(file: any) {
    if (file instanceof ArrayBuffer) return (new TextDecoder('utf-8')).decode(file)

    return (new TextDecoder('utf-8')).decode(await (await file.getFile()).arrayBuffer())
}

export async function doesExist(handle: any) {
    try {
        if (handle.kind == 'directory') {
            const dir = await handle.getDirectoryHandle('.vectorEngineNoExist', { create: true })

            await handle.removeEntry(dir.name)
        }
    } catch (err) {
        console.warn(err)

        return false
    }

    return true
}

export async function writeFile(file: any, object: any) {
    const writable = await file.createWritable()

    await writable.write(object)

    await writable.close()
}

export function isJSON(str: string) {
    try {
        JSON.parse(str)

        return true
    } catch { }

    return false
}