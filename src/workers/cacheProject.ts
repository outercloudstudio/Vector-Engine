import { get, set, keys, clear } from 'idb-keyval'

onmessage = async (event) => {    
    await set('project-vfs-cache', event.data[0])
    await set('project-handle-cache', event.data[1])

    console.log('Cached Project!')

    close()
}