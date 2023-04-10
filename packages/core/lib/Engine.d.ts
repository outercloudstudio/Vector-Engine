import { Scene } from './Scene';
export declare class Engine {
    project: any;
    loaded: boolean;
    scenes: Scene[];
    frameRate: number;
    length: number;
    frame: number;
    markers: {
        name: string;
        id: string;
        frame: number;
    }[];
    audioTrack: AudioBuffer | undefined;
    inferenceAudio: boolean;
    onError: any;
    constructor(project: any, markers: {
        name: string;
        id: string;
        frame: number;
    }[], inferenceAudio?: boolean, onError?: any);
    load(): Promise<void>;
    render(): Promise<OffscreenCanvas>;
    reload(): Promise<void>;
    next(): Promise<void>;
}
