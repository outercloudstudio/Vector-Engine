import { makeProject } from '@vector-engine/core'
import audio from '../Assets/Full.mp3'
import scene from './scene'
import laggyScene from './laggyScene'

export default makeProject(
	60,
	10,
	{
		laggyScene,
		scene,
	},
	audio
)
