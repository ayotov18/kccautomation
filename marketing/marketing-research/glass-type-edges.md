# Glass, Type, Edges — KCC Automation polish research

Dark premium B2B construction SaaS. Next.js 15 + Tailwind v4 + Motion. All snippets are drop-in.

---

## 1. Edge bleeding / seamless section transitions

Fix is never one technique — combine **color bleed** (shared gradient), **luminance bleed** (mask fade), and **focal bleed** (progressive blur).

### A. Shared-gradient continuity (default, 80% of cases)
Bottom of A = top of B. Paint is continuous, no seam.
```css
/* A */ background: linear-gradient(to bottom,#0a0a0b,#0f0d0a);
/* B */ background: linear-gradient(to bottom,#0f0d0a,#0a0a0b);
```

### B. Mask double-fade + negative margin (dissolve)
([MDN mask-image](https://developer.mozilla.org/en-US/docs/Web/CSS/mask-image))
```css
.bleed-top   { mask-image: linear-gradient(to bottom,transparent,#000 12%); }
.bleed-bottom{ mask-image: linear-gradient(to top,transparent,#000 12%); }
.section-next{ margin-top:-6rem; position:relative; z-index:1; }
```

### C. Progressive blur band (the Apple seam)
120px tall absolute div, 5 stacked backdrop-blur layers. ([kennethnym](https://kennethnym.com/blog/progressive-blur-in-css/), [Motion Primitives](https://motion-primitives.com/docs/progressive-blur))
```css
.pb-band{position:absolute;inset:auto 0 -60px 0;height:120px;pointer-events:none}
.pb-band>div{position:absolute;inset:0}
.pb-band>div:nth-child(1){backdrop-filter:blur(1px); mask:linear-gradient(#0000,#000 10%,#000 30%,#0000 40%)}
.pb-band>div:nth-child(2){backdrop-filter:blur(2px); mask:linear-gradient(#0000 10%,#000 20%,#000 40%,#0000 50%)}
.pb-band>div:nth-child(3){backdrop-filter:blur(4px); mask:linear-gradient(#0000 20%,#000 40%,#000 60%,#0000 70%)}
.pb-band>div:nth-child(4){backdrop-filter:blur(8px); mask:linear-gradient(#0000 40%,#000 60%,#000 80%,#0000 90%)}
.pb-band>div:nth-child(5){backdrop-filter:blur(16px);mask:linear-gradient(#0000 60%,#000 80%)}
```

### D. Scrim (flat veil, anti-banding, cheap)
```css
.scrim{position:absolute;inset:auto 0 0 0;height:240px;pointer-events:none;
  background:linear-gradient(to bottom,transparent,#08080a 85%)}
```

### E. Accent gleam crossing the seam
```css
.gleam{position:absolute;left:20%;bottom:-20%;width:700px;aspect-ratio:1;
  background:radial-gradient(closest-side,oklch(.72 .16 55/.18),transparent 70%);
  filter:blur(40px);pointer-events:none;z-index:1}
```

**Decision:** shared gradient = default. Mask double-fade = backgrounds genuinely differ. Progressive blur = once, at nav + above footer (GPU-heavy). Scrim = over imagery. Gleam = signature moment only.

---

## 2. Liquid Glass on the web (Apple iOS 26 / macOS 26)

**What it actually is.** WWDC 2025 material is not glassmorphism. Four optical passes: (1) backdrop **blur + saturation** (~140–180%), (2) **refractive edge** — bright 1px inset top, dark 1px inset bottom (light model), (3) **specular highlight** tracking pointer/scroll (moving radial), (4) **chromatic dispersion** — R/G/B offset at curved edges via SVG `feDisplacementMap`. ([CSS-Tricks](https://css-tricks.com/getting-clarity-on-apples-liquid-glass/), [LogRocket](https://blog.logrocket.com/how-create-liquid-glass-effects-css-and-svg/))

**Browser reality.** Safari + Firefox don't support SVG filters feeding `backdrop-filter`; Chromium does. Ship CSS approximation; enhance with SVG inside `@supports`.

**Existing libs worth stealing from:**
- [rdev/liquid-glass-react](https://github.com/rdev/liquid-glass-react) — defaults: `displacementScale=70, blurAmount=0.0625, saturation=140, aberrationIntensity=2, elasticity=0.15, cornerRadius=999`. Modes: `standard|polar|prominent|shader`.
- [nikdelvin/liquid-glass](https://github.com/nikdelvin/liquid-glass) — pure CSS+SVG recreation.
- [clayharmon/webgl-liquid-glass](https://github.com/clayharmon/webgl-liquid-glass) — WebGL shader for navbars.
- [@specy/liquid-glass-react](https://www.npmjs.com/package/@specy/liquid-glass-react) — chromatic dispersion prop.

**Drop-in CSS (cross-browser).** Ship this. No SVG dependency.
```css
.liquid-glass{
  position: relative;
  border-radius: 20px;
  background: color-mix(in oklch, white 6%, transparent);
  backdrop-filter: blur(24px) saturate(180%);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
  /* refractive edge: bright top inset + dark bottom inset */
  box-shadow:
    inset 0 1px 0 0 rgb(255 255 255 / 0.22),   /* top light */
    inset 0 -1px 0 0 rgb(0 0 0 / 0.35),        /* bottom shadow */
    inset 0 0 0 1px rgb(255 255 255 / 0.08),   /* hairline */
    0 20px 60px -20px rgb(0 0 0 / 0.6);        /* cast shadow */
}
.liquid-glass::before{ /* specular — moves on pointer (see below) */
  content:""; position:absolute; inset:0; border-radius:inherit; pointer-events:none;
  background: radial-gradient(120px circle at var(--mx,30%) var(--my,0%),
    rgb(255 255 255 / 0.18), transparent 60%);
}
```

**Pointer-tracked specular (React).**
```tsx
const onMove = (e: React.PointerEvent<HTMLDivElement>) => {
  const r = e.currentTarget.getBoundingClientRect();
  e.currentTarget.style.setProperty("--mx", `${((e.clientX-r.left)/r.width)*100}%`);
  e.currentTarget.style.setProperty("--my", `${((e.clientY-r.top)/r.height)*100}%`);
};
return <div className="liquid-glass" onPointerMove={onMove}>...</div>;
```

**Chromatic dispersion (Chromium-only, progressive enhancement).**
```html
<svg style="position:absolute;width:0;height:0">
  <filter id="lg-disp" x="0" y="0" width="100%" height="100%">
    <feTurbulence type="fractalNoise" baseFrequency="0.008" numOctaves="2" seed="4"/>
    <feDisplacementMap in="SourceGraphic" scale="12"/>
  </filter>
</svg>
```
```css
@supports (backdrop-filter: url(#lg-disp)){
  .liquid-glass{ backdrop-filter: blur(24px) saturate(180%) url(#lg-disp); }
}
```

**Dark-mode tuning** for KCC's near-black: drop tint to `white 4%`, bump top-inset to `.28` — the light ring is what sells the material.

---

## 3. Modern typography for 2026 landing pages

**Stack.** "One display serif + one grotesk + one mono" is the 2026 trend (Vercel, Linear, Arc, Cursor, Anthropic). KCC pick:
- **Display serif (accent):** [Instrument Serif](https://fonts.google.com/specimen/Instrument+Serif) — italic on 1 hero word.
- **Grotesk (workhorse):** [Geist](https://vercel.com/font) or [Inter v4](https://rsms.me/inter/).
- **Mono:** [Geist Mono](https://vercel.com/font) or [JetBrains Mono](https://www.jetbrains.com/lp/mono/).
- **Paid alt:** PP Neue Montreal, Söhne, Neue Haas Grotesk Display.

**Fluid scale** via [utopia.fyi/type/calculator](https://utopia.fyi/type/calculator/) (base 16→18, ratio 1.2→1.333, 360→1440):
```css
@theme {
  --font-sans: "Geist", ui-sans-serif, system-ui;
  --font-serif: "Instrument Serif", ui-serif, Georgia;
  --font-mono: "Geist Mono", ui-monospace;
  --text-xs:   clamp(0.75rem, 0.72rem + 0.15vw, 0.8125rem);
  --text-sm:   clamp(0.875rem, 0.83rem + 0.22vw, 0.9375rem);
  --text-base: clamp(1rem, 0.95rem + 0.25vw, 1.125rem);
  --text-lg:   clamp(1.125rem, 1.05rem + 0.38vw, 1.375rem);
  --text-xl:   clamp(1.375rem, 1.25rem + 0.62vw, 1.75rem);
  --text-2xl:  clamp(1.75rem, 1.5rem + 1.25vw, 2.5rem);
  --text-3xl:  clamp(2.25rem, 1.85rem + 2vw, 3.5rem);
  --text-4xl:  clamp(3rem, 2.4rem + 3vw, 5rem);
  --text-5xl:  clamp(3.75rem, 2.8rem + 4.75vw, 7rem);
}
```

**Tracking (letter-spacing) rules.** Display bigger = tighter.
- ≥64px: `-0.035em`
- 40–63px: `-0.025em`
- 24–39px: `-0.015em`
- 16–23px: `-0.01em`
- <16px body: `0`
- All-caps eyebrows: `+0.08em`

**Line-height.** Display `1.02–1.08` (never 1.2+ on hero). Subheads `1.15`. Body `1.55–1.65`. Mono spec lines `1.4`.

**Alignment.** Hero = **left**, always, on B2B. Centered hero reads consumer/marketing-fluff. Eyebrow left-aligned above. Subhead `max-w-[52ch]`. Captions/kickers `font-mono uppercase text-xs tracking-[0.14em]`.

**Weights.** Display 500 (not 700 — 700 looks cheap on Geist/Inter at large sizes). Body 400. UI labels 500. Mono 400. Italic serif only for accent words.

**Features.**
```css
body { font-feature-settings: "ss01","cv11","liga","kern","calt"; text-rendering: optimizeLegibility; }
.tabular { font-variant-numeric: tabular-nums; }  /* stats, pricing, counters */
```

---

## 4. Word-splitter fix

Your bug: animated `inline-block` spans need **a per-word clipping wrapper with `overflow:hidden`**. Without it, at `clamp(1.75rem,4vw,3rem)` the transform pushes descenders past the line box; Safari also line-breaks mid-word because `inline-block` spans don't behave like text for line-breaking.

**Three fixes (priority order):**
1. Wrap each word in `<span class="clip">` with `overflow:hidden` + `display:inline-block` + `vertical-align:bottom`.
2. Preserve spaces — emit `" "` between spans, do NOT use flex gap.
3. Inner animated span = `inline-block` + `will-change:transform`.

**Canonical impl.** Mirrors [Motion splitText](https://motion.dev/docs/split-text), [web.dev split-text](https://web.dev/building-split-text-animations/), Motion Primitives, Magic UI blur-fade.
```tsx
"use client";
import { motion } from "motion/react";
export function TextReveal({ text, className = "", delay = 0 }: { text: string; className?: string; delay?: number }) {
  const words = text.split(" ");
  return (
    <span className={`inline ${className}`} aria-label={text}>
      {words.map((w, i) => (
        <span key={i} aria-hidden
          className="inline-block overflow-hidden align-bottom leading-[1.05] pb-[0.12em]">
          <motion.span
            className="inline-block will-change-transform"
            initial={{ y: "110%" }}
            whileInView={{ y: "0%" }}
            viewport={{ once: true, margin: "-10%" }}
            transition={{ duration: 0.7, ease: [0.22, 1, 0.36, 1], delay: delay + i * 0.04 }}>
            {w}
          </motion.span>
          {i < words.length - 1 && " "}
        </span>
      ))}
    </span>
  );
}
```

**Why this works:**
- `pb-[0.12em]` on the clip expands the clip box for descenders (g, y, p) — without it the bottom gets shaved.
- ` ` (NBSP) keeps spacing even when the inner span is translated.
- `align-bottom` forces all clip boxes onto one baseline — without it, Safari staircase-lays mismatched x-heights.
- `leading-[1.05]` matches display line-height so clips don't stack vertically on wrap.
- `viewport margin: -10%` prevents double-trigger on long heroes.

**If text still wraps mid-word:** your parent has `break-word` or `word-break: break-all`. Set `className="text-balance [&_span]:break-normal"` on the heading wrapper. Use `text-wrap: balance` (Chrome/Safari 2024+) on all display headings — this single property fixes 80% of "broken looking" headings.

---

## Priority apply order (fastest visible win first)

1. **Fix the text splitter** — swap to the canonical `TextReveal` above + add `text-wrap: balance` to all `h1/h2`. (15 min) — kills the "broken text" complaint immediately.
2. **Shared-gradient continuity** between sections + `-mt-24` overlap with mask fade. (20 min) — kills the seam.
3. **Typography tokens** — add the Tailwind v4 `@theme` block, set `-0.025em` tracking on display. (15 min) — the whole page levels up.
4. **Liquid Glass on nav + pricing cards + CTA** — drop-in `.liquid-glass` class. (30 min) — this is the "premium" signal.
5. **Progressive blur band** under the fixed nav and above the footer. (15 min) — the Apple signature.
6. **Accent gleam** crossing one section seam (hero → features). (10 min) — one signature moment, not five.
7. **SVG chromatic dispersion** inside `@supports` — Chromium-only enhancement on hero glass card. (20 min) — optional.

Total: ~2 hours end-to-end.

Sources: [liquid-glass-react](https://github.com/rdev/liquid-glass-react) · [nikdelvin/liquid-glass](https://github.com/nikdelvin/liquid-glass) · [CSS-Tricks Liquid Glass](https://css-tricks.com/getting-clarity-on-apples-liquid-glass/) · [LogRocket Liquid Glass](https://blog.logrocket.com/how-create-liquid-glass-effects-css-and-svg/) · [Progressive blur in CSS](https://kennethnym.com/blog/progressive-blur-in-css/) · [Motion Primitives Progressive Blur](https://motion-primitives.com/docs/progressive-blur) · [Motion splitText](https://motion.dev/docs/split-text) · [web.dev split-text](https://web.dev/building-split-text-animations/) · [Utopia type calculator](https://utopia.fyi/type/calculator/) · [Geist font](https://vercel.com/font) · [Instrument Serif](https://fonts.google.com/specimen/Instrument+Serif) · [MDN mask-image](https://developer.mozilla.org/en-US/docs/Web/CSS/mask-image).
