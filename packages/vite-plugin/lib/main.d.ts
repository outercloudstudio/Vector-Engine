export default function VectorEngine(configURI: string): Promise<{
    name: string;
    resolveId(id: string): void;
    load(id: string): void;
    transform(code: any, id: any): void;
    configureServer(server: any): void;
    handleHotUpdate(ctx: any): Promise<any[]>;
}>;
