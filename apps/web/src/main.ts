import {
    Engine,
    Api,
    floats,
    identity,
    importFile,
    resetReplacementChoice,
} from "./engine.js";
const el = <T extends HTMLElement>(id: string) =>
    document.getElementById(id) as T;
const canvas = el<HTMLCanvasElement>("canvas"),
    ctx = canvas.getContext("2d")!;
let engine: Engine,
    tool = "pen",
    zoom = 1,
    tx = 0,
    ty = 0,
    width = 0,
    height = 0,
    space = false;
let owner: number | undefined,
    sample = 0n,
    mode = "",
    resize: { x: number; y: number; distance: number } | undefined;
const contacts = new Map<number, { x: number; y: number }>();
let pair: number[] = [];
let navigation: { x: number; y: number; distance: number } | undefined;
const point = (x: number, y: number) => ({
    x: (x - tx) / zoom,
    y: (y - ty) / zoom,
});
const error = (e: unknown) => {
    el("error-text").textContent = String(e);
    el<HTMLDialogElement>("error").showModal();
};
let failed = false,
    lastDirty = false,
    protectFailedDocument = false;
function disableEngine(message: string, protectDocument: boolean) {
    failed = true;
    protectFailedDocument = protectDocument;
    document
        .querySelectorAll<
            HTMLButtonElement | HTMLInputElement
        >("header button,.tools button,.tools input")
        .forEach((control) => (control.disabled = true));
    canvas.setAttribute("aria-disabled", "true");
    el("status").textContent = message;
    el("reload").hidden = false;
}
el("reload").addEventListener("click", () => location.reload());
const safe = (fn: () => void) => {
    if (!engine || failed) return;
    try {
        fn();
        refresh();
    } catch (e) {
        cancel();
        if (!engine.isAvailable()) {
            disableEngine(
                "Drawing engine stopped. Reload to recover; unsaved work may be lost.",
                true,
            );
            error(
                "The drawing engine stopped unexpectedly. Reload this page to recover. Unsaved changes cannot be recovered from this failed engine.",
            );
        } else error(e);
    }
};
function cancel() {
    if (engine?.isAvailable()) engine.command(5);
    owner = undefined;
    mode = "";
    resize = undefined;
}
function refresh() {
    if (!engine?.isAvailable()) return;
    const s = engine.state();
    lastDirty = s.dirty;
    for (const [id, disabled] of Object.entries({
        undo: !s.undo,
        redo: !s.redo,
        previous: !s.count,
        next: !s.count,
        scale: !s.selected,
        apply: !s.selected,
        delete: !s.selected,
    }))
        (el(id) as HTMLButtonElement).disabled = disabled;
    el("status").textContent =
        `${s.count} stroke${s.count === 1n ? "" : "s"} · ${Math.round(zoom * 100)}%${s.dirty ? " · Unsaved" : ""}${s.selected ? " · Stroke selected" : ""}`;
    canvas.setAttribute(
        "aria-label",
        `Drawing canvas, ${s.count} strokes, ${s.selected ? "stroke selected, " : ""}${Math.round(zoom * 100)} percent zoom`,
    );
    el("hint").hidden = s.count > 0n || s.interaction !== 0;
    const dpr = window.devicePixelRatio || 1;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = "#fff";
    ctx.fillRect(0, 0, width, height);
    ctx.translate(tx, ty);
    ctx.scale(zoom, zoom);
    const bytes = engine.command(6),
        d = new DataView(bytes.buffer);
    let id: bigint | undefined;
    const flush = () => {
        if (id !== undefined) ctx.fill("nonzero");
    };
    for (let o = 0; o < bytes.length; o += 96) {
        const next = d.getBigUint64(o, true);
        if (next !== id) {
            flush();
            ctx.beginPath();
            id = next;
            const c = [16, 24, 32, 40].map((n) => d.getFloat64(o + n, true));
            ctx.fillStyle = `rgba(${c[0] * 255},${c[1] * 255},${c[2] * 255},${c[3]})`;
        }
        const p = [48, 56, 64, 72, 80, 88].map((n) =>
            d.getFloat64(o + n, true),
        );
        switch (d.getUint32(o + 8, true)) {
            case 0:
                ctx.moveTo(p[0], p[1]);
                break;
            case 1:
                ctx.lineTo(p[0], p[1]);
                break;
            case 2:
                ctx.bezierCurveTo(p[0], p[1], p[2], p[3], p[4], p[5]);
                break;
            case 3:
                ctx.closePath();
        }
    }
    flush();
    if (s.selected) {
        const [x, y, w, h] = s.bounds;
        ctx.strokeStyle = "#1761db";
        ctx.lineWidth = 1 / zoom;
        ctx.strokeRect(x, y, w, h);
        for (const [a, b] of [
            [x, y],
            [x + w, y],
            [x, y + h],
            [x + w, y + h],
        ]) {
            ctx.fillStyle = "#fff";
            ctx.fillRect(a - 4 / zoom, b - 4 / zoom, 8 / zoom, 8 / zoom);
            ctx.strokeRect(a - 4 / zoom, b - 4 / zoom, 8 / zoom, 8 / zoom);
        }
    }
}
function setZoom(value: number, x = width / 2, y = height / 2) {
    cancel();
    const p = point(x, y);
    zoom = Math.max(0.05, Math.min(32, value));
    tx = x - p.x * zoom;
    ty = y - p.y * zoom;
    refresh();
}
function metric() {
    const a = contacts.get(pair[0]),
        b = contacts.get(pair[1]);
    if (!a || !b) return undefined;
    return {
        x: (a.x + b.x) / 2,
        y: (a.y + b.y) / 2,
        distance: Math.max(1, Math.hypot(a.x - b.x, a.y - b.y)),
    };
}
function position(e: PointerEvent) {
    const r = canvas.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
}
function append(e: PointerEvent) {
    const events = e.getCoalescedEvents?.() || [];
    if (!events.length) events.push(e);
    engine.append(
        BigInt(e.pointerId),
        events.map((e) => {
            const p = position(e),
                q = point(p.x, p.y);
            return {
                ...q,
                time: e.timeStamp / 1000,
                pressure:
                    e.pointerType === "pen"
                        ? Math.max(0, Math.min(1, e.pressure))
                        : undefined,
                id: ++sample,
            };
        }),
    );
}
canvas.addEventListener("pointerdown", (e) =>
    safe(() => {
        if (e.button !== 0 && e.button !== 1) return;
        canvas.focus();
        const p = position(e);
        contacts.set(e.pointerId, p);
        canvas.setPointerCapture(e.pointerId);
        if (contacts.size >= 2) {
            cancel();
            if (pair.length === 0) {
                pair = [...contacts.keys()].slice(0, 2);
                navigation = metric();
            }
            return;
        }
        if (pair.length) return;
        owner = e.pointerId;
        if (space || e.button === 1) {
            mode = "pan";
            return;
        }
        const q = point(p.x, p.y),
            s = engine.state();
        if (tool === "select") {
            if (s.selected) {
                const [x, y, w, h] = s.bounds;
                const corners = [
                    [x, y],
                    [x + w, y],
                    [x, y + h],
                    [x + w, y + h],
                ];
                const index = corners.findIndex(
                    ([a, b]) => Math.hypot(q.x - a, q.y - b) < 14 / zoom,
                );
                if (index >= 0) {
                    const [a, b] = corners[3 - index];
                    resize = {
                        x: a,
                        y: b,
                        distance: Math.max(0.01, Math.hypot(q.x - a, q.y - b)),
                    };
                    engine.command(11);
                    mode = "resize";
                    return;
                }
            }
            engine.command(7, floats(q.x, q.y, 8 / zoom));
            mode = "select";
            return;
        }
        const hex = el<HTMLInputElement>("color").value,
            rgba = [1, 3, 5].map(
                (i) => parseInt(hex.slice(i, i + 2), 16) / 255,
            );
        rgba.push(1);
        engine.begin(
            BigInt(e.pointerId),
            e.pointerType === "pen" ? 1 : e.pointerType === "touch" ? 2 : 0,
            rgba,
            Number(el<HTMLInputElement>("width").value),
        );
        mode = "ink";
        sample = 0n;
        append(e);
    }),
);
canvas.addEventListener("pointermove", (e) =>
    safe(() => {
        if (!contacts.has(e.pointerId)) return;
        const old = contacts.get(e.pointerId)!;
        const p = position(e);
        contacts.set(e.pointerId, p);
        if (pair.length) {
            const next = metric();
            if (next && navigation) {
                const q = point(navigation.x, navigation.y);
                zoom = Math.max(
                    0.05,
                    Math.min(32, (zoom * next.distance) / navigation.distance),
                );
                tx = next.x - q.x * zoom;
                ty = next.y - q.y * zoom;
            }
            navigation = next;
            return;
        }
        if (owner !== e.pointerId) return;
        if (mode === "pan") {
            tx += p.x - old.x;
            ty += p.y - old.y;
        } else if (mode === "ink") append(e);
        else if (mode === "resize" && resize) {
            const q = point(p.x, p.y);
            engine.command(
                12,
                floats(
                    Math.max(
                        0.01,
                        Math.hypot(q.x - resize.x, q.y - resize.y) /
                            resize.distance,
                    ),
                    resize.x,
                    resize.y,
                ),
            );
        }
    }),
);
function finish(e: PointerEvent, aborted: boolean) {
    safe(() => {
        if (owner === e.pointerId) {
            if (aborted) cancel();
            else {
                if (mode === "ink") {
                    append(e);
                    engine.command(4, identity(BigInt(e.pointerId)));
                }
                if (mode === "resize" && resize) {
                    const p = position(e),
                        q = point(p.x, p.y);
                    engine.command(
                        12,
                        floats(
                            Math.max(
                                0.01,
                                Math.hypot(q.x - resize.x, q.y - resize.y) /
                                    resize.distance,
                            ),
                            resize.x,
                            resize.y,
                        ),
                    );
                    engine.command(13);
                }
                owner = undefined;
                mode = "";
                resize = undefined;
            }
        }
        contacts.delete(e.pointerId);
        if (pair.includes(e.pointerId)) navigation = undefined;
        if (!contacts.size) {
            pair = [];
            navigation = undefined;
        }
    });
}
canvas.addEventListener("pointerup", (e) => finish(e, false));
canvas.addEventListener("pointercancel", (e) => finish(e, true));
canvas.addEventListener("lostpointercapture", (e) => {
    if (contacts.has(e.pointerId)) finish(e, true);
});
canvas.addEventListener(
    "wheel",
    (e) => {
        e.preventDefault();
        safe(() => {
            cancel();
            if (e.ctrlKey || e.metaKey) {
                const r = canvas.getBoundingClientRect();
                setZoom(
                    zoom * Math.exp(-e.deltaY * 0.01),
                    e.clientX - r.left,
                    e.clientY - r.top,
                );
            } else {
                tx -= e.deltaX;
                ty -= e.deltaY;
            }
        });
    },
    { passive: false },
);
window.addEventListener("blur", () => {
    space = false;
    cancel();
    contacts.clear();
    pair = [];
    navigation = undefined;
    refresh();
});
new ResizeObserver(() => {
    const r = canvas.getBoundingClientRect();
    cancel();
    tx += (r.width - width) / 2;
    ty += (r.height - height) / 2;
    if (!width) {
        tx = 0;
        ty = 0;
    }
    width = r.width;
    height = r.height;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    refresh();
}).observe(canvas);
function chooseTool(next: string) {
    cancel();
    tool = next;
    if (tool === "pen") engine.command(9);
    el("pen").setAttribute("aria-pressed", String(tool === "pen"));
    el("select").setAttribute("aria-pressed", String(tool === "select"));
    refresh();
}
const click = (id: string, fn: () => void) =>
    el(id).addEventListener("click", () =>
        safe(() => {
            cancel();
            fn();
        }),
    );
click("pen", () => chooseTool("pen"));
click("select", () => chooseTool("select"));
for (const [id, op] of [
    ["undo", 14],
    ["redo", 15],
    ["delete", 10],
] as const)
    click(id, () => engine.command(op));
click("previous", () => {
    chooseTool("select");
    engine.cycle(-1);
});
click("next", () => {
    chooseTool("select");
    engine.cycle(1);
});
click("apply", () => {
    const [x, y, w, h] = engine.state().bounds;
    engine.command(11);
    try {
        engine.command(
            12,
            floats(
                Number(el<HTMLInputElement>("scale").value) / 100,
                x + w / 2,
                y + h / 2,
            ),
        );
        engine.command(13);
    } catch (e) {
        cancel();
        throw e;
    }
});
click("in", () => setZoom(zoom * 1.25));
click("out", () => setZoom(zoom / 1.25));
click("reset", () => setZoom(1));
function download() {
    cancel();
    const snapshot = engine.snapshot();
    const url = URL.createObjectURL(
        new Blob([snapshot.bytes.slice().buffer], {
            type: "application/octet-stream",
        }),
    );
    const a = document.createElement("a");
    a.href = url;
    a.download = "Drawing.nsketch";
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 60000);
    refresh();
}
el<HTMLDialogElement>("guard").addEventListener("cancel", () =>
    resetReplacementChoice(el<HTMLDialogElement>("guard")),
);
async function guard() {
    cancel();
    if (!engine.state().dirty) return true;
    const dialog = el<HTMLDialogElement>("guard");
    resetReplacementChoice(dialog);
    dialog.showModal();
    const result = await new Promise<string>((resolve) =>
        dialog.addEventListener("close", () => resolve(dialog.returnValue), {
            once: true,
        }),
    );
    if (result === "download") download();
    return result === "discard";
}
click("save", download);
click("new", () => {
    void guard()
        .then((ok) => {
            if (ok) safe(() => engine.command(16));
        })
        .catch(error);
});
click("open", () => {
    void guard()
        .then((ok) => {
            if (ok) el<HTMLInputElement>("file").click();
        })
        .catch(error);
});
el<HTMLInputElement>("file").addEventListener("change", async (e) => {
    const input = e.target as HTMLInputElement;
    const f = input.files?.[0];
    input.value = "";
    if (!f) return;
    try {
        await importFile(engine, f, guard);
        refresh();
    } catch (e) {
        error(e);
    }
});
window.addEventListener("beforeunload", (e) => {
    if (protectFailedDocument || lastDirty) {
        e.preventDefault();
        e.returnValue = "";
    }
});
document.addEventListener("keydown", (e) => {
    if (
        (e.target as HTMLElement).closest(
            "input,textarea,select,dialog,[contenteditable=true]",
        )
    )
        return;
    if (!engine?.isAvailable() || failed) return;
    const canvasFocused = e.target === canvas;
    const modifier = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    if (canvasFocused && key === " ") {
        space = true;
        e.preventDefault();
        return;
    }
    let action: (() => void) | undefined;
    if (key === "escape") action = cancel;
    else if (modifier && key === "z")
        action = () => {
            const s = engine.state();
            if (e.shiftKey ? s.redo : s.undo)
                engine.command(e.shiftKey ? 15 : 14);
        };
    else if (modifier && key === "s") action = download;
    else if (modifier && key === "o") action = () => el("open").click();
    else if (modifier && key === "n") action = () => el("new").click();
    else if (canvasFocused && !modifier && key === "p")
        action = () => chooseTool("pen");
    else if (canvasFocused && !modifier && key === "v")
        action = () => chooseTool("select");
    else if (
        canvasFocused &&
        (key === "delete" || key === "backspace") &&
        engine.state().selected
    )
        action = () => engine.command(10);
    if (action) {
        e.preventDefault();
        safe(() => {
            cancel();
            action!();
        });
    }
});
document.addEventListener("keyup", (e) => {
    if (e.key === " ") space = false;
});
try {
    const response = await fetch("sketch_wasm.wasm");
    if (!response.ok) throw Error("Drawing engine unavailable");
    const instance = await WebAssembly.instantiate(
        await response.arrayBuffer(),
        {},
    );
    engine = new Engine(instance.instance.exports as Api);
    refresh();
    if ("serviceWorker" in navigator)
        void navigator.serviceWorker.register("./sw.js").catch(() => {});
} catch (e) {
    disableEngine("Drawing engine failed to load. Reload to try again.", false);
    error(e);
}
