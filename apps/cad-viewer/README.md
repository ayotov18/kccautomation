# @kcc/cad-viewer-app

Iframed CAD viewer built on [mlightcad/cad-viewer](https://github.com/mlightcad/cad-viewer) (MIT). Runs as its own Vite-served app and is embedded in the Next.js frontend via an `<iframe>` path-mounted at `/cad-viewer/`. All communication with the host happens through `window.postMessage` using the protocol defined in `packages/viewer-bridge-types/`.

## Contract with the Next.js host

1. The host mounts this app in a sandboxed iframe (`allow-scripts allow-same-origin`).
2. On load, the app sends a `ready` message.
3. Host responds with `load` carrying a short-lived, HMAC-signed `sourceUrl` (`/api/v1/viewer/source/:drawingId?token=...`). The app fetches the DWG from that URL.
4. Host pushes KCC features via `setKccFeatures` whenever they change. The overlay component renders hotpoints on top of the cad-viewer canvas.
5. Clicks on hotpoints propagate to the host via `featureClicked`.

## What this app does NOT do

- Accept local file uploads. There is exactly one upload pipeline for the whole system — the Next.js → Rust path that owns KCC analysis. Dragging a file into this iframe does nothing. If upstream mlightcad ever surfaces a file picker, the `element-plus-override.scss` hides it via `.upload-screen { display: none }`.
- Write anything back into the KCC pipeline. The app is read-only with respect to drawing source and analysis results.
- Run in isolation. The `ready → load` handshake is required before any rendering happens.

## Local dev

```bash
pnpm install
pnpm dev   # starts on :3002
```

Pair with the Next.js frontend running on :3001 and set `NEXT_PUBLIC_CAD_VIEWER_ORIGIN=http://localhost:3002` so the host knows where to point the iframe.

## Known gaps (wired in later phases)

- `setLayerVisible` from host is currently a no-op; needs hookup to `AcApDocManager` layer table once `@create` fires.
- `zoom` commands from host are a no-op; needs binding to `AcApZoomCmd`.
- `KccOverlay` uses a placeholder world-space viewBox instead of screen-space projection via the cad-viewer camera. Phase 3b in the execution plan.
