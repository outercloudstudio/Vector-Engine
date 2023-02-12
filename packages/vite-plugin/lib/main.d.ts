export default function VectorEngine(): Promise<{
    name: string;
    load(id: string): string;
    configureServer(server: any): void;
}>;
