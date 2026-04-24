import { BRIDGE_VERSION, type FrameMessage, type HostMessage } from './protocol';

type Handler = (msg: FrameMessage) => void;

/**
 * Host (parent) side of the postMessage bridge for the iframed mlightcad app.
 *
 * Security model:
 *   - Pin the iframe origin at construction time. All incoming messages are
 *     rejected if they come from a different origin.
 *   - All outgoing messages target that same origin.
 *   - Incoming payloads are shape-checked before being dispatched. We use a
 *     lightweight structural check here instead of pulling Zod into the client
 *     bundle; the iframe side validates with Zod already.
 */
export class ViewerBridgeHost {
  private handlers = new Set<Handler>();
  private readonly frameOrigin: string;
  private readonly frame: () => Window | null;

  constructor(params: { iframeOrigin: string; getWindow: () => Window | null }) {
    this.frameOrigin = params.iframeOrigin;
    this.frame = params.getWindow;
    window.addEventListener('message', this.onMessage);
  }

  on(handler: Handler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  send(msg: HostMessage) {
    const w = this.frame();
    if (!w) return;
    w.postMessage(msg, this.frameOrigin);
  }

  dispose() {
    window.removeEventListener('message', this.onMessage);
    this.handlers.clear();
  }

  private onMessage = (event: MessageEvent) => {
    if (event.origin !== this.frameOrigin) return;
    const data = event.data as unknown;
    if (!isFrameMessage(data)) return;
    for (const h of this.handlers) h(data);
  };
}

function isFrameMessage(x: unknown): x is FrameMessage {
  if (typeof x !== 'object' || x === null) return false;
  const o = x as Record<string, unknown>;
  if (o.v !== BRIDGE_VERSION) return false;
  if (typeof o.type !== 'string') return false;
  switch (o.type) {
    case 'ready':
      return true;
    case 'loadProgress':
      return typeof o.pct === 'number';
    case 'loaded':
      return typeof o.bounds === 'object' && Array.isArray(o.layers);
    case 'featureClicked':
      return typeof o.featureId === 'string';
    case 'entityClicked':
      return typeof o.entityId === 'number';
    case 'error':
      return typeof o.code === 'string' && typeof o.message === 'string';
    default:
      return false;
  }
}
