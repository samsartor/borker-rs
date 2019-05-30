export type Bindings = typeof import('../bindings/pkg-web');

export let bindings: Promise<Bindings>;
export let syncBindings: Bindings | undefined;

const isNode = new Function("try {return this===global;}catch(e){return false;}");
if (isNode()) {
  bindings = import("../bindings/pkg-node");
} else {
  bindings = import("../bindings/pkg-web");
}
bindings.then((b) => {
  syncBindings = b;
})
