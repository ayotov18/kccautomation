# KCC Automation — Animated Landing Page Design Trends (2025–2026)

A tight synthesis focused on a Next.js 15 / React 19 / Tailwind v4 / shadcn stack for a premium, technical B2B SaaS (Rust-backed quoting + DXF/BoQ for construction).

---

## 1. Animation libraries actually shipping in 2025–2026

For a Next.js App Router + shadcn stack, the current default is **Motion** (the rebranded, framework-agnostic successor to Framer Motion). Teams still write `motion/react` but the project is now just `motion`.

- **Motion / Framer Motion** — default for React component-level animation, layout animations, `AnimatePresence`, spring physics. Works cleanly with RSC when scoped to a `"use client"` leaf.
  - https://motion.dev
  - https://github.com/motiondivision/motion
- **GSAP** — still the benchmark for complex scroll orchestration (ScrollTrigger, ScrollSmoother, SplitText, Flip). Went fully free under Webflow in 2024, which reversed the "avoid it in SaaS" posture. Use it when you need timeline sequencing Motion cannot cleanly express.
  - https://gsap.com
  - https://gsap.com/docs/v3/Plugins/ScrollTrigger/
- **Lenis** — smooth-scroll of choice (from Studio Freight/Darkroom). Pairs well with GSAP ScrollTrigger and native CSS scroll-driven animations. Every "premium feeling" site you've seen in the last 18 months is probably running it.
  - https://github.com/darkroomengineering/lenis
- **CSS scroll-driven animations + View Transitions API** — now baseline in Chromium and Safari TP. `animation-timeline: scroll()` / `view()` removes the need for JS on many reveal effects. Use for progress bars, parallax, pinned sections when you do not need cross-element choreography.
  - https://developer.mozilla.org/en-US/docs/Web/CSS/animation-timeline
  - https://developer.chrome.com/docs/web-platform/view-transitions
- **tsParticles / OGL / React Three Fiber** — for hero-background WebGL. R3F remains the React-idiomatic choice; OGL is preferred when bundle size matters.
  - https://github.com/pmndrs/react-three-fiber
  - https://github.com/oframe/ogl
- **Component registries to lift from, not compete with**: Aceternity UI, Magic UI, and shadcn's own `/blocks`. These are MIT, copy-paste, and Tailwind-native.
  - https://ui.aceternity.com
  - https://magicui.design
  - https://ui.shadcn.com/blocks

**Recommended default for KCC**: Motion + Lenis + CSS scroll-driven animations, with GSAP ScrollTrigger reserved for one or two "hero set-pieces." Skip full-page WebGL unless you already have an art director.

---

## 2. Patterns that work — with real references

- **Pinned hero with scroll-scrubbed product reveal** (GSAP ScrollTrigger + pin)
  - https://linear.app
  - https://vercel.com
  - https://cursor.com
- **Animated bento / feature grids** (hover-choreographed mini-demos inside each cell)
  - https://linear.app/features
  - https://www.raycast.com
  - https://resend.com
- **Marquee / infinite logo strips with duplicated tracks** (CSS keyframes, pausable on hover)
  - https://clerk.com
  - https://resend.com
  - https://liveblocks.io
- **Gradient mesh + noise/grain overlay backgrounds** (SVG turbulence or a 1px tiled PNG at ~5% opacity)
  - https://vercel.com
  - https://linear.app
  - https://stripe.com
- **Text reveals — line-by-line mask, letter stagger, or blur-in** (Motion `whileInView`, or SplitText)
  - https://www.framer.com
  - https://arc.net
  - https://mem.ai
- **Magnetic buttons / cursor-follow CTAs** (pointer-aware transform, damped spring)
  - https://arc.net
  - https://www.ramp.com
  - https://dub.co
- **3D card tilt with depth parallax layers** (`react-parallax-tilt` or hand-rolled `perspective()`)
  - https://linear.app
  - https://www.raycast.com
- **SVG line-drawing / schematic diagrams** (`pathLength` animation as the section scrolls into view) — this one is a natural fit for KCC's DXF story.
  - https://stripe.com
  - https://www.val.town
  - https://resend.com/emails
- **Scroll-linked number counters and metric blocks**
  - https://www.ramp.com
  - https://stripe.com
- **View Transitions for page-to-page "morph"** (product cards expanding into detail pages)
  - https://developer.chrome.com/docs/web-platform/view-transitions
  - https://astro.build (reference implementation)
- **Ambient background video / looping dashboard capture** (muted, 2–4 MB AV1/VP9, poster frame)
  - https://www.cursor.com
  - https://www.notion.so
- **Animated code blocks / terminal type-on** — relevant for your Rust-backed angle
  - https://resend.com
  - https://railway.com

---

## 3. Things to avoid in 2025–2026

- **Full-page WebGL hero shaders with no fallback** — kills LCP on mid-tier laptops and most construction-industry buyers are not on M-series Macs.
- **Lottie everywhere** — still fine for one or two icons, but a 900 KB JSON blob just to animate a checkmark is 2021 thinking.
- **Parallax on every section** — now reads as template work; reserve it for one anchor moment.
- **AOS.js / WOW.js / data-aos attributes** — dated, janky on mobile, and unnecessary now that `whileInView` and CSS scroll-timeline exist.
- **Autoplaying unmuted video, scroll-hijacking, cursor-takeover** — all hostile and all hurt CWV / bounce.
- **Gradient-orb-blur on absolutely everything** — 2023's "Vercel look" is now the stock-template look; pair it with a stronger structural element (grid, schematic) to avoid pastiche.
- **Heavy libraries loaded eagerly on the landing route** — GSAP plugins, R3F, and Lottie all need `next/dynamic` with `ssr: false` and usually an `IntersectionObserver` gate.
- **Animations that run during the first 2.5 s** — they sabotage LCP and INP. Defer until after paint, honor `prefers-reduced-motion`.

---

## 4. Aesthetic direction for premium industrial / construction B2B

The sweet spot is **"technical drawing meets Swiss editorial"** — not the beige trade-show look, not the generic purple-gradient SaaS look.

- **Palette options**:
  - *Graphite + Safety Amber*: `#0B0B0F` / `#F5F5F2` / accent `#FF7A1A` or `#FFB020`. Reads as jobsite without being literal.
  - *Blueprint*: near-black `#0A1628`, paper-cream `#F4EFE6`, cyan line `#7DD3FC`. Uses the DXF/CAD connotation directly.
  - *Concrete + Chlorophyll*: `#1A1A1A`, `#E9E6DF`, accent `#8EE06B` for the "automation/speed" signal.
- **Typography pairings** (all modern variable fonts):
  - *Inter Display* (headline) + *Inter* (body) + *JetBrains Mono* (code/measurements). Safe, production default.
  - *Geist* + *Geist Mono* (Vercel's stack) — feels current, ships from `next/font`.
  - *Söhne* or *Söhne Mono* (paid, but what Linear / Stripe / OpenAI use) — if budget allows.
  - *Neue Haas Grotesk* / *Neue Haas Unica* for a more editorial, architectural feel.
- **Grid & layout references**:
  - https://linear.app — dark, precise, 12-col with generous whitespace
  - https://www.ramp.com — editorial grid, large numerals, industrial restraint
  - https://www.rivet.work — hardware/industrial SaaS, technical palette
  - https://www.cursor.com — dark with single accent hue and code-as-hero
  - https://www.val.town — annotated schematic aesthetic, good for DXF analog
- **Texture signals to lean on**: faint dotted/engineering grid background, 1px hairlines with cross-tick intersections, monospace micro-labels ("01 / 04", "§ BOQ"), measurement dimension lines rendered in SVG on hover.

---

## 6. Concrete component inventory (what to plan for)

- **Hero**
  - Left: tight headline (2–3 lines max, display weight), monospace eyebrow, dual CTA (primary solid + ghost "See how it works").
  - Right: either (a) scroll-scrubbed product video in a bezel-less frame, or (b) live SVG schematic that animates drawing a countertop outline into a BoQ table. Strongly recommend (b) for KCC — differentiates from every other SaaS hero.
  - Trust bar directly below (6–8 customer wordmarks, grayscale, marquee on mobile only).
- **Logo / social-proof strip** — duplicated-track marquee, pausable on hover, grayscale with a hover color restore.
- **Feature bento** — 5–7 cells, asymmetric grid, each cell owns one mini-animation (DXF parse, quote generation, margin calculator, Rust speed gauge). Aceternity and Magic UI have near-drop-in references.
- **"How it works" scroll section** — pinned, 3–4 steps revealed as you scroll. GSAP ScrollTrigger + a single SVG pipeline that fills left-to-right.
- **Stack / integrations showcase** — orbit or grid of logos (ERP, accounting, CAD formats: DXF, DWG, STEP). Orbit pattern (Magic UI) is on-trend but almost overused; a static precision grid may feel more premium for industrial.
- **Live metric / speed section** — animated counter ("12,400 quotes generated this week", "avg. drawing parsed in 420 ms"). Real numbers only.
- **Testimonial / case study** — single large quote with a real jobsite photo + two-column metric callouts, OR a 3-up card grid with company logo, role, measurable outcome (quote time down, margin up).
- **Pricing / CTA band** — dark contrast section, one-line value prop, primary CTA, secondary "Talk to sales" for enterprise.
- **Footer** — large wordmark, 4-column nav, newsletter input with animated focus ring, SOC2/compliance badges, status-page link. Keep it editorial, not cluttered.

---

## Extra repo references worth bookmarking

- https://github.com/shadcn-ui/ui
- https://github.com/magicuidesign/magicui
- https://github.com/aceternity/ui
- https://github.com/pacocoursey/next-themes
- https://github.com/studio-freight/lenis
- https://github.com/pmndrs/drei
