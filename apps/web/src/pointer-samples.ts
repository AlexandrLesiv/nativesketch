import type { EditorHost } from './host.ts';

export function metric(app: EditorHost) {
  const a = app.contacts.get(app.pair[0]),
    b = app.contacts.get(app.pair[1]);
  if (!a || !b) return undefined;
  return {
    x: (a.x + b.x) / 2,
    y: (a.y + b.y) / 2,
    distance: Math.max(1, Math.hypot(a.x - b.x, a.y - b.y)),
  };
}
export function position(app: EditorHost, e: PointerEvent) {
  const r = app.canvas.getBoundingClientRect();
  return { x: e.clientX - r.left, y: e.clientY - r.top };
}
export function append(app: EditorHost, e: PointerEvent) {
  const events = e.getCoalescedEvents?.() || [];
  if (!events.length) events.push(e);
  app.engine.append(
    BigInt(e.pointerId),
    events.map((e) => {
      const p = app.position(e),
        q = app.point(p.x, p.y);
      return {
        ...q,
        time: e.timeStamp / 1000,
        pressure: e.pointerType === 'pen' ? Math.max(0, Math.min(1, e.pressure)) : undefined,
        id: ++app.sample,
      };
    }),
  );
}
