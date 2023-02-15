export default function VectorEngine(configURI: string): Promise<{
    name: string;
    resolveId(id: string): string;
    load(id: string): string;
    configureServer(server: any): void;
    handleHotUpdate(ctx: any): Promise<any[]>;
}>;
