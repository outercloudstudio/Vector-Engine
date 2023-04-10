import { Scene } from './Scene';
function useProjectContext(engine, forReload) {
    return {
        frameRate(frameRate) {
            engine.frameRate = frameRate;
        },
        length(length) {
            engine.length = length;
        },
        minutes(minutes) {
            return engine.frameRate * 60 * minutes;
        },
        seconds(seconds) {
            return engine.frameRate * seconds;
        },
        async loadScene(sceneContext) {
            const scene = new Scene(sceneContext, engine);
            await scene.load();
            engine.scenes.push(scene);
        },
        audioTrack(audio) {
            if (forReload)
                return;
            engine.audioTrack = audio;
        },
    };
}
export class Engine {
    project;
    loaded = false;
    scenes = [];
    frameRate = 60;
    length = 60;
    frame = -1;
    markers = [];
    audioTrack = undefined;
    inferenceAudio = false;
    onError;
    constructor(project, markers, inferenceAudio, onError) {
        this.project = project;
        this.markers = markers;
        if (inferenceAudio)
            this.inferenceAudio = inferenceAudio;
        this.onError = onError;
    }
    async load() {
        try {
            await this.project(useProjectContext(this));
        }
        catch (error) {
            if (this.onError)
                this.onError(error);
            return;
        }
        this.loaded = true;
    }
    async render() {
        const canvas = new OffscreenCanvas(1920, 1080);
        const ctx = canvas.getContext('2d');
        for (const scene of this.scenes) {
            const activeSceneRender = await scene.render();
            ctx.drawImage(activeSceneRender, 0, 0);
        }
        return canvas;
    }
    async reload() {
        this.frameRate = 60;
        this.length = 60;
        this.scenes = [];
        this.frame = 0;
        if (!this.project)
            return;
        try {
            await this.project(useProjectContext(this));
        }
        catch (error) {
            if (this.onError)
                this.onError(error);
            return;
        }
    }
    async next() {
        for (const scene of this.scenes) {
            await scene.next();
        }
        this.frame++;
    }
}
//# sourceMappingURL=Engine.js.map