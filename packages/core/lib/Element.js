import { RenderingBuilder } from './Builders';
import { uuid } from './Math';
export class Element {
    builder;
    isRendering = false;
    scene;
    id = uuid();
    constructor(scene, builder, options) {
        this.scene = scene;
        if (builder == undefined)
            throw new Error('You must specify a builder when creating an element');
        this.builder = new builder(this);
        this.builder.setup(options);
    }
    async render(parentCtx) {
        if (!(this.builder instanceof RenderingBuilder))
            throw new Error('Can not render a non rendering builder');
        let doDirectRender = this.directRender && this.renderingModifier == undefined;
        if (doDirectRender) {
            parentCtx.translate(this.position.x, this.position.y);
            parentCtx.rotate((this.rotation * Math.PI) / 180);
            const extent = this.builder.extent();
            const offset = this.builder.extentOffset();
            parentCtx.translate(-offset.x, -offset.y);
            parentCtx.translate(-extent.x * this.origin.x, -extent.y * this.origin.y);
            this.builder.render(parentCtx);
            parentCtx.resetTransform();
        }
        else {
            const bounds = this.builder.bounds();
            const canvas = new OffscreenCanvas(bounds.x, bounds.y);
            const ctx = canvas.getContext('2d');
            ctx.translate(0, canvas.height);
            ctx.scale(1, -1);
            this.builder.render(ctx);
            const extent = this.builder.extent();
            const offset = this.builder.extentOffset();
            parentCtx.translate(this.position.x, this.position.y);
            parentCtx.rotate((this.rotation * Math.PI) / 180);
            parentCtx.translate(-offset.x, -offset.y);
            parentCtx.translate(-extent.x * this.origin.x, -extent.y * this.origin.y);
            if (canvas.width == 0 || canvas.height == 0) {
                parentCtx.resetTransform();
                return;
            }
            if (this.renderingModifier != undefined) {
                parentCtx.drawImage(this.renderingModifier(canvas, this), 0, 0);
            }
            else {
                parentCtx.drawImage(canvas, 0, 0);
            }
            parentCtx.resetTransform();
        }
    }
}
//# sourceMappingURL=Element.js.map