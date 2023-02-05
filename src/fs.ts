import { keys, get, set } from 'idb-keyval'

declare const window: any

export async function getProjectsFolder(): Promise<any> {
  if (!(await keys()).includes('projects-folder')) return null

  return await get('projects-folder')
}

export async function hasProjectsFolderPermissions(): Promise<boolean> {
  const projectsFolder = await getProjectsFolder()

  if (projectsFolder == null) return false

  return (
    (await projectsFolder.queryPermission({
      mode: 'readwrite',
    })) == 'granted'
  )
}

export async function hasPermissions(handle: any): Promise<boolean> {
  return (
    (await handle.queryPermission({
      mode: 'readwrite',
    })) == 'granted'
  )
}

export async function getProjectsFolderPermissions(): Promise<boolean> {
  const projectsFolder = await getProjectsFolder()

  if (projectsFolder == null) return false

  if (
    (await projectsFolder.queryPermission({ mode: 'readwrite' })) == 'granted'
  )
    return true

  if (
    (await projectsFolder.requestPermission({ mode: 'readwrite' })) != 'granted'
  )
    return false

  return true
}

async function requestPermissions(handle: any): Promise<boolean> {
  if ((await handle.queryPermission({ mode: 'readwrite' })) == 'granted')
    return true

  if ((await handle.requestPermission({ mode: 'readwrite' })) != 'granted')
    return false

  return true
}

export async function getDirectoryPicker(): Promise<any> {
  let handle = null

  try {
    handle = await window.showDirectoryPicker()

    await requestPermissions(handle)
  } catch {
    handle = null
  }

  return handle
}

export async function setProjectsFolder(handle: any) {
  await set('projects-folder', handle)
}

export async function hasProjectsFolder(): Promise<boolean> {
  return (await keys()).includes('projects-folder')
}

export function isFile(handle: any): boolean {
  return handle.kind == 'file'
}

export function isFolder(handle: any): boolean {
  return handle.kind == 'directory'
}

export async function getFolders(
  handle: any
): Promise<FileSystemDirectoryHandle[]> {
  if (!(await requestPermissions(handle))) return []

  let folders: any[] = []

  for await (const potentialFolder of handle.values()) {
    if (isFile(potentialFolder)) continue

    folders.push(potentialFolder)
  }

  return folders
}

export async function createProject(name: string) {
  if (!(await getProjectsFolderPermissions())) return

  const projectsFolder = await getProjectsFolder()

  await projectsFolder.getDirectoryHandle(name, { create: true })
}

export async function deleteProject(name: string) {
  if (!(await getProjectsFolderPermissions())) return

  const projectsFolder = await getProjectsFolder()

  await projectsFolder.removeEntry(name)
}

export async function getProjectFolder(
  name: string
): Promise<FileSystemDirectoryHandle | null> {
  const dbKeys = await keys()

  if (dbKeys.includes('project-folder')) {
    const projectFolder = await get('project-folder')

    if (
      projectFolder &&
      (await requestPermissions(projectFolder)) &&
      (projectFolder.name == name || name == '')
    ) {
      return projectFolder
    }
  }

  const projectsFolder = await getProjectsFolder()

  if (!projectsFolder) return null

  let projectFolder = null

  try {
    projectFolder = await projectsFolder.getDirectoryHandle(name)
  } catch {}

  return await projectFolder
}

export async function cacheProjectFolder(
  handle: FileSystemDirectoryHandle
): Promise<void> {
  set('project-folder', handle)
}
