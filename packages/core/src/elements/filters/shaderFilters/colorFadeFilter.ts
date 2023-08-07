import { MaybeReactor } from '../../../reactive'
import { Vector4 } from '../../../vector'
import { AnimatedReactiveProperty, animatedReactiveProperty } from '../../element'
import { ShaderFilter } from './shaderFilter'

export class ColorFadeFilter extends ShaderFilter {
	public factor: AnimatedReactiveProperty<number> = animatedReactiveProperty<number>(0)
	public color: AnimatedReactiveProperty<Vector4> = animatedReactiveProperty<Vector4>(
		new Vector4(1, 1, 1, 1)
	)

	constructor(options?: { factor?: MaybeReactor<number>; color?: MaybeReactor<Vector4> }) {
		super()

		if (options === undefined) return

		if (options.factor !== undefined) this.factor(options.factor)
		if (options.color !== undefined) this.color(options.color)
	}

	protected fragment = ``

	public async render(canvas: OffscreenCanvas) {
		const factorString = this.factor().toFixed(4).toString()
		const color = this.color()
		const colorString = `vec4(${color.x.toFixed(4).toString()}, ${color.y
			.toFixed(4)
			.toString()}, ${color.z.toFixed(4).toString()}, ${color.w.toFixed(4).toString()})`

		this.fragment = `
    precision mediump float;
  
    uniform sampler2D u_image;
  
    varying vec2 v_texCoord;
  
    void main() {
      gl_FragColor = mix(texture2D(u_image, v_texCoord), ${colorString}, ${factorString});
    }
    `

		super.render(canvas)
	}
}
