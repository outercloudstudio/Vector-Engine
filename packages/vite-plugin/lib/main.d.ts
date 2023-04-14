export default function VectorEngine(configURI: string): Promise<{
    name: string;
    resolveId(id: string, importer: string | undefined, options: {
        assertions: Record<string, string>;
        custom?: {
            [plugin: string]: any;
        };
        isEntry: boolean;
    }): string;
    load(id: string): string;
    transform(code: any, id: any): string;
    configureServer(server: any): void;
    handleHotUpdate(ctx: any): Promise<any[]>;
}>;
