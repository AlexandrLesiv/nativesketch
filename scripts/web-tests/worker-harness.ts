import vm from 'node:vm';

type InstallEvent = { waitUntil(promise: Promise<unknown>): void };

/** Execute emitted worker installation logic; this is not a real-browser offline test. */
export function workerHarness(source: string, failure?: Error) {
  const listeners = new Map<string, (event: InstallEvent) => void>();
  const state = { assets: [] as string[], cacheName: '', deleted: [] as string[] };
  vm.runInNewContext(source, {
    self: {
      addEventListener(name: string, callback: (event: InstallEvent) => void) {
        listeners.set(name, callback);
      },
    },
    caches: {
      async open(name: string) {
        state.cacheName = name;
        return {
          async addAll(paths: string[]) {
            state.assets = [...paths];
            if (failure) throw failure;
          },
        };
      },
      async keys() {
        return ['native-sketch-previous'];
      },
      async delete(name: string) {
        state.deleted.push(name);
        return true;
      },
    },
  });
  return {
    state,
    async install() {
      const install = listeners.get('install');
      if (!install) throw new Error('Worker has no install listener');
      let pending: Promise<unknown> | undefined;
      install({
        waitUntil(promise) {
          pending = promise;
        },
      });
      if (!pending) throw new Error('Worker installation is not awaited');
      await pending;
    },
  };
}
