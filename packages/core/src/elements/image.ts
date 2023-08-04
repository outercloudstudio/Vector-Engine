import { ImageAsset } from '../assets/image'
import { Vector2 } from '../vector'
import { createProgramFromScripts } from '../webgl'
import { Element } from './element'

const vertexShaderSource = `
attribute vec2 a_position;
attribute vec2 a_texCoord;

uniform vec2 u_resolution;

varying vec2 v_texCoord;

void main() {
  // convert the rectangle from pixels to 0.0 to 1.0
  vec2 zeroToOne = a_position / u_resolution;

  // convert from 0->1 to 0->2
  vec2 zeroToTwo = zeroToOne * 2.0;

  // convert from 0->2 to -1->+1 (clipspace)
  vec2 clipSpace = zeroToTwo - 1.0;

  gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);

  // pass the texCoord to the fragment shader
  // The GPU will interpolate this value between points.
  v_texCoord = a_texCoord;
}
`

const fragmentShaderSource = `
precision mediump float;

// our texture
uniform sampler2D u_image;

// the texCoords passed in from the vertex shader.
varying vec2 v_texCoord;

void main() {
  gl_FragColor = texture2D(u_image, v_texCoord).bgra;
}
`

export class VectorImage extends Element {
	public image: ImageAsset
	public position: Vector2
	public size: Vector2

	constructor(image: () => ImageAsset, position: Vector2, size: Vector2) {
		super()

		this.image = image()
		this.position = position
		this.size = size
	}

	public async render(canvas: OffscreenCanvas) {
		this.image.position = this.position
		this.image.size = this.size

		await this.image.render(canvas)
	}
}
