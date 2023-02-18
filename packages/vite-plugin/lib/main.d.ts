export default function VectorEngine(configURI: string): Promise<{
    name: string;
    resolveId(id: string): string | {
        id: string;
        external: boolean;
    };
    load(id: string): string;
    transform(code: any, id: any): string;
    configureServer(server: any): void;
    handleHotUpdate(ctx: any): Promise<any[]>;
}>;
