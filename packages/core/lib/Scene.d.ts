import { Engine } from './Engine';
import { Element } from './Element';
import { Vector } from './Vector';
import { Rect, Ellipse, Image as ImageBuilder, Text, Link, Builder } from './Builders';
export declare function useSceneContext(scene: Scene): {
    Vector: typeof Vector;
    Builders: {
        Rect: typeof Rect;
        Ellipse: typeof Ellipse;
        Image: typeof ImageBuilder;
        Link: typeof Link;
        Text: typeof Text;
    };
    Modes: {
        Linear(time: number): number;
        Ease(time: number): number;
        EaseIn(time: number): number;
        EaseOut(time: number): number;
    };
    Transitions: {
        Cut: ({ load, unload }: any) => Generator<never, void, unknown>;
        Fade(length: number): ({ load, unload, defineModifier }: any) => Generator<any, void, unknown>;
    };
    ElementTransitions: {
        Fade: (inputCanvas: OffscreenCanvas, element: Element) => OffscreenCanvas;
        Circle: (inputCanvas: OffscreenCanvas, element: Element) => OffscreenCanvas;
        CircleCut: (inputCanvas: OffscreenCanvas, element: Element) => OffscreenCanvas;
    };
    Element: typeof Element;
    defineName(name: string): void;
    createElement(builder: typeof Builder, options: object): Element;
    removeElement(element: Element): Element;
    animate: (length: number, mode: any, operator: any) => AsyncGenerator<any, void, unknown>;
    animateVector: (property: any, a: Vector, b: Vector, length: number, mode: any) => Generator<any, void, unknown>;
    wait: (length: number) => Generator<any, void, unknown>;
    waitForMarker: (name: string, offset?: number) => Generator<any, void, unknown>;
    waitWhile: (condition: any) => AsyncGenerator<any, void, unknown>;
    waitForTransition: () => Generator<any, void, unknown>;
    lerp(a: number, b: number, t: number): number;
    relative(pos: Vector): Vector;
    absolute(pos: Vector): Vector;
    seconds(seconds: number): number;
    minutes(minutes: number): number;
    minutesFrames(minutes: number): number;
    frames(frames: number): number;
    aside(context: any): void;
    waitForAll: () => AsyncGenerator<any, void, unknown>;
    waitForAny: () => AsyncGenerator<any, void, unknown>;
    loop: (contextLambda: any) => AsyncGenerator<any, never, any>;
    transition: (sceneContext: any, transition: any) => AsyncGenerator<any, void, any>;
};
export declare class Scene {
    name: string;
    context: any;
    unloadedContext: any;
    loaded: boolean;
    engine: Engine;
    elements: Element[];
    sideContexts: any[];
    transitionRenderModifier: any;
    id: string;
    constructor(context: any, engine: Engine);
    load(): Promise<void>;
    render(): Promise<any>;
    next(): Promise<void>;
    addElement(element: Element): void;
}
