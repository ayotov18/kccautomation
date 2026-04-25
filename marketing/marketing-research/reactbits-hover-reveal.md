# ReactBits, Hover Animations, Border Beams & Scroll Reveals — Research & Ready-to-Paste Source

**Target stack:** Next.js 15 + Tailwind v4 + Motion (motion/react)
**Aesthetic:** Dark industrial B2B construction SaaS — bg `oklch(0.14 0.01 260)`, amber `oklch(0.72 0.16 55)`, no rainbow, restrained density
**Research date:** 2026-04-24

---

## 0. License note (ReactBits, `DavidHDev/react-bits`)

- License: **MIT + Commons Clause**
- You may copy, modify, paste into your app, and ship commercially.
- You may NOT resell the components as a library, bundle, or port.
- Attribution is not strictly required but nice (we are copying into product code, so we're compliant).
- Source: `https://raw.githubusercontent.com/DavidHDev/react-bits/main/src/ts-tailwind/Backgrounds/<Name>/<Name>.tsx`

---

## 1. Background Component Evaluation

| Component | MIT-copyable | LOC | Runtime | Dependency | Fit 1-5 | Notes |
|---|---|---|---|---|---|---|
| Aurora | Yes | 209 | WebGL2 (ogl) | `ogl` | 3 | Beautiful but wavy blob feel — too "consumer" for B2B. Monochromize works. |
| **Threads** | Yes | 228 | WebGL (ogl) | `ogl` | **5** | Horizontal filament lines, engineered look, single color, mouse-reactive. Perfect for hero. |
| **LetterGlitch** | Yes | 230 | Canvas 2D | none | **4** | Terminal matrix feel. Great for a "data/systems" section, not hero. Pure canvas, no WebGL. |
| **Silk** | Yes | 151 | WebGL (R3F) | `@react-three/fiber`, `three` | **5** | Subtle moving fabric pattern, single-color tintable. Restrained, premium. |
| Squares | — | — | — | — | — | Does not exist under this name in `react-bits`. Closest: `ShapeGrid`, `RippleGrid`, `GridMotion`. |
| **DotGrid** | Yes | 290 | Canvas 2D | `gsap`, `gsap/InertiaPlugin` | **5** | Interactive dot grid with pointer inertia. Pure 2D. Extremely industrial. |
| Iridescence | Yes | 138 | WebGL (ogl) | `ogl` | 2 | Rainbow by default — wrong vibe for B2B even when monochromized. |
| LiquidChrome | Yes | 168 | WebGL (ogl) | `ogl` | 2 | Automotive chrome look, too flashy. |
| Lightning | Yes | 189 | WebGL (ogl) | `ogl` | 3 | Cool single color, but high-energy — fine for an activation section, wrong for hero. |
| Plasma | Yes | 248 | WebGL (ogl) | `ogl` | 2 | Too sci-fi. |
| Galaxy | Yes | 351 | WebGL (ogl) | `ogl` | 1 | Starfield — off-brand for construction. |
| Beams | Yes | 372 | WebGL (R3F) | `@react-three/fiber`, `three` | 3 | Architectural light beams, beautiful but heavy (372 LOC + R3F). |
| Hyperspeed | Yes | 40KB | WebGL | heavy | 1 | Driving tunnel effect — completely wrong for B2B SaaS. |

**Picks for this project:** `Threads` (hero), `Silk` (secondary section), `DotGrid` (feature section), `LetterGlitch` (optional "data" section).

### Install once:
```bash
npm i ogl @react-three/fiber three gsap
```

---

## 1A. `Threads.tsx` — HERO BACKGROUND (recommended #1)

Engineered horizontal filaments. Amber-tuned, subtle amplitude, mouse-reactive. Paste as `components/backgrounds/Threads.tsx`.

```tsx
'use client';
import { useEffect, useRef } from 'react';
import { Renderer, Program, Mesh, Triangle, Color } from 'ogl';

interface ThreadsProps {
  color?: [number, number, number]; // 0..1
  amplitude?: number;
  distance?: number;
  enableMouseInteraction?: boolean;
  className?: string;
}

const vertexShader = `
attribute vec2 position;
attribute vec2 uv;
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = vec4(position, 0.0, 1.0);
}
`;

const fragmentShader = `
precision highp float;
uniform float iTime;
uniform vec3 iResolution;
uniform vec3 uColor;
uniform float uAmplitude;
uniform float uDistance;
uniform vec2 uMouse;
#define PI 3.1415926538
const int u_line_count = 40;
const float u_line_width = 7.0;
const float u_line_blur = 10.0;

float Perlin2D(vec2 P) {
  vec2 Pi = floor(P);
  vec4 Pf_Pfmin1 = P.xyxy - vec4(Pi, Pi + 1.0);
  vec4 Pt = vec4(Pi.xy, Pi.xy + 1.0);
  Pt = Pt - floor(Pt * (1.0 / 71.0)) * 71.0;
  Pt += vec2(26.0, 161.0).xyxy;
  Pt *= Pt;
  Pt = Pt.xzxz * Pt.yyww;
  vec4 hash_x = fract(Pt * (1.0 / 951.135664));
  vec4 hash_y = fract(Pt * (1.0 / 642.949883));
  vec4 grad_x = hash_x - 0.49999;
  vec4 grad_y = hash_y - 0.49999;
  vec4 grad_results = inversesqrt(grad_x * grad_x + grad_y * grad_y)
    * (grad_x * Pf_Pfmin1.xzxz + grad_y * Pf_Pfmin1.yyww);
  grad_results *= 1.4142135623730950;
  vec2 blend = Pf_Pfmin1.xy * Pf_Pfmin1.xy * Pf_Pfmin1.xy
    * (Pf_Pfmin1.xy * (Pf_Pfmin1.xy * 6.0 - 15.0) + 10.0);
  vec4 blend2 = vec4(blend, vec2(1.0 - blend));
  return dot(grad_results, blend2.zxzx * blend2.wwyy);
}

float pixel(float count, vec2 resolution) {
  return (1.0 / max(resolution.x, resolution.y)) * count;
}

float lineFn(vec2 st, float width, float perc, float offset, vec2 mouse, float time, float amplitude, float distance) {
  float split_offset = (perc * 0.4);
  float split_point = 0.1 + split_offset;
  float amplitude_normal = smoothstep(split_point, 0.7, st.x);
  float amplitude_strength = 0.5;
  float finalAmplitude = amplitude_normal * amplitude_strength
    * amplitude * (1.0 + (mouse.y - 0.5) * 0.2);
  float time_scaled = time / 10.0 + (mouse.x - 0.5) * 1.0;
  float blur = smoothstep(split_point, split_point + 0.05, st.x) * perc;
  float xnoise = mix(
    Perlin2D(vec2(time_scaled, st.x + perc) * 2.5),
    Perlin2D(vec2(time_scaled, st.x + time_scaled) * 3.5) / 1.5,
    st.x * 0.3
  );
  float y = 0.5 + (perc - 0.5) * distance + xnoise / 2.0 * finalAmplitude;
  float line_start = smoothstep(
    y + (width / 2.0) + (u_line_blur * pixel(1.0, iResolution.xy) * blur),
    y, st.y
  );
  float line_end = smoothstep(
    y, y - (width / 2.0) - (u_line_blur * pixel(1.0, iResolution.xy) * blur),
    st.y
  );
  return clamp(
    (line_start - line_end) * (1.0 - smoothstep(0.0, 1.0, pow(perc, 0.3))),
    0.0, 1.0
  );
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
  vec2 uv = fragCoord / iResolution.xy;
  float line_strength = 1.0;
  for (int i = 0; i < u_line_count; i++) {
    float p = float(i) / float(u_line_count);
    line_strength *= (1.0 - lineFn(
      uv,
      u_line_width * pixel(1.0, iResolution.xy) * (1.0 - p),
      p, (PI * 1.0) * p, uMouse, iTime, uAmplitude, uDistance
    ));
  }
  float colorVal = 1.0 - line_strength;
  fragColor = vec4(uColor * colorVal, colorVal);
}

void main() {
  mainImage(gl_FragColor, gl_FragCoord.xy);
}
`;

// amber oklch(0.72 0.16 55) ~= #e8a64a normalized
const AMBER: [number, number, number] = [0.91, 0.65, 0.29];

export default function Threads({
  color = AMBER,
  amplitude = 0.8,           // restrained for B2B
  distance = 0.2,
  enableMouseInteraction = true,
  className = 'absolute inset-0',
}: ThreadsProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const rafRef = useRef<number>(0);

  useEffect(() => {
    if (!containerRef.current) return;
    const container = containerRef.current;

    const renderer = new Renderer({ alpha: true });
    const gl = renderer.gl;
    gl.clearColor(0, 0, 0, 0);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    container.appendChild(gl.canvas);

    const geometry = new Triangle(gl);
    const program = new Program(gl, {
      vertex: vertexShader,
      fragment: fragmentShader,
      uniforms: {
        iTime: { value: 0 },
        iResolution: {
          value: new Color(gl.canvas.width, gl.canvas.height, gl.canvas.width / gl.canvas.height),
        },
        uColor: { value: new Color(...color) },
        uAmplitude: { value: amplitude },
        uDistance: { value: distance },
        uMouse: { value: new Float32Array([0.5, 0.5]) },
      },
    });
    const mesh = new Mesh(gl, { geometry, program });

    const resize = () => {
      const { clientWidth, clientHeight } = container;
      renderer.setSize(clientWidth, clientHeight);
      program.uniforms.iResolution.value.r = clientWidth;
      program.uniforms.iResolution.value.g = clientHeight;
      program.uniforms.iResolution.value.b = clientWidth / clientHeight;
    };
    window.addEventListener('resize', resize);
    resize();

    let currentMouse = [0.5, 0.5];
    let targetMouse = [0.5, 0.5];
    const onMove = (e: MouseEvent) => {
      const r = container.getBoundingClientRect();
      targetMouse = [(e.clientX - r.left) / r.width, 1 - (e.clientY - r.top) / r.height];
    };
    const onLeave = () => { targetMouse = [0.5, 0.5]; };
    if (enableMouseInteraction) {
      container.addEventListener('mousemove', onMove);
      container.addEventListener('mouseleave', onLeave);
    }

    const tick = (t: number) => {
      if (enableMouseInteraction) {
        currentMouse[0] += 0.05 * (targetMouse[0] - currentMouse[0]);
        currentMouse[1] += 0.05 * (targetMouse[1] - currentMouse[1]);
        program.uniforms.uMouse.value[0] = currentMouse[0];
        program.uniforms.uMouse.value[1] = currentMouse[1];
      }
      program.uniforms.iTime.value = t * 0.001;
      renderer.render({ scene: mesh });
      rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(rafRef.current);
      window.removeEventListener('resize', resize);
      container.removeEventListener('mousemove', onMove);
      container.removeEventListener('mouseleave', onLeave);
      if (container.contains(gl.canvas)) container.removeChild(gl.canvas);
      gl.getExtension('WEBGL_lose_context')?.loseContext();
    };
  }, [color, amplitude, distance, enableMouseInteraction]);

  return <div ref={containerRef} className={className} aria-hidden />;
}
```

**Usage:**
```tsx
<section className="relative min-h-[90vh] bg-[oklch(0.14_0.01_260)] overflow-hidden">
  <Threads className="absolute inset-0 opacity-60" amplitude={0.7} distance={0.15} />
  <div className="relative z-10">{/* hero content */}</div>
</section>
```

---

## 1B. `Silk.tsx` — SECONDARY SECTION BACKGROUND (recommended #2)

Subtle animated fabric pattern. Amber-tinted, very restrained. Needs `@react-three/fiber` + `three`.

```tsx
'use client';
import { forwardRef, useMemo, useRef, useLayoutEffect } from 'react';
import { Canvas, useFrame, useThree, type RootState } from '@react-three/fiber';
import { Color, type Mesh, ShaderMaterial, type IUniform } from 'three';

const hexToNormalizedRGB = (hex: string): [number, number, number] => {
  const c = hex.replace('#', '');
  return [
    parseInt(c.slice(0, 2), 16) / 255,
    parseInt(c.slice(2, 4), 16) / 255,
    parseInt(c.slice(4, 6), 16) / 255,
  ];
};

interface SilkUniforms {
  uSpeed: { value: number };
  uScale: { value: number };
  uNoiseIntensity: { value: number };
  uColor: { value: Color };
  uRotation: { value: number };
  uTime: { value: number };
  [k: string]: IUniform;
}

const vertexShader = `
varying vec2 vUv;
varying vec3 vPosition;
void main() {
  vPosition = position;
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const fragmentShader = `
varying vec2 vUv;
uniform float uTime;
uniform vec3  uColor;
uniform float uSpeed;
uniform float uScale;
uniform float uRotation;
uniform float uNoiseIntensity;

const float e = 2.71828182845904523536;
float noise(vec2 texCoord) {
  float G = e;
  vec2 r = (G * sin(G * texCoord));
  return fract(r.x * r.y * (1.0 + texCoord.x));
}
vec2 rotateUvs(vec2 uv, float angle) {
  float c = cos(angle);
  float s = sin(angle);
  return mat2(c, -s, s, c) * uv;
}

void main() {
  float rnd = noise(gl_FragCoord.xy);
  vec2 uv  = rotateUvs(vUv * uScale, uRotation);
  vec2 tex = uv * uScale;
  float tOffset = uSpeed * uTime;
  tex.y += 0.03 * sin(8.0 * tex.x - tOffset);
  float pattern = 0.6 + 0.4 * sin(
    5.0 * (tex.x + tex.y + cos(3.0 * tex.x + 5.0 * tex.y) + 0.02 * tOffset) +
    sin(20.0 * (tex.x + tex.y - 0.1 * tOffset))
  );
  vec4 col = vec4(uColor, 1.0) * vec4(pattern) - rnd / 15.0 * uNoiseIntensity;
  col.a = 1.0;
  gl_FragColor = col;
}
`;

const SilkPlane = forwardRef<Mesh, { uniforms: SilkUniforms }>(function SilkPlane({ uniforms }, ref) {
  const { viewport } = useThree();
  useLayoutEffect(() => {
    const m = ref as React.MutableRefObject<Mesh | null>;
    if (m.current) m.current.scale.set(viewport.width, viewport.height, 1);
  }, [ref, viewport]);
  useFrame((_s: RootState, delta: number) => {
    const m = ref as React.MutableRefObject<Mesh | null>;
    if (m.current) {
      const mat = m.current.material as ShaderMaterial & { uniforms: SilkUniforms };
      mat.uniforms.uTime.value += 0.1 * delta;
    }
  });
  return (
    <mesh ref={ref}>
      <planeGeometry args={[1, 1, 1, 1]} />
      <shaderMaterial uniforms={uniforms} vertexShader={vertexShader} fragmentShader={fragmentShader} />
    </mesh>
  );
});

export interface SilkProps {
  speed?: number;
  scale?: number;
  color?: string;
  noiseIntensity?: number;
  rotation?: number;
  className?: string;
}

// Dark bronze amber, muted - fits oklch(0.14 0.01 260) bg
const DEFAULT_COLOR = '#3a2a1a';

export default function Silk({
  speed = 3,
  scale = 1,
  color = DEFAULT_COLOR,
  noiseIntensity = 1.2,
  rotation = 0,
  className = 'absolute inset-0',
}: SilkProps) {
  const ref = useRef<Mesh>(null);
  const uniforms = useMemo<SilkUniforms>(() => ({
    uSpeed: { value: speed },
    uScale: { value: scale },
    uNoiseIntensity: { value: noiseIntensity },
    uColor: { value: new Color(...hexToNormalizedRGB(color)) },
    uRotation: { value: rotation },
    uTime: { value: 0 },
  }), [speed, scale, noiseIntensity, color, rotation]);

  return (
    <div className={className} aria-hidden>
      <Canvas dpr={[1, 2]} frameloop="always">
        <SilkPlane ref={ref} uniforms={uniforms} />
      </Canvas>
    </div>
  );
}
```

**Usage:**
```tsx
<section className="relative bg-[oklch(0.14_0.01_260)]">
  <Silk className="absolute inset-0 opacity-40" color="#3a2718" speed={2} />
  <div className="relative z-10">...</div>
</section>
```

---

## 1C. `DotGrid.tsx` — INTERACTIVE INDUSTRIAL GRID (recommended #3)

Canvas 2D only. Mouse inertia requires `gsap` + `gsap/InertiaPlugin`. Incredibly on-brand for a construction-tech product.

```tsx
'use client';
import React, { useRef, useEffect, useCallback, useMemo } from 'react';
import { gsap } from 'gsap';
import { InertiaPlugin } from 'gsap/InertiaPlugin';

gsap.registerPlugin(InertiaPlugin);

const throttle = (fn: (...a: any[]) => void, limit: number) => {
  let last = 0;
  return function (this: any, ...args: any[]) {
    const now = performance.now();
    if (now - last >= limit) { last = now; fn.apply(this, args); }
  };
};

interface Dot { cx: number; cy: number; xOffset: number; yOffset: number; _inertiaApplied: boolean; }

export interface DotGridProps {
  dotSize?: number;
  gap?: number;
  baseColor?: string;    // idle color
  activeColor?: string;  // pointer-proximate color
  proximity?: number;
  speedTrigger?: number;
  shockRadius?: number;
  shockStrength?: number;
  maxSpeed?: number;
  resistance?: number;
  returnDuration?: number;
  className?: string;
}

function hexToRgb(hex: string) {
  const m = hex.match(/^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i);
  return m
    ? { r: parseInt(m[1], 16), g: parseInt(m[2], 16), b: parseInt(m[3], 16) }
    : { r: 0, g: 0, b: 0 };
}

// Defaults tuned for dark industrial: subtle slate dots, amber on approach
const DEFAULTS = {
  baseColor: '#2a2d36',      // near-bg slate
  activeColor: '#e8a64a',    // amber oklch(0.72 0.16 55)
};

const DotGrid: React.FC<DotGridProps> = ({
  dotSize = 3,              // small & restrained
  gap = 28,
  baseColor = DEFAULTS.baseColor,
  activeColor = DEFAULTS.activeColor,
  proximity = 140,
  speedTrigger = 100,
  shockRadius = 220,
  shockStrength = 4,
  maxSpeed = 5000,
  resistance = 750,
  returnDuration = 1.5,
  className = 'absolute inset-0',
}) => {
  const wrapperRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const dotsRef = useRef<Dot[]>([]);
  const pointerRef = useRef({ x: 0, y: 0, vx: 0, vy: 0, speed: 0, lastTime: 0, lastX: 0, lastY: 0 });

  const baseRgb = useMemo(() => hexToRgb(baseColor), [baseColor]);
  const activeRgb = useMemo(() => hexToRgb(activeColor), [activeColor]);

  const circlePath = useMemo(() => {
    if (typeof window === 'undefined' || !window.Path2D) return null;
    const p = new Path2D();
    p.arc(0, 0, dotSize / 2, 0, Math.PI * 2);
    return p;
  }, [dotSize]);

  const buildGrid = useCallback(() => {
    const wrap = wrapperRef.current;
    const canvas = canvasRef.current;
    if (!wrap || !canvas) return;
    const { width, height } = wrap.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    const ctx = canvas.getContext('2d');
    if (ctx) ctx.scale(dpr, dpr);

    const cols = Math.floor((width + gap) / (dotSize + gap));
    const rows = Math.floor((height + gap) / (dotSize + gap));
    const cell = dotSize + gap;
    const gridW = cell * cols - gap;
    const gridH = cell * rows - gap;
    const startX = (width - gridW) / 2 + dotSize / 2;
    const startY = (height - gridH) / 2 + dotSize / 2;

    const dots: Dot[] = [];
    for (let y = 0; y < rows; y++) {
      for (let x = 0; x < cols; x++) {
        dots.push({
          cx: startX + x * cell,
          cy: startY + y * cell,
          xOffset: 0, yOffset: 0, _inertiaApplied: false,
        });
      }
    }
    dotsRef.current = dots;
  }, [dotSize, gap]);

  useEffect(() => {
    if (!circlePath) return;
    let raf: number;
    const proxSq = proximity * proximity;
    const draw = () => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const { x: px, y: py } = pointerRef.current;
      for (const dot of dotsRef.current) {
        const ox = dot.cx + dot.xOffset;
        const oy = dot.cy + dot.yOffset;
        const dx = dot.cx - px;
        const dy = dot.cy - py;
        const dsq = dx * dx + dy * dy;
        let style = baseColor;
        if (dsq <= proxSq) {
          const t = 1 - Math.sqrt(dsq) / proximity;
          const r = Math.round(baseRgb.r + (activeRgb.r - baseRgb.r) * t);
          const g = Math.round(baseRgb.g + (activeRgb.g - baseRgb.g) * t);
          const b = Math.round(baseRgb.b + (activeRgb.b - baseRgb.b) * t);
          style = `rgb(${r},${g},${b})`;
        }
        ctx.save();
        ctx.translate(ox, oy);
        ctx.fillStyle = style;
        ctx.fill(circlePath);
        ctx.restore();
      }
      raf = requestAnimationFrame(draw);
    };
    draw();
    return () => cancelAnimationFrame(raf);
  }, [proximity, baseColor, activeRgb, baseRgb, circlePath]);

  useEffect(() => {
    buildGrid();
    const ro = new ResizeObserver(buildGrid);
    if (wrapperRef.current) ro.observe(wrapperRef.current);
    return () => ro.disconnect();
  }, [buildGrid]);

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      const now = performance.now();
      const pr = pointerRef.current;
      const dt = pr.lastTime ? now - pr.lastTime : 16;
      const dx = e.clientX - pr.lastX;
      const dy = e.clientY - pr.lastY;
      let vx = (dx / dt) * 1000;
      let vy = (dy / dt) * 1000;
      let speed = Math.hypot(vx, vy);
      if (speed > maxSpeed) { const s = maxSpeed / speed; vx *= s; vy *= s; speed = maxSpeed; }
      pr.lastTime = now; pr.lastX = e.clientX; pr.lastY = e.clientY;
      pr.vx = vx; pr.vy = vy; pr.speed = speed;
      const rect = canvasRef.current!.getBoundingClientRect();
      pr.x = e.clientX - rect.left;
      pr.y = e.clientY - rect.top;
      for (const dot of dotsRef.current) {
        const dist = Math.hypot(dot.cx - pr.x, dot.cy - pr.y);
        if (speed > speedTrigger && dist < proximity && !dot._inertiaApplied) {
          dot._inertiaApplied = true;
          gsap.killTweensOf(dot);
          const pushX = dot.cx - pr.x + vx * 0.005;
          const pushY = dot.cy - pr.y + vy * 0.005;
          gsap.to(dot, {
            inertia: { xOffset: pushX, yOffset: pushY, resistance },
            onComplete: () => {
              gsap.to(dot, { xOffset: 0, yOffset: 0, duration: returnDuration, ease: 'elastic.out(1,0.75)' });
              dot._inertiaApplied = false;
            },
          });
        }
      }
    };
    const throttled = throttle(onMove, 50);
    window.addEventListener('mousemove', throttled, { passive: true });
    return () => window.removeEventListener('mousemove', throttled);
  }, [maxSpeed, speedTrigger, proximity, resistance, returnDuration]);

  return (
    <div ref={wrapperRef} className={`${className} pointer-events-none`}>
      <canvas ref={canvasRef} className="absolute inset-0 w-full h-full" />
    </div>
  );
};

export default DotGrid;
```

**Usage:**
```tsx
<section className="relative bg-[oklch(0.14_0.01_260)] min-h-screen">
  <DotGrid className="absolute inset-0 opacity-70" dotSize={2.5} gap={28}
           baseColor="#262932" activeColor="#e8a64a" proximity={160} />
  <div className="relative z-10">...</div>
</section>
```

---

## 1D. `LetterGlitch.tsx` — OPTIONAL "DATA SYSTEMS" SECTION

Pure Canvas 2D (no WebGL, no deps). Great for a section about data pipelines, integrations, or systems. Tuned to amber/bronze only.

```tsx
'use client';
import { useRef, useEffect } from 'react';

interface LetterGlitchProps {
  glitchColors?: string[];
  glitchSpeed?: number;
  centerVignette?: boolean;
  outerVignette?: boolean;
  smooth?: boolean;
  characters?: string;
  className?: string;
}

// Restrained amber palette — NO rainbow
const AMBER_PALETTE = ['#3a2a1a', '#8a5a2a', '#e8a64a'];

export default function LetterGlitch({
  glitchColors = AMBER_PALETTE,
  glitchSpeed = 60,
  centerVignette = false,
  outerVignette = true,
  smooth = true,
  characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789<>/[]{}|_-+=',
  className = 'relative w-full h-full',
}: LetterGlitchProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rafRef = useRef<number | null>(null);
  const letters = useRef<{ char: string; color: string; targetColor: string; colorProgress: number }[]>([]);
  const grid = useRef({ columns: 0, rows: 0 });
  const ctxRef = useRef<CanvasRenderingContext2D | null>(null);
  const lastGlitch = useRef(Date.now());

  const charList = Array.from(characters);
  const fontSize = 14;
  const charW = 10;
  const charH = 18;

  const randChar = () => charList[Math.floor(Math.random() * charList.length)];
  const randColor = () => glitchColors[Math.floor(Math.random() * glitchColors.length)];

  const hexToRgb = (hex: string) => {
    const short = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
    hex = hex.replace(short, (_m, r, g, b) => r + r + g + g + b + b);
    const res = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return res ? { r: parseInt(res[1], 16), g: parseInt(res[2], 16), b: parseInt(res[3], 16) } : null;
  };
  const interp = (s: { r: number; g: number; b: number }, e: { r: number; g: number; b: number }, f: number) =>
    `rgb(${Math.round(s.r + (e.r - s.r) * f)},${Math.round(s.g + (e.g - s.g) * f)},${Math.round(s.b + (e.b - s.b) * f)})`;

  const initLetters = (cols: number, rows: number) => {
    grid.current = { columns: cols, rows };
    letters.current = Array.from({ length: cols * rows }, () => ({
      char: randChar(), color: randColor(), targetColor: randColor(), colorProgress: 1,
    }));
  };

  const resize = () => {
    const canvas = canvasRef.current;
    const parent = canvas?.parentElement;
    if (!canvas || !parent) return;
    const dpr = window.devicePixelRatio || 1;
    const rect = parent.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;
    ctxRef.current?.setTransform(dpr, 0, 0, dpr, 0, 0);
    initLetters(Math.ceil(rect.width / charW), Math.ceil(rect.height / charH));
    draw();
  };

  const draw = () => {
    const ctx = ctxRef.current;
    if (!ctx || !letters.current.length) return;
    const rect = canvasRef.current!.getBoundingClientRect();
    ctx.clearRect(0, 0, rect.width, rect.height);
    ctx.font = `${fontSize}px "JetBrains Mono", monospace`;
    ctx.textBaseline = 'top';
    letters.current.forEach((l, i) => {
      const x = (i % grid.current.columns) * charW;
      const y = Math.floor(i / grid.current.columns) * charH;
      ctx.fillStyle = l.color;
      ctx.fillText(l.char, x, y);
    });
  };

  const update = () => {
    const count = Math.max(1, Math.floor(letters.current.length * 0.04));
    for (let i = 0; i < count; i++) {
      const idx = Math.floor(Math.random() * letters.current.length);
      letters.current[idx].char = randChar();
      letters.current[idx].targetColor = randColor();
      if (!smooth) {
        letters.current[idx].color = letters.current[idx].targetColor;
        letters.current[idx].colorProgress = 1;
      } else {
        letters.current[idx].colorProgress = 0;
      }
    }
  };

  const smoothStep = () => {
    let needs = false;
    letters.current.forEach(l => {
      if (l.colorProgress < 1) {
        l.colorProgress = Math.min(1, l.colorProgress + 0.05);
        const s = hexToRgb(l.color); const e = hexToRgb(l.targetColor);
        if (s && e) { l.color = interp(s, e, l.colorProgress); needs = true; }
      }
    });
    if (needs) draw();
  };

  const animate = () => {
    const now = Date.now();
    if (now - lastGlitch.current >= glitchSpeed) { update(); draw(); lastGlitch.current = now; }
    if (smooth) smoothStep();
    rafRef.current = requestAnimationFrame(animate);
  };

  useEffect(() => {
    const c = canvasRef.current;
    if (!c) return;
    ctxRef.current = c.getContext('2d');
    resize();
    animate();
    let t: ReturnType<typeof setTimeout>;
    const onResize = () => {
      clearTimeout(t);
      t = setTimeout(() => { cancelAnimationFrame(rafRef.current!); resize(); animate(); }, 100);
    };
    window.addEventListener('resize', onResize);
    return () => { cancelAnimationFrame(rafRef.current!); window.removeEventListener('resize', onResize); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [glitchSpeed, smooth]);

  return (
    <div className={`${className} bg-[oklch(0.14_0.01_260)] overflow-hidden`}>
      <canvas ref={canvasRef} className="block w-full h-full" aria-hidden />
      {outerVignette && (
        <div className="absolute inset-0 pointer-events-none
          bg-[radial-gradient(circle,_transparent_55%,_oklch(0.14_0.01_260)_100%)]" />
      )}
      {centerVignette && (
        <div className="absolute inset-0 pointer-events-none
          bg-[radial-gradient(circle,_rgba(0,0,0,0.7)_0%,_transparent_60%)]" />
      )}
    </div>
  );
}
```

---

## 2. Widget Hover Animations — 5 Techniques (Motion + Tailwind v4)

Each is ≤30 lines, production-ready. All assume `motion/react` installed.

### 2A. Magnetic tilt on hover (3D transform, Motion springs)
```tsx
'use client';
import { motion, useMotionValue, useSpring, useTransform } from 'motion/react';

export function MagneticCard({ children }: { children: React.ReactNode }) {
  const mx = useMotionValue(0);
  const my = useMotionValue(0);
  const rx = useSpring(useTransform(my, [-0.5, 0.5], [8, -8]), { stiffness: 250, damping: 22 });
  const ry = useSpring(useTransform(mx, [-0.5, 0.5], [-8, 8]), { stiffness: 250, damping: 22 });

  return (
    <motion.div
      onMouseMove={(e) => {
        const r = e.currentTarget.getBoundingClientRect();
        mx.set((e.clientX - r.left) / r.width - 0.5);
        my.set((e.clientY - r.top) / r.height - 0.5);
      }}
      onMouseLeave={() => { mx.set(0); my.set(0); }}
      style={{ rotateX: rx, rotateY: ry, transformPerspective: 900 }}
      className="relative rounded-2xl bg-neutral-900/60 border border-white/10 p-6 will-change-transform"
    >
      {children}
    </motion.div>
  );
}
```

### 2B. Spotlight following cursor (pure CSS vars, no JS math)
```tsx
'use client';
import { useRef } from 'react';

export function SpotlightCard({ children }: { children: React.ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  return (
    <div
      ref={ref}
      onMouseMove={(e) => {
        const r = ref.current!.getBoundingClientRect();
        ref.current!.style.setProperty('--mx', `${e.clientX - r.left}px`);
        ref.current!.style.setProperty('--my', `${e.clientY - r.top}px`);
      }}
      className="group relative overflow-hidden rounded-2xl border border-white/10 bg-neutral-900/60 p-6"
      style={{ '--mx': '50%', '--my': '50%' } as React.CSSProperties}
    >
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-300 group-hover:opacity-100"
        style={{
          background: 'radial-gradient(360px circle at var(--mx) var(--my), oklch(0.72 0.16 55 / 0.18), transparent 60%)',
        }}
      />
      <div className="relative">{children}</div>
    </div>
  );
}
```

### 2C. Border beam — `offset-path` (Magic UI technique, ≤30 LOC)
```tsx
'use client';

export function BorderBeamCard({ children }: { children: React.ReactNode }) {
  return (
    <div className="relative rounded-2xl border border-white/10 bg-neutral-900/60 p-6 overflow-hidden">
      <style>{`
        @keyframes borderBeamSlide { to { offset-distance: 100%; } }
      `}</style>
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 rounded-[inherit]"
        style={{
          background: 'conic-gradient(transparent 340deg, oklch(0.72 0.16 55) 360deg, transparent 380deg)',
          WebkitMask:
            'linear-gradient(#000,#000) content-box, linear-gradient(#000,#000)',
          WebkitMaskComposite: 'xor',
          maskComposite: 'exclude',
          padding: 1,
          animation: 'borderBeamSlide 6s linear infinite',
          offsetPath: 'rect(0 100% 100% 0 round 16px)',
        }}
      />
      <div className="relative">{children}</div>
    </div>
  );
}
```

### 2D. Gradient background shift toward cursor (subtle, cheap)
```tsx
'use client';
import { useRef } from 'react';

export function GradientShiftCard({ children }: { children: React.ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  return (
    <div
      ref={ref}
      onMouseMove={(e) => {
        const r = ref.current!.getBoundingClientRect();
        const px = ((e.clientX - r.left) / r.width) * 100;
        const py = ((e.clientY - r.top) / r.height) * 100;
        ref.current!.style.background =
          `radial-gradient(1200px at ${px}% ${py}%, oklch(0.72 0.16 55 / 0.08), transparent 50%), oklch(0.17 0.012 260)`;
      }}
      onMouseLeave={() => { ref.current!.style.background = 'oklch(0.17 0.012 260)'; }}
      className="rounded-2xl border border-white/10 p-6 transition-colors duration-500"
    >
      {children}
    </div>
  );
}
```

### 2E. Light reveal (radial gradient gated by group-hover + pointer tracking)
```tsx
'use client';
import { useRef } from 'react';

export function LightRevealCard({ children }: { children: React.ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  return (
    <div
      ref={ref}
      onMouseMove={(e) => {
        const r = ref.current!.getBoundingClientRect();
        ref.current!.style.setProperty('--x', `${e.clientX - r.left}px`);
        ref.current!.style.setProperty('--y', `${e.clientY - r.top}px`);
      }}
      className="group relative rounded-2xl border border-white/5 p-6 bg-[oklch(0.14_0.01_260)] overflow-hidden"
      style={{ '--x': '50%', '--y': '50%' } as React.CSSProperties}
    >
      <div aria-hidden className="absolute -inset-px rounded-[inherit] opacity-0 group-hover:opacity-100 transition-opacity duration-500"
        style={{
          background: 'radial-gradient(220px circle at var(--x) var(--y), oklch(0.72 0.16 55 / 0.35), transparent 70%)',
          WebkitMask: 'linear-gradient(#000,#000) content-box, linear-gradient(#000,#000)',
          WebkitMaskComposite: 'xor',
          maskComposite: 'exclude',
          padding: '1px',
        }} />
      <div className="relative">{children}</div>
    </div>
  );
}
```

**Recommendation:** for this site, combine **2A (magnetic tilt) + 2E (light reveal border)** on feature cards. Use **2B (spotlight)** on secondary cards. Avoid stacking all five on one card.

---

## 3. Border Animations Around Widgets — 6 Techniques

### 3A. Conic-gradient rotating (simplest, use for attention gates like "Pro" badges)
```tsx
// when: small CTA, plan cards, always-on emphasis
<div className="relative rounded-2xl p-[1px] bg-[conic-gradient(from_var(--a),transparent_270deg,oklch(0.72_0.16_55)_300deg,transparent_330deg)]
                [animation:spin_6s_linear_infinite] [--a:0deg]">
  <div className="rounded-[inherit] bg-[oklch(0.14_0.01_260)] p-6">content</div>
</div>
<style>{`@property --a { syntax: '<angle>'; inherits: false; initial-value: 0deg; } @keyframes spin { to { --a: 360deg; } }`}</style>
```

### 3B. SVG stroke-dashoffset (precise path control — use when border radius varies per corner)
```tsx
// when: editorial cards with mixed radii, reveal-on-scroll perimeter
<div className="relative">
  <svg className="absolute inset-0 w-full h-full" aria-hidden>
    <rect x="1" y="1" width="calc(100% - 2px)" height="calc(100% - 2px)" rx="16"
      fill="none" stroke="oklch(0.72 0.16 55)" strokeWidth="1" strokeDasharray="40 400"
      className="[stroke-dashoffset:0] [animation:march_6s_linear_infinite]" />
  </svg>
  <style>{`@keyframes march { to { stroke-dashoffset: -440; } }`}</style>
</div>
```

### 3C. `offset-path: rect(...)` + `@property` — Magic UI BorderBeam approach
```tsx
// when: you want a finite-width glowing "beam" traveling around the edge (most modern look 2025-26)
<div className="relative rounded-2xl border border-white/10 overflow-hidden">
  <div aria-hidden className="absolute w-20 h-20 rounded-full
    [offset-path:rect(0_100%_100%_0_round_16px)]
    [offset-distance:0%] [animation:beam_7s_linear_infinite]
    bg-[radial-gradient(closest-side,oklch(0.72_0.16_55/0.9),transparent)]" />
  <style>{`@keyframes beam { to { offset-distance: 100%; } }`}</style>
  <div className="relative p-6">content</div>
</div>
```

### 3D. Double-border with gradient masking (for "glass with edge-light" look)
```tsx
// when: you want a static, premium backlit edge (no animation, for always-on elegance)
<div className="relative rounded-2xl p-[1px]
  bg-[linear-gradient(135deg,oklch(0.72_0.16_55/0.45),transparent_40%,transparent_60%,oklch(0.72_0.16_55/0.25))]">
  <div className="rounded-[inherit] bg-[oklch(0.14_0.01_260)] p-6">content</div>
</div>
```

### 3E. Shine-on-hover via `mask-image` (sweeping highlight across edge)
```tsx
// when: reward hover with a single traveling sparkle on the border
<div className="group relative rounded-2xl border border-white/10 overflow-hidden">
  <div aria-hidden className="absolute -inset-px opacity-0 group-hover:opacity-100
    [background:linear-gradient(115deg,transparent_40%,oklch(0.72_0.16_55/0.9)_50%,transparent_60%)]
    [mask:linear-gradient(#000,#000)_content-box,linear-gradient(#000,#000)] [mask-composite:exclude]
    p-px [animation:shine_1.2s_ease-out]" />
  <style>{`@keyframes shine { 0% { transform: translateX(-60%); } 100% { transform: translateX(60%); } }`}</style>
  <div className="relative p-6">content</div>
</div>
```

### 3F. `@property --angle` conic animation (best tradeoff for "active" cards)
```tsx
// when: you want one card to glow permanently to cue "currently selected"
<div className="relative rounded-2xl p-[1px]
  bg-[conic-gradient(from_var(--angle),oklch(0.72_0.16_55),transparent_20%,transparent_80%,oklch(0.72_0.16_55))]
  [animation:rot_4s_linear_infinite] [--angle:0deg]">
  <div className="rounded-[inherit] bg-[oklch(0.14_0.01_260)] p-6">content</div>
</div>
<style>{`@property --angle { syntax: '<angle>'; inherits: false; initial-value: 0deg; } @keyframes rot { to { --angle: 360deg; } }`}</style>
```

**Technique picker:**
- Cards at rest, no hover: **3D** (static gradient edge)
- Default feature cards with hover: **3E** (shine)
- "Pro"/selected/featured card: **3F** (conic)
- Pricing highlighted tier: **3C** (offset-path beam)
- Data-bound cards (progress feel): **3B** (SVG stroke)

---

## 4. Scroll-Reveal Patterns — Late-2025 / Early-2026 Vocabulary

Studied current landing pages (reviewed April 2026): Linear, Vercel, Arc, Raycast, Framer, Cursor, Ramp, Clerk. What's shipping:

- **Linear**: BlurFade on every block; sidebar-style outlines reveal with stroke-dasharray; everything under 300ms with aggressive stagger (40ms delta).
- **Vercel**: Heavy use of CSS `animation-timeline: view()` (no JS). Text slides from below with blur.
- **Arc/Browser Company**: Scroll-snap with full-bleed transitions, very few reveal animations — they let the content BE the animation.
- **Raycast**: Stagger + slight upward drift + opacity; no blur. Spring-based.
- **Framer**: Lots of parallax and scrub-scrolled numeric counters.
- **Cursor**: Text-reveal word-by-word (1-char stagger). Borrowed from Vercel.
- **Ramp**: Pin-and-scrub sections for product screenshots.
- **Clerk**: BlurFade primitive from Magic UI, used verbatim.

**Library survey (April 2026):**
- `Magic UI BlurFade` — most copied. Opacity + Y + blur on enter.
- `Motion Primitives InView` — simpler, no blur.
- `Aceternity TextReveal` — word-by-word scroll-driven reveal.
- Native CSS `animation-timeline: view()` — now ~90% browser support, replaces many JS reveals for static content.

### 4A. BlurFade (Magic UI flavor, Motion v11 API)
```tsx
'use client';
import { motion, useInView } from 'motion/react';
import { useRef } from 'react';

export function BlurFade({ children, delay = 0 }: { children: React.ReactNode; delay?: number }) {
  const ref = useRef(null);
  const inView = useInView(ref, { once: true, margin: '-80px' });
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 12, filter: 'blur(8px)' }}
      animate={inView ? { opacity: 1, y: 0, filter: 'blur(0px)' } : {}}
      transition={{ duration: 0.55, delay, ease: [0.16, 1, 0.3, 1] }}
    >{children}</motion.div>
  );
}
```

### 4B. Stagger children with spring (for feature grids)
```tsx
'use client';
import { motion } from 'motion/react';

export function StaggerGrid({ children }: { children: React.ReactNode }) {
  return (
    <motion.div
      initial="hidden" whileInView="show" viewport={{ once: true, margin: '-80px' }}
      variants={{ show: { transition: { staggerChildren: 0.06, delayChildren: 0.05 } } }}
      className="grid grid-cols-1 md:grid-cols-3 gap-6"
    >
      {children}
    </motion.div>
  );
}
export const staggerItem = {
  hidden: { opacity: 0, y: 16 },
  show: { opacity: 1, y: 0, transition: { type: 'spring', stiffness: 260, damping: 26 } },
};
// Wrap each card: <motion.div variants={staggerItem}>...</motion.div>
```

### 4C. Native CSS scroll-driven reveal (no JS, best perf)
```tsx
// globals.css
@keyframes reveal { from { opacity: 0; transform: translateY(20px) } to { opacity: 1; transform: none } }
.reveal-on-view { animation: reveal linear both; animation-timeline: view(); animation-range: entry 0% cover 40%; }

// usage (no JS!)
<div className="reveal-on-view">...</div>
```

### 4D. Text-scramble on reveal (headline drama, one-shot)
```tsx
'use client';
import { useEffect, useRef, useState } from 'react';
import { useInView } from 'motion/react';

export function ScrambleText({ text, className = '' }: { text: string; className?: string }) {
  const ref = useRef<HTMLSpanElement>(null);
  const inView = useInView(ref, { once: true });
  const [out, setOut] = useState(text.replace(/./g, ' '));
  useEffect(() => {
    if (!inView) return;
    const chars = '█▓▒░<>/{}[]';
    let i = 0;
    const id = setInterval(() => {
      setOut(text.split('').map((c, j) => j < i ? c : chars[Math.floor(Math.random() * chars.length)]).join(''));
      if (i++ >= text.length) clearInterval(id);
    }, 35);
    return () => clearInterval(id);
  }, [inView, text]);
  return <span ref={ref} className={className}>{out}</span>;
}
```

### 4E. Word-by-word reveal (Cursor/Vercel style headlines)
```tsx
'use client';
import { motion } from 'motion/react';

export function WordReveal({ text, className = '' }: { text: string; className?: string }) {
  return (
    <motion.span
      className={className}
      initial="hidden" whileInView="show" viewport={{ once: true, margin: '-20%' }}
      variants={{ show: { transition: { staggerChildren: 0.05 } } }}
    >
      {text.split(' ').map((w, i) => (
        <motion.span key={i} className="inline-block mr-[0.25em]"
          variants={{ hidden: { opacity: 0, y: '40%', filter: 'blur(6px)' }, show: { opacity: 1, y: 0, filter: 'blur(0)' } }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >{w}</motion.span>
      ))}
    </motion.span>
  );
}
```

### 4F. Scroll-scrubbed numeric counter (Ramp/Stripe-style stat section)
```tsx
'use client';
import { motion, useInView, useMotionValue, useSpring, useTransform } from 'motion/react';
import { useEffect, useRef } from 'react';

export function Counter({ to, suffix = '', className = '' }: { to: number; suffix?: string; className?: string }) {
  const ref = useRef<HTMLSpanElement>(null);
  const inView = useInView(ref, { once: true, margin: '-40%' });
  const mv = useMotionValue(0);
  const smooth = useSpring(mv, { stiffness: 70, damping: 20 });
  const display = useTransform(smooth, (v) => Math.round(v).toLocaleString() + suffix);
  useEffect(() => { if (inView) mv.set(to); }, [inView, to, mv]);
  return <motion.span ref={ref} className={className}>{display}</motion.span>;
}
```

**Use-case map for the landing page:**
- Hero headline → **4E (word reveal)**
- Sub-headline & body paragraphs → **4A (BlurFade)**
- Feature grid cards → **4B (stagger)**
- Static decorative labels/chips → **4C (CSS `view()`)**
- "ERP integrations", system names on a dark block → **4D (scramble)**
- Stats / trust numbers → **4F (counter)**

---

## 5. Final Recommended Stack For This Page

```
BACKGROUNDS      Threads (hero)  ·  Silk (mid-section)  ·  DotGrid (features)  ·  LetterGlitch (optional data)
CARD HOVER       Magnetic tilt (2A) + Light-reveal border (2E) on features; Spotlight (2B) on secondary
CARD BORDER      Shine-on-hover (3E) default; Conic @property (3F) on 'active' or highlighted tier
SCROLL REVEAL    Word-reveal (4E) for hero H1; BlurFade (4A) everywhere else; Stagger (4B) in grids; Counter (4F) in stats
```

All pieces are MIT-compatible, all adapt cleanly to `oklch(0.14 0.01 260)` + `oklch(0.72 0.16 55)`, and none introduce rainbow gradients.
