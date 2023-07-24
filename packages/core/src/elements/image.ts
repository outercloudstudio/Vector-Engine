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

		const imageOffscreenCanvas = new OffscreenCanvas(this.size.x, this.size.y)
		imageOffscreenCanvas
			.getContext('2d')
			.drawImage(await this.image.getImage(), 0, 0, this.size.x, this.size.y)

		const offscreenCanvas = new OffscreenCanvas(this.size.x, this.size.y)
		const gl = offscreenCanvas.getContext('webgl2')

		gl.clearColor(1, 0.5, 0.5, 1)
		gl.clear(gl.COLOR_BUFFER_BIT)

		const program = createProgramFromScripts(gl, vertexShaderSource, fragmentShaderSource)
		const positionLocation = gl.getAttribLocation(program, 'a_position')
		const texcoordLocation = gl.getAttribLocation(program, 'a_texCoord')

		const positionBuffer = gl.createBuffer()
		gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer)

		gl.bufferData(
			gl.ARRAY_BUFFER,
			new Float32Array([
				0,
				0,
				this.size.x,
				0,
				0,
				this.size.y,
				0,
				this.size.y,
				this.size.x,
				0,
				this.size.x,
				this.size.y,
			]),
			gl.STATIC_DRAW
		)

		const texcoordBuffer = gl.createBuffer()
		gl.bindBuffer(gl.ARRAY_BUFFER, texcoordBuffer)
		gl.bufferData(
			gl.ARRAY_BUFFER,
			new Float32Array([0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0]),
			gl.STATIC_DRAW
		)

		const texture = gl.createTexture()
		gl.bindTexture(gl.TEXTURE_2D, texture)

		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE)
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE)
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST)
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST)

		gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, imageOffscreenCanvas)

		const resolutionLocation = gl.getUniformLocation(program, 'u_resolution')

		gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)

		gl.useProgram(program)

		gl.enableVertexAttribArray(positionLocation)

		gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer)

		gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0)

		gl.enableVertexAttribArray(texcoordLocation)

		gl.bindBuffer(gl.ARRAY_BUFFER, texcoordBuffer)

		gl.vertexAttribPointer(texcoordLocation, 2, gl.FLOAT, false, 0, 0)

		gl.uniform2f(resolutionLocation, gl.canvas.width, gl.canvas.height)

		gl.drawArrays(gl.TRIANGLES, 0, 6)

		const targetCanvasContext = canvas.getContext('2d')

		targetCanvasContext.drawImage(offscreenCanvas, 100, 100)

		// await this.image.render(canvas)
	}
}
