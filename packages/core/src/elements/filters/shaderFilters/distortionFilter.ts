import { MaybeReactor } from '../../../reactive'
import { AnimatedReactiveProperty, animatedReactiveProperty } from '../../element'
import { ShaderFilter } from './shaderFilter'

export class DistortionFilter extends ShaderFilter {
	public distortion: AnimatedReactiveProperty<number> = animatedReactiveProperty<number>(0)
	public power: AnimatedReactiveProperty<number> = animatedReactiveProperty<number>(1.5)

	constructor(options?: { distortion?: MaybeReactor<number>; power?: MaybeReactor<number> }) {
		super()

		if (options === undefined) return

		if (options.distortion !== undefined) this.distortion(options.distortion)
		if (options.power !== undefined) this.power(options.power)
	}

	protected fragment = ``

	public async render(canvas: OffscreenCanvas) {
		const distortionString = this.distortion().toFixed(4).toString()
		const powerString = this.power().toFixed(4).toString()

		this.fragment = `
    precision mediump float;
  
    uniform sampler2D u_image;
  
    varying vec2 v_texCoord;
  
    void main() {
      vec2 uv = v_texCoord;
      vec2 uvNormalized = uv * 2.0 - 1.0;
      float distortionMagnitude = abs(uvNormalized.x * uvNormalized.y);
      float smoothDistortionMagnitude = pow(distortionMagnitude, ${powerString});
      vec2 uvDistorted = uv + uvNormalized * smoothDistortionMagnitude * ${distortionString};
      
      // gl_FragColor = vec4(smoothDistortionMagnitude, smoothDistortionMagnitude, smoothDistortionMagnitude, 1.0);
      gl_FragColor = texture2D(u_image, uvDistorted);
    }
    `

		super.render(canvas)
	}
}
