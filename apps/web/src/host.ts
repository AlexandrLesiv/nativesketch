import type { Engine } from './engine.ts';
import { el } from './elements.ts';
import { error, safe, refresh, disableEngine } from './presentation.ts';
import { setZoom } from './viewport.ts';
import { metric, position, append } from './pointer-samples.ts';
import { chooseTool, click } from './controls.ts';
import { download, guard } from './files.ts';

/** Native browser presentation state; Rust remains the only document owner. */
export class EditorHost {
  readonly canvas = el<HTMLCanvasElement>('canvas');
  readonly ctx = this.canvas.getContext('2d')!;
  engine!: Engine;
  tool = 'pen';
  zoom = 1;
  tx = 0;
  ty = 0;
  width = 0;
  height = 0;
  space = false;
  owner: number | undefined;
  sample = 0n;
  mode = '';
  resize: { x: number; y: number; distance: number } | undefined;
  readonly contacts = new Map<number, { x: number; y: number }>();
  pair: number[] = [];
  navigation: { x: number; y: number; distance: number } | undefined;
  failed = false;
  lastDirty = false;
  protectFailedDocument = false;
  point(x: number, y: number) {
    return { x: (x - this.tx) / this.zoom, y: (y - this.ty) / this.zoom };
  }
  error = (e: unknown) => {
    error(this, e);
  };
  safe(fn: () => void) {
    safe(this, fn);
  }
  refresh() {
    refresh(this);
  }
  disableEngine(message: string, protect: boolean) {
    disableEngine(this, message, protect);
  }
  cancel = () => {
    if (this.engine?.isAvailable()) this.engine.command(5);
    this.owner = undefined;
    this.mode = '';
    this.resize = undefined;
  };
  setZoom(value: number, x = this.width / 2, y = this.height / 2) {
    setZoom(this, value, x, y);
  }
  metric() {
    return metric(this);
  }
  position(e: PointerEvent) {
    return position(this, e);
  }
  append(e: PointerEvent) {
    append(this, e);
  }
  chooseTool(next: string) {
    chooseTool(this, next);
  }
  click(id: string, fn: () => void) {
    click(this, id, fn);
  }
  download = () => {
    download(this);
  };
  guard = () => {
    return guard(this);
  };
}
