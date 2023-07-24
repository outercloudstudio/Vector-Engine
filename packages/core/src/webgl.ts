export function compileShader(
	gl: WebGL2RenderingContext,
	shaderSource: string,
	shaderType: number
) {
	let shader = gl.createShader(shaderType)

	gl.shaderSource(shader, shaderSource)

	gl.compileShader(shader)

	let success = gl.getShaderParameter(shader, gl.COMPILE_STATUS)

	if (!success) throw 'could not compile shader:' + gl.getShaderInfoLog(shader)

	return shader
}

export function createProgram(
	gl: WebGL2RenderingContext,
	vertexShader: WebGLShader,
	fragmentShader: WebGLShader
): WebGLProgram {
	let program = gl.createProgram()

	gl.attachShader(program, vertexShader)
	gl.attachShader(program, fragmentShader)

	gl.linkProgram(program)

	let success = gl.getProgramParameter(program, gl.LINK_STATUS)

	if (!success) throw 'program failed to link:' + gl.getProgramInfoLog(program)

	return program
}

export function createProgramFromScripts(
	gl: WebGL2RenderingContext,
	vertexShaderCode: string,
	fragmentShaderCode: string
): WebGLProgram {
	let vertexShader = compileShader(gl, vertexShaderCode, gl.VERTEX_SHADER)
	let fragmentShader = compileShader(gl, fragmentShaderCode, gl.FRAGMENT_SHADER)

	return createProgram(gl, vertexShader, fragmentShader)
}
