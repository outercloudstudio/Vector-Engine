import { MaybeReactor } from '../../../reactive'
import { AnimatedReactiveProperty, animatedReactiveProperty } from '../../element'
import { ShaderFilter } from './shaderFilter'

export class DistortionFilter extends ShaderFilter {
	public distortion: AnimatedReactiveProperty<number> = animatedReactiveProperty<number>(0)

	constructor(options?: { distortion?: MaybeReactor<number> }) {
		super()

		if (options === undefined) return

		if (options.distortion !== undefined) this.distortion(options.distortion)
	}

	protected fragment = ``

	public async render(canvas: OffscreenCanvas) {
		const distortionString = this.distortion().toFixed(2).toString()

		console.log(distortionString)

		this.fragment = `
    precision mediump float;
  
    uniform sampler2D u_image;
  
    varying vec2 v_texCoord;
  
    void main() {
      vec2 uv = v_texCoord;
      vec2 uvNormalized = uv * 2.0 - 1.0;
      float distortionMagnitude = abs(uvNormalized.x * uvNormalized.y);
      float smoothDistortionMagnitude = pow(distortionMagnitude, 1.001);
      vec2 uvDistorted = uv + uvNormalized * smoothDistortionMagnitude * ${distortionString};
      
      // gl_FragColor = vec4(smoothDistortionMagnitude, smoothDistortionMagnitude, smoothDistortionMagnitude, 1.0);
      gl_FragColor = texture2D(u_image, uvDistorted);
    }
    `

		super.render(canvas)
	}
}
