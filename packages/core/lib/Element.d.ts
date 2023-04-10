import { Builder } from './Builders';
import { Scene } from './Scene';
export declare class Element {
    builder: Builder;
    isRendering: boolean;
    scene: Scene;
    id: string;
    constructor(scene: Scene, builder: typeof Builder, options: object);
    render(parentCtx: OffscreenCanvasRenderingContext2D): Promise<void>;
    [key: string]: any;
}
