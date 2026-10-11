// The yard out of doors, behind whoever stands in it: the sky of the hour, clouds
// going by, two mountain ranges paling with distance, a line of firs; and as the
// window widens, a tree that keeps the seasons, a well, a mill wheel. Painted in
// true colours rather than letters: what is far fades into the sky, a ridge or a
// rim is smoothed where it crosses a pixel, a cloud is soft. The door frame (oak
// posts, the eave, the sill) parts it from the room. What falls (rain, snow,
// leaves, petals) is laid over the clear cells in finer glyphs.

import { dot, quad, rand } from '../pixels.ts'
import type { Grid } from '../pixels.ts'
import { season } from '../props.ts'
import type { World } from '../world.ts'

type RGB = readonly [number, number, number]

const hex = (n: number): RGB => [(n >> 16) & 255, (n >> 8) & 255, n & 255]
const mix = (a: RGB, b: RGB, k: number): RGB => [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
const mul = (a: RGB, m: RGB): RGB => [a[0] * m[0], a[1] * m[1], a[2] * m[2]]
const clamp01 = (v: number) => Math.max(0, Math.min(1, v))
const smooth = (a: number, b: number, v: number) => { const t = clamp01((v - a) / (b - a)); return t * t * (3 - 2 * t) }
/** A noise that rolls rather than jumps, along one axis. */
const roll = (x: number, seed: number) => {
  const i = Math.floor(x), f = x - i, u = f * f * (3 - 2 * f)
  return rand(i * 7 + seed) * (1 - u) + rand((i + 1) * 7 + seed) * u
}
/** The same over a plane: for the clouds. */
const roll2 = (x: number, y: number, seed: number) => {
  const j = Math.floor(y), f = y - j, u = f * f * (3 - 2 * f)
  return roll(x, seed + j * 131) * (1 - u) + roll(x, seed + (j + 1) * 131) * u
}

const HEIGHT = 32
const GROUND = 28 // the yard's floor, from this row down
const HORIZON = 25

/** The hour and the weather out of doors: the sky's two ends, the light things take, the forge's glow at the door. */
type Sky = { top: RGB; low: RGB; amb: RGB; spill: number; night: boolean; dusk: boolean; storm: boolean; flash: boolean }

function skyOf(w: World): Sky {
  const night = w.hour < 6 || w.hour >= 21
  const dusk = !night && (w.hour < 8 || w.hour >= 18)
  const storm = w.weather === 'storm'
  // the lightning, on the window's beat
  const flash = storm && w.clock % 23 < 2
  const base = storm
    ? night
      ? { top: hex(0x0d0f16), low: hex(0x262a36), amb: [0.26, 0.29, 0.38] as RGB, spill: 0.5 }
      : { top: hex(0x3a3f4c), low: hex(0x7a808c), amb: [0.58, 0.61, 0.68] as RGB, spill: 0.25 }
    : night ? { top: hex(0x070b22), low: hex(0x1d2c58), amb: [0.3, 0.36, 0.6] as RGB, spill: 0.55 }
    : dusk ? { top: hex(0x34407e), low: hex(0xf3a35e), amb: [0.92, 0.72, 0.62] as RGB, spill: 0.25 }
    : { top: hex(0x4f9fe6), low: hex(0xc4e6f6), amb: [1, 1, 0.97] as RGB, spill: 0 }
  if (flash) return { ...base, top: mix(base.top, hex(0xc8ccdc), 0.5), low: mix(base.low, hex(0xe0e4f0), 0.5), amb: mix(base.amb, [1, 1, 1.1], 0.5), night, dusk, storm, flash }
  return { ...base, night, dusk, storm, flash }
}

/** Where things stand in a yard from the door (`from`) to the scene's edge (`to`): the more room, the more things. */
export function layout(from: number, to: number) {
  const room = to - from
  const wheel = room >= 62 ? to - 11 : undefined
  // a narrow yard has only the edge of a crown peeking in, high on the right
  const tree = room < 30 ? { x: to + 1, r: 7 } : { x: wheel !== undefined ? to - 30 : to - 9, r: 8.5 }
  const well = room >= 44 ? tree.x - 21 : undefined
  return { wheel, tree, well }
}

/**
 * The yard for a frame: `back` the colour behind the actors at (x, y), `light` what
 * the out-of-doors does to an actor's colour there. `working`: the wheel turns faster.
 */
export function outdoors(w: World, working: boolean, from: number, to: number) {
  const sky = skyOf(w)
  const se = season(w), winter = se === 'winter'
  const t = w.clock
  const { wheel, tree, well } = layout(from, to)
  const lit = (c: RGB, far = 0): RGB => mix(mul(c, sky.amb), sky.low, far)

  // the forge's glow through the door, evening and night: a warm pool on the floor
  const spill = (x: number, y: number, c: RGB): RGB => {
    if (sky.spill <= 0 || y < 22) return c
    const d = Math.hypot((x - from) / 1.6, (y - 31) * 1.4)
    const f = Math.max(0, 1 - d / 14) ** 2 * sky.spill
    return [c[0] + 230 * f, c[1] + 135 * f, c[2] + 63 * f]
  }

  const skyAt = (x: number, y: number): RGB => {
    let c = mix(sky.top, sky.low, clamp01(y / HORIZON))
    if (sky.night && !sky.storm && y < 18 && rand(x * 97 + y * 13) > 0.975) c = rand(x + y) > 0.6 ? [250, 245, 220] : [170, 180, 220]
    // the clouds, drifting right to left; thick and quick in a storm
    const speed = sky.storm ? 0.04 : 0.012
    const d = (roll2(x / 10 + t * speed, y / 3, 11) * 0.65 + roll2(x / 5 + t * speed * 1.4, y / 2, 23) * 0.35) * (1 - Math.abs(y - 6) / 11)
    const a = smooth(sky.storm ? 0.22 : 0.5, sky.storm ? 0.4 : 0.66, d)
    if (a <= 0) return c
    const under = clamp01((y - 2) / 9) // a cloud is darker underneath
    const cloud = sky.storm ? mix(hex(0x4e5260), hex(0x2c303c), under)
      : sky.night ? mix(hex(0x2c3554), hex(0x1a2240), under)
      : sky.dusk ? mix(hex(0xf2b8a6), hex(0x8a5a7a), under)
      : mix(hex(0xf6f8fb), hex(0xbcc6d4), under)
    return mix(c, sky.flash ? mix(cloud, [230, 232, 245], 0.4) : cloud, a * (sky.night ? 0.8 : 1))
  }

  /** Two ranges and the firs: each ridge smoothed where it crosses a pixel. */
  const hills = (x: number, y: number, c: RGB): RGB => {
    const far = 8 + roll(x / 9, 1) * 6 + roll(x / 4, 2) * 2
    if (y + 1 > far) {
      const snow = y < far + (winter ? 4 : 1.5) || y < 11 // the high peaks keep their snow
      c = mix(c, lit(snow ? hex(0xe8eef4) : hex(0x5d6e86), 0.6), clamp01(y + 1 - far))
    }
    const near = 15 + roll(x / 6, 3) * 5
    if (y + 1 > near) c = mix(c, lit(winter && y < near + 1 ? hex(0xc8d4e0) : hex(0x3e5560), 0.38), clamp01(y + 1 - near))
    const fir = Math.round(x / 5) * 5, top = 27 - (5 + Math.floor(rand(fir) * 4)) + Math.abs(x - fir) * 2
    if (y >= top || y >= 26) c = lit(winter && (x + y) % 3 === 0 ? hex(0xdfe8f0) : hex(0x23402c), 0.05)
    return c
  }

  const ground = (x: number, y: number): RGB => {
    const path = x < from + 14 // cobbles from the door, then grass
    if (winter && !(path && x < from + 6)) return lit((x * 3 + y * 5) % 11 === 0 ? hex(0xc9d6e6) : hex(0xeaf0f7))
    if (path) {
      const row = y % 2, stone = Math.floor((x + row * 2) / 4)
      const edge = (x + row * 2) % 4 === 0 || y === GROUND
      return lit(edge ? hex(0x4a4048) : rand(stone * 13 + y) > 0.5 ? hex(0x7a7078) : hex(0x6c6470))
    }
    if (se === 'autumn' && rand(x * 17 + y * 3) > 0.9) return lit(hex(rand(x) > 0.5 ? 0xd06a22 : 0xe3a63c))
    const tuft = rand(x * 31 + y * 7) > 0.8
    return lit(hex(se === 'autumn' ? (tuft ? 0x8a6a2a : 0x6f6f30) : se === 'spring' ? (tuft ? 0x6fb04a : 0x4f8a36) : (tuft ? 0x3c6a2a : 0x4c7a34)))
  }

  /** The tree: its crown sways with the wind (harder in a storm), and keeps the season. */
  const crown = (x: number, y: number, c: RGB): RGB => {
    const { x: tx, r } = tree, ty = r + 0.5
    const gust = sky.storm ? 1.4 : 0.6
    const sway = Math.sin(t / (sky.storm ? 3 : 7) + y / 3) * gust * clamp01((ty + r - y) / (2 * r))
    const sx = x - sway
    const reach = r + roll(Math.atan2(y - ty, sx - tx) * 3 + 5, 5) * 2.5
    const leafy = Math.hypot((sx - tx) * 0.85, y - ty) < reach && y < ty + r * 0.75
    const trunk = Math.abs(x - tx) <= 1 && y >= ty + 3 && y < GROUND + 2
    // the boughs: three pairs fanning out from the trunk, seen whole once the leaves are down
    const up = ty + 3 - y, off = Math.abs(sx - tx)
    const bough = up >= 0 && y > ty - r * 0.8 && [0.45, 1, 1.8].some((k, i) => Math.abs(off - up * k) < 0.55 && up > i * 1.5 && off <= r - 1)
    if (leafy && !winter) {
      const k = rand(Math.round(sx) * 13 + y * 29)
      const leaf = se === 'autumn' ? (k > 0.66 ? 0xe07a22 : k > 0.33 ? 0xc04a1a : 0xe3a63c)
        : se === 'spring' ? (k > 0.85 ? 0xf6b8d0 : k > 0.4 ? 0x6fb04a : 0x4f8a36)
        : k > 0.6 ? 0x3c7a30 : 0x2f6228
      const shade = sx > tx + 2 || y > ty + 4 ? 0.8 : 1.05 // the light from the left, like the room's
      return lit(mul(hex(leaf), [shade, shade, shade]))
    }
    if (trunk || bough) return lit(x === tx - 1 ? hex(0x6a4228) : hex(0x4a2c16))
    if (winter && leafy && y % 3 === 0 && rand(x * 5 + y) > 0.55) return lit(hex(0xeef2f8)) // snow on the bare twigs
    return c
  }

  const wellAt = (x: number, y: number, c: RGB): RGB => {
    if (well === undefined || x < well || x >= well + 9 || y < 17 || y >= GROUND + 2) return c
    const dx = x - well
    if (y >= 24) return lit((dx + (y % 2) * 2) % 3 === 0 ? hex(0x4f4a55) : hex(0x8a8494))
    if (y === 17 || (y === 18 && dx > 0 && dx < 8)) return lit(winter ? hex(0xeef2f8) : hex(0x8a3a24))
    if (dx === 1 || dx === 7) return lit(hex(0x5a3a1e))
    if (y === 20 && dx > 1 && dx < 7) return lit(hex(0x7a4a24)) // the windlass
    if (dx === 4 && y > 20 && y < 23) return lit(hex(0x8a90a0)) // the bucket's rope
    if (dx >= 3 && dx <= 5 && y === 23) return lit(hex(0x6a4228)) // the bucket
    return c
  }

  // the wheel: water from the race on the right falls on its paddles; it turns
  // faster while he works (it drives the bellows), and stands frozen in winter
  const turn = winter ? 0.4 : t * (working ? 0.22 : 0.09)
  const millAt = (x: number, y: number, c: RGB): RGB => {
    if (wheel === undefined || x < wheel - 10) return c
    const cx = wheel, cy = 19, R = 7
    // the race on its trestle, from beyond the edge to over the wheel
    if (x === to - 2 && y > 11 && y < GROUND) c = lit(hex(0x5a3a1e))
    if (y === 11 && x >= cx - 1) return lit(hex(0x7a4a24))
    const water = (x: number, y: number) => winter ? lit(hex(0xbfe0f0)) : rand(x * 3 + Math.floor(y - t * 0.8) * 7) > 0.7 ? lit(hex(0xdff6ff)) : lit(hex(0x7fd4ff))
    if (y === 10 && x >= cx) return winter ? lit(hex(0xbfe0f0)) : rand(Math.floor(x + t * 0.6) * 5) > 0.75 ? lit(hex(0xdff6ff)) : lit(hex(0x7fd4ff))
    if (!winter && x === cx - 1 && y === 12) return water(x, y)
    if (winter && y === 12 && x >= cx && x % 2 === 0) return lit(hex(0xcfe8f6)) // icicles
    // the pond at its foot
    if (y >= GROUND - 1 && y <= GROUND && x >= cx - 9) return winter ? lit(hex(0xd8ecf6)) : rand(Math.floor(x + t * 0.3) * 11 + y) > 0.85 ? lit(hex(0xdff6ff)) : mul(lit(hex(0x5fb4e0)), [0.9, 0.95, 1])
    const d = Math.hypot(x - cx, y - cy)
    if (d > R + 2) return c
    // the water comes along the race from the right onto its top: the top goes left (anticlockwise)
    const ang = Math.atan2(y - cy, x - cx) + turn
    const near = (n: number) => { const s = (2 * Math.PI) / n, a = ((ang % s) + s) % s; return Math.min(a, s - a) * d }
    // drawn with soft edges, so it turns smoothly rather than by whole pixels
    const paddle = d > R && d < R + 2 ? clamp01(1.5 - near(10) / 0.7) : 0
    const rim = clamp01(1.6 - Math.abs(d - R) / 0.6)
    const inner = clamp01(1.2 - Math.abs(d - R + 2.5) / 0.5)
    const spoke = d < R ? clamp01(1.4 - near(6) / 0.6) : 0
    if (d < 1.5) return lit(hex(0x4a4e5c))
    c = mix(c, lit(hex(0x3a2214)), Math.max(spoke, inner * 0.8))
    c = mix(c, lit(hex(0x2a1a10)), paddle)
    c = mix(c, lit(hex(0x8a5a2e)), rim)
    return c
  }

  /**
   * The door: two oak posts, the eave over the yard, the stone sill. Under a storm the
   * rain runs down the slates and drips off the eave's end; in winter, icicles under it.
   */
  const EAVE_END = from + 12
  const door = (x: number, y: number): RGB | undefined => {
    const eave = Math.floor((x - from) / 3)
    if (x <= EAVE_END) {
      if (y >= eave && y <= eave + 2) return lit(y === eave ? (winter ? hex(0xeef2f8) : hex(0x8a8494)) : y === eave + 1 ? hex(0x5d5866) : hex(0x3a2418))
      if (winter && y === eave + 3 && x > from && x % 2 === 0) return lit(hex(0xcfe8f6))
      // the runoff on the slates, going down the slope
      if (sky.storm && y === eave - 1 && x > from && (x - Math.floor(t / 2)) % 3 === 0) return lit(hex(0x7fd4ff))
    }
    if (sky.storm && x === EAVE_END) {
      // and off the end, drop after drop, down to the ground
      const lip = Math.floor((EAVE_END - from) / 3) + 3
      for (let k = 0; k < 2; k++) if (y === lip + ((Math.floor(t * 1.5) + k * 9) % (GROUND - lip))) return lit(hex(0x9fdcff))
    }
    if (x <= from + 1 && y >= 2) return lit(x === from ? hex(0x4a2c16) : hex(0x6a4228))
    if (y >= 30 && x <= from + 2) return lit(y === 30 ? hex(0x8a8494) : hex(0x5d5866))
    return undefined
  }

  // painted once a frame, from the door to the edge
  const width = Math.max(0, to - from)
  const painted: RGB[] = []
  for (let y = 0; y < HEIGHT; y++) {
    for (let x = from; x < to; x++) {
      let c = door(x, y)
      if (c === undefined) {
        c = y >= GROUND ? ground(x, y) : hills(x, y, skyAt(x, y))
        c = millAt(x, y, wellAt(x, y, crown(x, y, c)))
      }
      painted.push(spill(x, y, c))
    }
  }
  return {
    back: (x: number, y: number): RGB | undefined => (x >= from && x < to && y >= 0 && y < HEIGHT ? painted[y * width + x - from] : undefined),
    light: (c: RGB, x: number, y: number): RGB => spill(x, y, mul(c, sky.amb)),
  }
}

/** What falls over the yard, in glyphs finer than a pixel and on its clear cells only: rain in quarters, snow, leaves and petals in braille. */
export function drawYardWeather(g: Grid, w: World, from: number, to: number) {
  const t = w.clock, room = to - from - 2, se = season(w)
  if (room <= 0) return
  if (w.weather === 'storm') {
    for (let i = 0; i < Math.round(room * 0.45); i++) {
      // a streak a half pixel wide, two long, slanting with the wind
      const qx = 2 * (from + 2 + rand(i * 13) * room) - ((t * 1.5 + i * 5) % 30) * 0.3
      const qy = (t * 1.5 + i * 5) % 30
      quad(g, qx, qy, 'c', '.')
      quad(g, qx, qy - 1, 'c', '.')
    }
  }
  if (se === 'winter') {
    for (let i = 0; i < Math.round(room * 0.2); i++) {
      const fx = from + 2 + rand(i * 13) * room + Math.sin((t + i * 7) / 3) * 0.7
      const fy = (t * (0.35 + 0.3 * rand(i * 3)) + i * 5) % 30
      dot(g, 2 * fx, 2 * fy, 'w', '.')
    }
  }
  if (se === 'autumn' || se === 'spring') {
    // from the tree's crown down to the grass, turning over, the wind taking them left
    const { tree } = layout(from, to)
    const colours = se === 'autumn' ? 'FyF' : 'MMi'
    for (let i = 0; i < 3; i++) {
      const fall = (t * (se === 'autumn' ? 0.3 : 0.25) + i * 9) % 24
      const fx = 2 * (tree.x - tree.r + rand(i * 13 + Math.floor((t * 0.3 + i * 9) / 24)) * tree.r * 2 - fall * 0.5 + Math.sin((t + i * 7) / 3))
      const fy = 2 * (6 + fall)
      const shape = (Math.floor(t / 3) + i) % 2 ? [[0, 0], [1, 1], [2, 1]] : [[0, 1], [1, 0], [2, 0]]
      for (const [dx = 0, dy = 0] of shape) dot(g, fx + dx, fy + dy, colours[i] ?? 'F', '.')
    }
  }
}
