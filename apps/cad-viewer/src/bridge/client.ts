import {
  BRIDGE_VERSION,
  type FrameMessage,
  type HostMessage,
  hostMessageSchema,
} from '@kcc/viewer-bridge-types';

type Handler = (msg: HostMessage) => void;

/**
 * Iframe side of the postMessage bridge. Validates every incoming message with
 * Zod, ignores anything from an unexpected origin, and provides a typed send
 * channel back to the parent.
 */
export class BridgeClient {
  private handlers = new Set<Handler>();
  private parentOrigin: string | null = null;

  constructor() {
    window.addEventListener('message', this.onMessage);
  }

  /** Register a handler. Returns an unsubscribe function. */
  on(handler: Handler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  /** Send a FrameMessage to the parent. `ready` must be sent before anything else. */
  send(msg: FrameMessage) {
    if (!this.parentOrigin && msg.type !== 'ready') {
      console.warn('[bridge] send before parent origin known; dropping', msg.type);
      return;
    }
    const target = msg.type === 'ready' ? '*' : this.parentOrigin!;
    window.parent.postMessage(msg, target);
  }

  dispose() {
    window.removeEventListener('message', this.onMessage);
    this.handlers.clear();
  }

  private onMessage = (event: MessageEvent) => {
    // Only accept messages from the top window (our parent)
    if (event.source !== window.parent) return;

    const parsed = hostMessageSchema.safeParse(event.data);
    if (!parsed.success) {
      // Unknown or malformed message — silent drop
      return;
    }
    if (parsed.data.v !== BRIDGE_VERSION) {
      console.warn('[bridge] version mismatch', parsed.data.v, 'vs', BRIDGE_VERSION);
      return;
    }

    // Lock to the first parent origin we see
    if (!this.parentOrigin) {
      this.parentOrigin = event.origin;
    } else if (event.origin !== this.parentOrigin) {
      return;
    }

    for (const h of this.handlers) h(parsed.data);
  };
}

export const bridge = new BridgeClient();
