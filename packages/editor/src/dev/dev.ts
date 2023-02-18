import data from '@/dev/data.json'
import project from '@/dev/project'

export default function () {
  console.warn('Running dev project!', data)

  window.dispatchEvent(
    new CustomEvent('project', { detail: { project, data } })
  )
}
