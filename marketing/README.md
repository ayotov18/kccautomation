# KCC Marketing

Landing page for KCC Automation — Next.js 15 + Tailwind v4 + shadcn primitives + Motion (Framer Motion) + Lucide icons.

Lives at `marketing/` inside the main repo. Deploys independently.

## Local dev

```bash
npm install
npm run dev   # http://localhost:3100
```

## Generating assets

Images (~2 min total, OpenRouter `openai/gpt-5.4-image-2`):

```bash
node scripts/gen-images.mjs            # all 6
node scripts/gen-images.mjs hero cta   # subset
```

Videos (~5–10 min per clip, KIE.ai `bytedance/seedance-2`, image-to-video):

```bash
node scripts/gen-videos.mjs            # all 3, uses S3 presigned URLs for ref upload
```

Outputs land in `public/assets/gen/`. Commit the generated files — regen on demand.

## Stack

- Next.js 15 (App Router, standalone output)
- React 19
- Tailwind v4 (`@tailwindcss/postcss`, no config file — `@theme` in `globals.css`)
- Motion (ex-Framer Motion) for scroll + stagger reveals
- Lucide React for icons
- `class-variance-authority` + `tailwind-merge` + `clsx` for the shadcn-style `cn()`

## Structure

```
app/
  layout.tsx            # root, loads Geist + Geist Mono
  page.tsx              # composes all sections
  globals.css           # Tailwind v4 + design tokens
components/
  nav.tsx               # sticky header, amber CTA
  hero.tsx              # fullscreen, generated bg image
  marquee.tsx           # monospace stack strip
  features.tsx          # 3 cards, non-technical copy
  pipeline.tsx          # animated 6-node schematic
  bento.tsx             # 5-tile "under the hood"
  testimonial.tsx       # pull quote, dusk background
  stack.tsx             # engineering detail table
  security.tsx          # 3 data posture tiles
  faq.tsx               # accordion, 4 items
  cta.tsx               # closer with generated bg
  footer.tsx            # 4 columns
  ui/
    button.tsx          # shadcn-style, CVA variants
lib/
  cn.ts                 # class merger
scripts/
  gen-images.mjs        # OpenRouter image gen
  gen-videos.mjs        # KIE seedance-2 image-to-video
public/assets/gen/      # generated images + videos
COPY.md                 # authoritative landing page copy
```

Generated images (6) + videos (3) live under `public/assets/gen/`. The hero / CTA / testimonial use the image as a background; videos are available for later swap-in once tuned.

## Copy

Single source of truth: `COPY.md`. Don't edit component copy directly — change the markdown, then propagate.
