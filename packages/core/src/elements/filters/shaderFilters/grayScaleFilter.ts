import { ShaderFilter } from './shaderFilter'

export class GrayScaleFilter extends ShaderFilter {
	protected fragment = `
  precision mediump float;

  uniform sampler2D u_image;

  varying vec2 v_texCoord;

  void main() {
    vec4 texture = texture2D(u_image, v_texCoord);
    vec3 grayScale = vec3(0.5, 0.5, 0.5);
    gl_FragColor = vec4(vec3(dot(texture.rgb, grayScale)), texture.a);
  }
  `
}
