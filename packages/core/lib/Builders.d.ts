import { Vector } from './Vector';
import { Element } from './Element';
export declare class Builder {
    element: Element;
    constructor(me: Element);
    setup(options: any): void;
    protected defineAnimatedVectorSetter(property: string): (vector: Vector, length: number, mode: any) => AsyncGenerator<any, void, unknown>;
    protected defineAnimatedNumberSetter(property: string): (value: number, length: number, mode: any) => AsyncGenerator<any, void, unknown>;
}
export declare class TransformBuilder extends Builder {
    setup(options: any): void;
}
export declare class Link extends TransformBuilder {
    setup(options: any): void;
    rotateVector(vector: Vector, angle: number): Vector;
    updatePosition(): void;
    updateRotation(): void;
    updateScale(): void;
    updateCache(): void;
}
export declare class RenderingBuilder extends TransformBuilder {
    setup(options: any): void;
    bounds(): Vector;
    extent(): Vector;
    extentOffset(): Vector;
    render(ctx: OffscreenCanvasRenderingContext2D): Promise<void>;
    defineTransition(): (time: number, transition: (canvas: OffscreenCanvas, element: Element) => OffscreenCanvas, mode: any) => AsyncGenerator<any, void, unknown>;
    defineTransitionOut(): (time: number, transition: (canvas: OffscreenCanvas, element: Element) => OffscreenCanvas) => AsyncGenerator<any, void, unknown>;
}
export declare class Rect extends RenderingBuilder {
    setup(options: any): void;
    bounds(): Vector;
    extent(): Vector;
    extentOffset(): Vector;
    render(ctx: OffscreenCanvasRenderingContext2D): Promise<void>;
}
export declare class Ellipse extends RenderingBuilder {
    setup(options: any): void;
    bounds(): Vector;
    extent(): Vector;
    extentOffset(): Vector;
    render(ctx: OffscreenCanvasRenderingContext2D): Promise<void>;
}
export declare class Image extends RenderingBuilder {
    setup(options: any): void;
    bounds(): Vector;
    extent(): Vector;
    extentOffset(): Vector;
    render(ctx: OffscreenCanvasRenderingContext2D): Promise<void>;
}
export declare class Text extends RenderingBuilder {
    setup(options: any): void;
    bounds(): Vector;
    extent(): Vector;
    extentOffset(): Vector;
    render(ctx: OffscreenCanvasRenderingContext2D): Promise<void>;
}
