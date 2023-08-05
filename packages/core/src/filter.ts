import { createProgramFromScripts } from './webgl'

export class Filter {
	private vertex = `
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
    v_texCoord = a_texCoord;
  }
  `

	private fragment = `
  precision mediump float;

  uniform sampler2D u_image;

  varying vec2 v_texCoord;

  void main() {
    gl_FragColor = texture2D(u_image, v_texCoord).bgra;
  }
  `

	constructor() {}

	public async render(canvas: OffscreenCanvas) {
		console.log(canvas.width, canvas.height)

		const resultCanvas = new OffscreenCanvas(canvas.width, canvas.height)

		const webgl = resultCanvas.getContext('webgl2')

		webgl.clearColor(1, 0.5, 0.5, 1)
		webgl.clear(webgl.COLOR_BUFFER_BIT)

		const program = createProgramFromScripts(webgl, this.vertex, this.fragment)
		const positionLocation = webgl.getAttribLocation(program, 'a_position')
		const texcoordLocation = webgl.getAttribLocation(program, 'a_texCoord')

		const positionBuffer = webgl.createBuffer()
		webgl.bindBuffer(webgl.ARRAY_BUFFER, positionBuffer)
		webgl.bufferData(
			webgl.ARRAY_BUFFER,
			new Float32Array([
				0,
				0,
				canvas.width,
				0,
				0,
				canvas.height,
				0,
				canvas.height,
				canvas.width,
				0,
				canvas.width,
				canvas.height,
			]),
			webgl.STATIC_DRAW
		)

		const texcoordBuffer = webgl.createBuffer()
		webgl.bindBuffer(webgl.ARRAY_BUFFER, texcoordBuffer)
		webgl.bufferData(
			webgl.ARRAY_BUFFER,
			new Float32Array([0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0]),
			webgl.STATIC_DRAW
		)

		const texture = webgl.createTexture()

		webgl.bindTexture(webgl.TEXTURE_2D, texture)
		webgl.texParameteri(webgl.TEXTURE_2D, webgl.TEXTURE_WRAP_S, webgl.CLAMP_TO_EDGE)
		webgl.texParameteri(webgl.TEXTURE_2D, webgl.TEXTURE_WRAP_T, webgl.CLAMP_TO_EDGE)
		webgl.texParameteri(webgl.TEXTURE_2D, webgl.TEXTURE_MIN_FILTER, webgl.NEAREST)
		webgl.texParameteri(webgl.TEXTURE_2D, webgl.TEXTURE_MAG_FILTER, webgl.NEAREST)

		webgl.texImage2D(webgl.TEXTURE_2D, 0, webgl.RGBA, webgl.RGBA, webgl.UNSIGNED_BYTE, canvas)

		const resolutionLocation = webgl.getUniformLocation(program, 'u_resolution')

		webgl.viewport(0, 0, webgl.canvas.width, webgl.canvas.height)

		webgl.useProgram(program)

		webgl.enableVertexAttribArray(positionLocation)
		webgl.bindBuffer(webgl.ARRAY_BUFFER, positionBuffer)
		webgl.vertexAttribPointer(positionLocation, 2, webgl.FLOAT, false, 0, 0)

		webgl.enableVertexAttribArray(texcoordLocation)
		webgl.bindBuffer(webgl.ARRAY_BUFFER, texcoordBuffer)
		webgl.vertexAttribPointer(texcoordLocation, 2, webgl.FLOAT, false, 0, 0)

		webgl.uniform2f(resolutionLocation, webgl.canvas.width, webgl.canvas.height)

		webgl.drawArrays(webgl.TRIANGLES, 0, 6)

		canvas.getContext('2d').drawImage(resultCanvas, 0, 0, canvas.width, canvas.height)
	}
}
