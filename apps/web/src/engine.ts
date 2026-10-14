// Own only live protocol allocations; copy each output before another command.
export interface Api extends WebAssembly.Exports {
    memory: WebAssembly.Memory;
    sketch_wasm_protocol_version(): number;
    sketch_wasm_create(): number;
    sketch_wasm_free(h: number): number;
    sketch_wasm_buffer_new(n: number): number;
    sketch_wasm_buffer_ptr(t: number): number;
    sketch_wasm_buffer_len(t: number): number;
    sketch_wasm_buffer_free(t: number): number;
    sketch_wasm_command(h: number, op: number, input: number): number;
    sketch_wasm_output(h: number): number;
}
export const floats = (...v: number[]) => {
    const b = new Uint8Array(v.length * 8),
        d = new DataView(b.buffer);
    v.forEach((x, i) => d.setFloat64(i * 8, x, true));
    return b;
};
export const identity = (id: bigint) => {
    const b = new Uint8Array(8);
    new DataView(b.buffer).setBigUint64(0, id, true);
    return b;
};
export class Engine {
    private handle: number;
    private importRequest = 0;
    isAvailable() {
        return this.handle !== 0;
    }
    beginImportRequest() {
        return ++this.importRequest;
    }
    isCurrentImportRequest(request: number) {
        return request === this.importRequest;
    }
    constructor(private api: Api) {
        if (api.sketch_wasm_protocol_version() !== 1)
            throw Error("Unsupported drawing engine");
        this.handle = api.sketch_wasm_create();
        if (!this.handle) throw Error("Cannot allocate drawing engine");
    }
    command(op: number, input = new Uint8Array()): Uint8Array {
        if (!this.handle)
            throw Error("Drawing engine is unavailable; reload the page.");
        if (op === 16 || op === 18) ++this.importRequest;
        let token = 0;
        try {
            if (input.length) {
                token = this.api.sketch_wasm_buffer_new(input.length);
                if (!token) throw Error("Document exceeds available memory");
                const p = this.api.sketch_wasm_buffer_ptr(token);
                new Uint8Array(this.api.memory.buffer, p, input.length).set(
                    input,
                );
            }
            const status = this.api.sketch_wasm_command(this.handle, op, token);
            if (status)
                throw Error(
                    `Drawing operation failed (${status}). The document was not replaced.`,
                );
            const out = this.api.sketch_wasm_output(this.handle);
            if (!out) return new Uint8Array();
            const p = this.api.sketch_wasm_buffer_ptr(out),
                n = this.api.sketch_wasm_buffer_len(out);
            return new Uint8Array(this.api.memory.buffer, p, n).slice();
        } catch (error) {
            if (error instanceof WebAssembly.RuntimeError) this.handle = 0;
            throw error;
        } finally {
            if (token && this.handle) this.api.sketch_wasm_buffer_free(token);
        }
    }
    state() {
        const b = this.command(1),
            d = new DataView(b.buffer);
        return {
            count: d.getBigUint64(0, true),
            selected: d.getBigUint64(8, true),
            undo: !!d.getUint32(16, true),
            redo: !!d.getUint32(20, true),
            dirty: !!d.getUint32(24, true),
            interaction: d.getUint32(28, true),
            bounds: [32, 40, 48, 56].map((o) => d.getFloat64(o, true)),
        };
    }
    begin(pointer: bigint, device: number, rgba: number[], width: number) {
        const b = new Uint8Array(52),
            d = new DataView(b.buffer);
        d.setBigUint64(0, pointer, true);
        d.setUint32(8, device, true);
        [...rgba, width].forEach((v, i) => d.setFloat64(12 + i * 8, v, true));
        this.command(2, b);
    }
    append(
        pointer: bigint,
        samples: {
            x: number;
            y: number;
            time: number;
            pressure?: number;
            id: bigint;
        }[],
    ) {
        const b = new Uint8Array(8 + 48 * samples.length),
            d = new DataView(b.buffer);
        d.setBigUint64(0, pointer, true);
        samples.forEach((s, i) => {
            const o = 8 + i * 48;
            [s.x, s.y, s.time, s.pressure ?? 0].forEach((v, j) =>
                d.setFloat64(o + j * 8, v, true),
            );
            d.setBigUint64(o + 32, s.id, true);
            d.setUint32(o + 40, s.pressure === undefined ? 0 : 1, true);
        });
        this.command(3, b);
    }
    cycle(direction: number) {
        const b = new Uint8Array(4);
        new DataView(b.buffer).setInt32(0, direction, true);
        this.command(8, b);
    }
    snapshot() {
        const b = this.command(17);
        return {
            revision: new DataView(b.buffer).getBigUint64(0, true),
            bytes: b.slice(8),
        };
    }
    dispose() {
        if (this.handle) this.api.sketch_wasm_free(this.handle);
        this.handle = 0;
    }
}

/** Native file reads are asynchronous: never replace intervening edits without a fresh choice. */
export async function importFile(
    engine: Engine,
    file: { size: number; arrayBuffer(): Promise<ArrayBuffer> },
    confirmReplacement: () => Promise<boolean>,
): Promise<boolean> {
    if (file.size > 64 * 1024 * 1024)
        throw Error("Files must be no larger than 64 MiB.");
    const request = engine.beginImportRequest();
    engine.command(5);
    const revision = engine.snapshot().revision;
    const bytes = new Uint8Array(await file.arrayBuffer());
    if (!engine.isCurrentImportRequest(request)) return false;
    const hadDraft = engine.state().interaction !== 0;
    engine.command(5);
    if (
        (hadDraft || engine.snapshot().revision !== revision) &&
        !(await confirmReplacement())
    )
        return false;
    if (!engine.isCurrentImportRequest(request)) return false;
    engine.command(18, bytes);
    return true;
}

/** A reused native dialog must not retain a previous destructive result on Escape. */
export function resetReplacementChoice(dialog: { returnValue: string }) {
    dialog.returnValue = "cancel";
}
