<script lang="ts">
	import { onMount } from 'svelte'
	import { mat4 } from 'gl-matrix'

	let canvas: HTMLCanvasElement

	onMount(() => {
		console.log(canvas)

		const webgl = canvas.getContext('webgl')
		webgl.clearColor(0.0, 0.0, 0.0, 1.0)
		webgl.clearDepth(1.0)
		webgl.enable(webgl.DEPTH_TEST)
		webgl.depthFunc(webgl.LEQUAL)

		webgl.clear(webgl.COLOR_BUFFER_BIT | webgl.DEPTH_BUFFER_BIT)

		const vertexShaderSource = `
      attribute vec4 aVertexPosition;

      uniform mat4 uModelViewMatrix;
      uniform mat4 uProjectionMatrix;

      void main() {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
      }
    `

		const fragementShaderSource = `
      void main() {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
      }
    `

		const vertexShader = webgl.createShader(webgl.VERTEX_SHADER)
		webgl.shaderSource(vertexShader, vertexShaderSource)
		webgl.compileShader(vertexShader)

		if (!webgl.getShaderParameter(vertexShader, webgl.COMPILE_STATUS)) {
			console.error(
				`An error occurred compiling the vertex shader: ${webgl.getShaderInfoLog(vertexShader)}`
			)

			webgl.deleteShader(vertexShader)
		}

		const fragmentShader = webgl.createShader(webgl.FRAGMENT_SHADER)
		webgl.shaderSource(fragmentShader, fragementShaderSource)
		webgl.compileShader(fragmentShader)

		if (!webgl.getShaderParameter(fragmentShader, webgl.COMPILE_STATUS)) {
			console.error(
				`An error occurred compiling the fragment shader: ${webgl.getShaderInfoLog(fragmentShader)}`
			)

			webgl.deleteShader(vertexShader)
		}

		const shaderProgram = webgl.createProgram()
		webgl.attachShader(shaderProgram, vertexShader)
		webgl.attachShader(shaderProgram, fragmentShader)
		webgl.linkProgram(shaderProgram)

		if (!webgl.getProgramParameter(shaderProgram, webgl.LINK_STATUS)) {
			console.error(
				`Unable to initialize the shader program: ${webgl.getProgramInfoLog(shaderProgram)}`
			)
		}

		const programInfo = {
			program: shaderProgram,
			attribLocations: {
				vertexPosition: webgl.getAttribLocation(shaderProgram, 'aVertexPosition'),
			},
			uniformLocations: {
				projectionMatrix: webgl.getUniformLocation(shaderProgram, 'uProjectionMatrix'),
				modelViewMatrix: webgl.getUniformLocation(shaderProgram, 'uModelViewMatrix'),
			},
		}

		const positionBuffer = webgl.createBuffer()
		webgl.bindBuffer(webgl.ARRAY_BUFFER, positionBuffer)
		const positions = [1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, -1.0]
		webgl.bufferData(webgl.ARRAY_BUFFER, new Float32Array(positions), webgl.STATIC_DRAW)

		const buffers = {
			position: positionBuffer,
		}

		const fieldOfView = (45 * Math.PI) / 180
		const aspect = webgl.canvas.width / webgl.canvas.height
		const zNear = 0.1
		const zFar = 100.0
		const projectionMatrix = mat4.create()
		mat4.perspective(projectionMatrix, fieldOfView, aspect, zNear, zFar)

		const modelViewMatrix = mat4.create()

		mat4.translate(modelViewMatrix, modelViewMatrix, [-0.0, 0.0, -6.0])

		webgl.bindBuffer(webgl.ARRAY_BUFFER, buffers.position)
		webgl.vertexAttribPointer(programInfo.attribLocations.vertexPosition, 2, webgl.FLOAT, false, 0, 0)
		webgl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition)

		webgl.useProgram(programInfo.program)

		webgl.uniformMatrix4fv(programInfo.uniformLocations.projectionMatrix, false, projectionMatrix)

		webgl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix)

		webgl.drawArrays(webgl.TRIANGLE_STRIP, 0, 4)
	})
</script>

<main>
	<canvas bind:this={canvas} width="640" height="480" />
</main>

<style>
</style>
