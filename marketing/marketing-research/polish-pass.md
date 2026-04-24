# KCC Automation — Landing Page Polish Pass

Dark industrial B2B, amber-bronze accents, Next.js 15 + Tailwind v4 + Motion. Every item below takes ≤10 lines and has a concrete fit for KCC's editorial/industrial tone.

---

## 1. Component Libraries — Install Only What Fits

### Magic UI (primary pick — industrial, clean, composable)
Install root: `pnpm dlx shadcn@latest add "https://magicui.design/r/<name>.json"` ([docs](https://magicui.design/docs)).

- **Border Beam** — single traveling light seam on cards/CTAs; reads like a machined edge. `pnpm dlx shadcn@latest add @magicui/border-beam` ([link](https://magicui.design/docs/components/border-beam)).
- **Animated Beam** — connector lines between logos/nodes for the pipeline section; industrial wiring vibe. ([link](https://magicui.design/docs/components/animated-beam))
- **Number Ticker** — stat counters for "hours saved / jobs automated"; spring-eased, feels expensive. ([link](https://magicui.design/docs/components/number-ticker))
- **Particles** — low-density (30–60) amber dots behind hero; subtle dust instead of Aceternity's sparkle. ([link](https://magicui.design/docs/components/particles))
- **Hero Video Dialog** — for the demo-reel play button; keeps your hover-play videos primary. ([link](https://magicui.design/docs/components/hero-video-dialog))

Skip: Meteors (too playful), OrbitingCircles (dev-toolish), Globe (not on-brand for construction).

### Motion Primitives (best-in-class micro-interactions — [motion-primitives.com](https://motion-primitives.com))
- **TextEffect** — per-word/char stagger reveals for H1/H2. ([link](https://motion-primitives.com/docs/text-effect))
- **Magnetic** — pull-to-cursor on primary CTAs only. ([link](https://motion-primitives.com/docs/magnetic))
- **Progressive Blur** — layered gradient blur at section seams; solves #2 below. ([link](https://motion-primitives.com/docs/progressive-blur))
- **InView / ScrollReveal** — drop-in replacement for `whileInView` boilerplate.
- **Cursor** — custom cursor with trailing dot, amber tint.

Install: CLI per-component `pnpm dlx shadcn@latest add "https://motion-primitives.com/c/<name>.json"`.

### Aceternity UI ([ui.aceternity.com](https://ui.aceternity.com/components))
- **Spotlight** — one controlled sweep behind hero headline. ([link](https://ui.aceternity.com/components/spotlight))
- **Background Beams** — SVG beam field; use at 30% opacity under dark sections. ([link](https://ui.aceternity.com/components/background-beams))
- **Tracing Beam** — scroll-following beam down the long-form case-study section. ([link](https://ui.aceternity.com/components/tracing-beam))
- **Lamp Effect** — use once, above a hero subsection title (Linear-grade). ([link](https://ui.aceternity.com/components/lamp-effect))

### React Bits ([reactbits.dev](https://www.reactbits.dev))
- **SplitText** — GSAP-style char reveal, heavier feel than TextEffect.
- **Aurora / Threads** — abstract dark-mode backgrounds. Pick ONE, place on hero only.
- **TiltedCard** — reserve for pricing/widget cards, `maxTilt={6}` (not 15).

### shadcn Blocks + primitives
Use the new `chart-*` blocks and `/blocks` page templates as scaffolding ([ui.shadcn.com/blocks](https://ui.shadcn.com/blocks)). With Tailwind v4 migration complete, use the updated CSS-var tokens: [ui.shadcn.com/docs/tailwind-v4](https://ui.shadcn.com/docs/tailwind-v4).

---

## 2. Edge Vignetting & Cinematic Depth (drop-in CSS)

Goal: dark pages should feel like a volumetric stage, not a flat canvas. All techniques leverage standard `mask-image` ([MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/mask-image)).

```css
/* 1. Viewport vignette — inset shadow on a fixed overlay */
.vignette::after{
  content:""; position:fixed; inset:0; pointer-events:none; z-index:40;
  box-shadow: inset 0 0 240px 40px rgb(0 0 0 / .75);
}

/* 2. Section fade-in/out top+bottom (Tailwind v4 arbitrary) */
.section-mask{
  mask-image: linear-gradient(to bottom, transparent, #000 10%, #000 90%, transparent);
}

/* 3. Radial spotlight mask — reveal content inside a soft circle */
.radial-reveal{
  mask-image: radial-gradient(ellipse 70% 60% at 50% 40%, #000 55%, transparent 90%);
}

/* 4. Edge blur trim — softens right/left page gutters */
.edge-blur{ position:fixed; inset:0; pointer-events:none;
  backdrop-filter: blur(8px);
  mask-image: linear-gradient(to right, #000, transparent 8%, transparent 92%, #000);
}

/* 5. Amber hot-spot (KCC accent) — paints a bronze gleam at 20% 30% */
.hot-spot::before{ content:""; position:absolute; inset:0; pointer-events:none;
  background: radial-gradient(600px circle at 20% 30%, oklch(0.72 0.16 55 / .18), transparent 60%);
}

/* 6. Grain + vignette composite overlay */
.grain{ background-image:url('/noise.png'); background-size:180px; opacity:.04;
  mix-blend-mode: overlay; position:fixed; inset:0; pointer-events:none; z-index:50; }
```

Tailwind v4 exposes `mask-*` utilities natively — see [tailwindcss.com/docs/mask-image](https://tailwindcss.com/docs/mask-image).

---

## 3. Floating Navbar — Concrete Spec

Reference: [Aceternity Navbar Pill](https://ui.aceternity.com/blocks/navbars/navbar-pill), [Aceternity Floating Navbar](https://ui.aceternity.com/components/floating-navbar), Linear/Arc/Raycast in the wild.

**Spec:**
- Shape: pill, `rounded-full`, height `56px` collapsed / `64px` expanded.
- Position: `fixed top-4 left-1/2 -translate-x-1/2`; max-w `min(960px, 92vw)`.
- Background (top of page): `bg-transparent` with 1px `border-white/5`.
- Background (scrolled > 24px): `bg-neutral-950/70 backdrop-blur-xl border-white/10`.
- Shrink: height 64→56, padding-x 28→20, logo 28→22 — spring `{stiffness:260,damping:28}`.
- Edge glow: `shadow-[0_0_60px_-12px_oklch(0.72_0.16_55/.35)]` only when scrolled.
- Inner separator: `divide-x divide-white/5` between logo / nav / CTA groups.
- CTA: single amber `<MagneticButton>` with BorderBeam.
- Mobile (<768): full-width rounded-2xl, tap opens sheet drawer from top with `backdrop-blur-2xl`.

Hook it up with Motion's `useScroll` + `useTransform`:
```tsx
const { scrollY } = useScroll();
const h  = useTransform(scrollY,[0,80],[64,56]);
const bg = useTransform(scrollY,[0,80],["rgba(10,10,10,0)","rgba(10,10,10,0.72)"]);
```

---

## 4. Everything-Animated Pass (without feeling "too much")

Rule: **one hero animation, one section revealer, one interaction layer.** Don't stack three.

| Effect | Use on | Package |
|---|---|---|
| Magnetic buttons | Primary CTAs only | [motion-primitives Magnetic](https://motion-primitives.com/docs/magnetic) |
| Text splitter + stagger | H1, section H2 | [TextEffect](https://motion-primitives.com/docs/text-effect) or [React Bits SplitText](https://www.reactbits.dev) |
| Scroll reveal | Every section wrapper | `motion/react` `whileInView` + `viewport={{once:true,margin:"-10%"}}` |
| SVG line-draw on scroll | Pipeline diagram | Motion `pathLength` + `useScroll` ([docs](https://motion.dev/docs/react-motion-value)) |
| Number tickers | Stats row | [Magic UI NumberTicker](https://magicui.design/docs/components/number-ticker) |
| Tilt on cards | Pricing + 1 feature grid max | `maxTilt=6`, disable on touch |
| View Transitions | Route changes, tab swaps | Next 15 `unstable_ViewTransition` |
| Cursor trail | Hero only, disabled on touch | [Motion Primitives Cursor](https://motion-primitives.com/docs/cursor) |
| Progressive blur | Above footer + below navbar | [Progressive Blur](https://motion-primitives.com/docs/progressive-blur) |

Default spring (paste into a `transitions.ts`):
```ts
export const spring = { type:"spring", stiffness:220, damping:26, mass:0.9 };
export const silky  = { duration:0.6, ease:[0.22,1,0.36,1] };
```

---

## 5. 2026 Polish Details

**Reduced motion (non-negotiable):**
```tsx
import { useReducedMotion } from "motion/react"; // motion.dev/docs
const rm = useReducedMotion();
const t  = rm ? { duration:0 } : spring;
```
Wrap your grain, cursor trail, and parallax in the same check. ([reference](https://motion.dev/docs))

**Noise/grain done right:** 180×180 tileable PNG @ ~4% opacity, `mix-blend-mode: overlay`, position fixed, `z-index: 50`. Never SVG feTurbulence in hot paths (repaint cost). Generate at [noisepng.com] or bake in Figma.

**Spring physics that read as expensive:** stiffness 200–260, damping 24–30, mass 0.8–1.0. Avoid defaults — the 170/26 Motion default feels generic. For buttons: `{stiffness:400,damping:30}` — snappy, deliberate.

**Micro-typography:**
- Tracking `-0.02em` on display (60px+), `-0.01em` on H2, `0` on body.
- `font-feature-settings: "ss01","cv11","liga","kern"` — turns on stylistic sets (if using Inter/Geist).
- Sentence case in nav + CTAs (Linear rule), Title Case only for section eyebrows.
- Hang punctuation: `hanging-punctuation: first last` on blockquotes.

**OKLCH color science** ([Tailwind v4 colors](https://tailwindcss.com/docs/colors), [Evil Martians](https://evilmartians.com/chronicles/better-dynamic-themes-in-tailwind-with-oklch-color-magic)): Tailwind v4 is already OKLCH-native. Define your amber in OKLCH so gradients don't muddy to gray:
```css
--amber: oklch(0.72 0.16 55);       /* hero accent */
--amber-hot: oklch(0.82 0.19 62);   /* hover */
--bronze: oklch(0.48 0.10 48);      /* shadow base */
--ink:    oklch(0.14 0.01 260);     /* near-black w/ cool tint, NOT pure #000 */
```
Use `oklch()` directly in `background-image: linear-gradient(...)` — smoother than HSL by a visible margin on dark surfaces.

**Background body color:** Never `#000`. Use `oklch(0.14 0.01 260)` — adds imperceptible cool tint that makes amber pop 15% more.

---

## Priority Apply Order (do these first)
1. OKLCH tokens + non-black background (10 min).
2. Floating pill navbar with scroll spring (30 min).
3. Section `mask-image` fades + viewport vignette (15 min).
4. Magic UI BorderBeam on primary CTA + NumberTicker on stats (20 min).
5. Motion Primitives Magnetic + TextEffect on hero (20 min).
6. Grain overlay + reduced-motion guard (10 min).

Sources: [magicui.design/docs](https://magicui.design/docs) · [motion-primitives.com](https://motion-primitives.com) · [ui.aceternity.com](https://ui.aceternity.com/components) · [reactbits.dev](https://www.reactbits.dev) · [motion.dev/docs](https://motion.dev/docs) · [tailwindcss.com/docs/mask-image](https://tailwindcss.com/docs/mask-image) · [tailwindcss.com/docs/colors](https://tailwindcss.com/docs/colors) · [ui.shadcn.com/docs/tailwind-v4](https://ui.shadcn.com/docs/tailwind-v4).
