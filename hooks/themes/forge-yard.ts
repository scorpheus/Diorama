// The yard out of doors, behind whoever stands in it: the sky of the hour, clouds
// going by, two mountain ranges paling with distance and falls down their flanks,
// a line of firs; and as the window widens, a tree that keeps the seasons, a well,
// a mill wheel. Painted in
// true colours rather than letters: what is far fades into the sky, a ridge or a
// rim is smoothed where it crosses a pixel, a cloud is soft. The door frame (oak
// posts, the eave, the sill) parts it from the room. What falls (rain, snow,
// leaves, petals) is laid over the clear cells in finer glyphs.

import { dot, quad, rand } from '../pixels.ts'
import type { Grid } from '../pixels.ts'
import { season } from '../props.ts'
import type { World } from '../world.ts'

// The engine runs this module in a vm context, where every read of a global (Math, Map…) costs
// several times a local's: bound once here, they are read as locals.
const { Math, Int32Array, Float64Array, Map, Array } = globalThis

type RGB = readonly [number, number, number]

/** A colour by its number: always the same one for the same number, so what is worked out from it can be kept. */
const hex = (n: number): RGB => {
  let c = hexes.get(n)
  if (c === undefined) {
    hexes.set(n, (c = [(n >> 16) & 255, (n >> 8) & 255, n & 255]))
    named.add(c)
  }
  return c
}
const hexes = new Map<number, RGB>()
const named = new WeakSet<RGB>()
const mix = (a: RGB, b: RGB, k: number): RGB => [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
const mul = (a: RGB, m: RGB): RGB => [a[0] * m[0], a[1] * m[1], a[2] * m[2]]
const clamp01 = (v: number) => Math.max(0, Math.min(1, v))
const smooth = (a: number, b: number, v: number) => { const t = clamp01((v - a) / (b - a)); return t * t * (3 - 2 * t) }
/** A noise that rolls rather than jumps, along one axis. */
const roll = (x: number, seed: number) => {
  const i = Math.floor(x), f = x - i, u = f * f * (3 - 2 * f)
  return rand(i * 7 + seed) * (1 - u) + rand((i + 1) * 7 + seed) * u
}

const HEIGHT = 32
const GROUND = 28 // the yard's floor, from this row down
const HORIZON = 25

/** A function of the column, worked out once a column: the land does not move. */
const once = (f: (x: number) => number) => {
  const known = new Map<number, number>()
  return (x: number) => {
    let v = known.get(x)
    if (v === undefined) known.set(x, (v = f(x)))
    return v
  }
}
/** The two ranges' ridges at column x: the far one, then the near one. */
const farRidge = once(x => 8 + roll(x / 9, 1) * 6 + roll(x / 4, 2) * 2)
const nearRidge = once(x => 15 + roll(x / 6, 3) * 5)
/** Where the firs at the foot of the near range begin, at column x: a fir every five columns. */
const firTop = once(x => { const fir = Math.round(x / 5) * 5; return 27 - (5 + Math.floor(rand(fir) * 4)) + Math.abs(x - fir) * 2 })

/** The boughs' slopes, from the trunk out. */
const BOUGHS = [0.45, 1, 1.8]
/** A leaf's colour, in the light or in the shade: one colour each, so the light on it is worked out once a frame. */
const leafOf = (n: number, shaded: boolean): RGB => {
  const key = n * 2 + (shaded ? 1 : 0)
  let c = leaves.get(key)
  if (c === undefined) {
    const shade = shaded ? 0.8 : 1.05
    leaves.set(key, (c = mul(hex(n), [shade, shade, shade])))
    named.add(c)
  }
  return c
}
const leaves = new Map<number, RGB>()

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

/** Where things stand in a yard from the door (`from`) to the scene's edge (`to`): the more room, the more things. Worked out once a width. */
export function layout(from: number, to: number): Layout {
  const key = `${from},${to}`
  let known = layouts.get(key)
  if (known === undefined) layouts.set(key, (known = placeAll(from, to)))
  return known
}
type Layout = { wheel: number | undefined; tree: { x: number; r: number }; well: number | undefined; falls: Array<{ x: number; near: boolean }> }
const layouts = new Map<string, Layout>()

function placeAll(from: number, to: number): Layout {
  const room = to - from
  const wheel = room >= 62 ? to - 11 : undefined
  // a narrow yard has only the edge of a crown peeking in, high on the right
  const tree = room < 30 ? { x: to + 1, r: 7 } : { x: wheel !== undefined ? to - 30 : to - 9, r: 8.5 }
  const well = room >= 44 ? tree.x - 21 : undefined
  // the falls. Far ones spring from the far range's cols (low points of its ridge), or else
  // from a flank, never a peak, where it stands well over the near one: one to a narrow yard,
  // more as it widens. The near one comes down the near range where it shows most face over
  // the firs, between two of them. All clear of the trunk and the wheel.
  const hidden = (x: number) => Math.abs(x - tree.x) <= 3 || (wheel !== undefined && x >= wheel - 10)
  const around = (ridge: (x: number) => number, x: number, test: (r: number, k: number) => boolean) =>
    [1, 2, 3].every(k => test(ridge(x), ridge(x - k)) && test(ridge(x), ridge(x + k)))
  const pick = (score: (x: number) => number | undefined, n: number, apart: number) => {
    const spots: Array<{ x: number; v: number }> = []
    for (let x = from + 7; x < to - 3; x++) { const v = hidden(x) ? undefined : score(x); if (v !== undefined) spots.push({ x, v }) }
    spots.sort((a, b) => b.v - a.v)
    const chosen: number[] = []
    for (const { x } of spots) if (chosen.length < n && chosen.every(c => Math.abs(c - x) >= apart)) chosen.push(x)
    return chosen
  }
  const flank = (ridge: (x: number) => number, x: number) => !around(ridge, x, (r, k) => r <= k + 0.6)
  const far = pick(x => {
    const gap = nearRidge(x) - farRidge(x)
    return gap < 3.5 || !flank(farRidge, x) ? undefined : gap + (around(farRidge, x, (r, k) => r >= k) ? 4 : 0)
  }, Math.max(1, Math.round(room / 26)), 10)
  const near = pick(x => {
    const face = firTop(x) - nearRidge(x)
    return face < 3 || !flank(nearRidge, x) || far.some(f => Math.abs(f - x) < 4 && f !== x) ? undefined : face
  }, 1, 0)
  const falls = [...far.map(x => ({ x, near: false })), ...near.map(x => ({ x, near: true }))]
  return { wheel, tree, well, falls }
}

// --- the Route du Rhum: the bay of Saint-Malo for the mountains, and the fleet on it -------

const SEA = 17 // the horizon: the sea from this row down to the quay

/**
 * The boats side on, bow to the right: m the mast, M the mainsail, S its stripe, J the jib,
 * b the boom, H the hull, h its bow, f a float or a foil. The last row but `under` sits on
 * the water; the rows after it are in the water, half seen.
 */
type Hull = { rows: readonly string[]; under: number }
const ULTIM: Hull = {
  rows: [
    '........m......',
    '.......Mm......',
    '.......MmJ.....',
    '......MMmJ.....',
    '......MSmJJ....',
    '.....MMSmJJ....',
    '.....MMSmJJJ...',
    '....MMMMSJJJ...',
    '...MMMMMmJJJJ..',
    '..MMMMMMmJJJJ..',
    '..bbbbbbm......',
    '.HHHHHHHHHHHHHh',
    '..fff.....ffff.',
  ],
  under: 1,
}
const IMOCA: Hull = {
  rows: [
    '.....m....',
    '....Mm....',
    '....MmJ...',
    '...MSmJ...',
    '...MSmJJ..',
    '..MMSmJJ..',
    '..MMMmJJJ.',
    '.bbbbm....',
    'HHHHHHHHHh',
    '.....f....',
  ],
  under: 1,
}
const CLASS40: Hull = { rows: ['...m...', '..Mm...', '..MmJ..', '.MSmJ..', '.bbm...', 'HHHHHHh'], under: 0 }
const SPECK: Hull = { rows: ['..m..', '.MmJ.', 'MMmJ.', 'HHHHh'], under: 0 }

/** A boat of the fleet: its shape, the row its hull rides, its pace, its colours, how far off it sails. */
type Sail = { hull: Hull; lane: number; speed: number; seed: number; colours: { H: number; h: number; M: number; S: number }; far: number }

/** The fleet a bay of this width holds, from the horizon in: the more room, the bigger the boats. */
function fleetOf(room: number): Sail[] {
  const fleet: Sail[] = [
    { hull: SPECK, lane: 19, speed: 0.07, seed: 3, colours: { H: 0xe8e8ec, h: 0xc0392b, M: 0xf4f4f0, S: 0xf4f4f0 }, far: 0.55 },
    { hull: SPECK, lane: 20, speed: 0.09, seed: 17, colours: { H: 0x2a3a6a, h: 0x2a3a6a, M: 0xf4f4f0, S: 0xf4f4f0 }, far: 0.5 },
    { hull: CLASS40, lane: 22, speed: 0.15, seed: 29, colours: { H: 0xf2c14e, h: 0x2a2a30, M: 0xf6f6f2, S: 0x3f80dc }, far: 0.3 },
  ]
  fleet.push({ hull: IMOCA, lane: 24, speed: 0.24, seed: 41, colours: { H: 0x24242c, h: 0xd04a2a, M: 0xe8e8ea, S: 0xd04a2a }, far: 0.15 })
  if (room >= 34) fleet.push({ hull: ULTIM, lane: 26, speed: 0.42, seed: 59, colours: { H: 0x1d3f8a, h: 0xe8e8ec, M: 0xf8f8f6, S: 0x1d3f8a }, far: 0.02 })
  if (room >= 70) fleet.push({ hull: ULTIM, lane: 26, speed: 0.38, seed: 83, colours: { H: 0xc0392b, h: 0xf2f2f2, M: 0xf8f8f6, S: 0xc0392b }, far: 0.02 })
  return fleet
}

/**
 * Where a boat is at `t`: the column of its left edge, and which way it goes. Racing, it
 * crosses the bay left to right, out to the Atlantic, and comes round again after a while;
 * before the start, it tacks to and fro about its own spot, waiting for the gun.
 */
function sailing(s: Sail, t: number, rhum: 'before' | 'race', from: number, to: number): { x: number; dir: 1 | -1 } {
  const w = s.hull.rows[0]?.length ?? 0, room = to - from
  if (rhum === 'race') {
    const lap = room + w + 4 + Math.floor(rand(s.seed) * 16)
    return { x: from - w + Math.floor((((t * s.speed + rand(s.seed * 7) * lap) % lap) + lap) % lap), dir: 1 }
  }
  const reach = Math.max(2, Math.min(14, (room - w) / 3)), centre = from + 2 + rand(s.seed * 3) * Math.max(0, room - w - 4)
  const a = (t * s.speed * 0.5) / reach + rand(s.seed * 11) * 6.28
  return { x: Math.max(from + 2, Math.min(to - w, Math.round(centre + Math.sin(a) * reach))), dir: Math.cos(a) >= 0 ? 1 : -1 }
}

/** Two gulls over the bay, gliding on wide loops, a wingbeat now and then: their place at `t`, and their wings up or down. */
const gulls = (t: number, from: number, to: number) => [0, 1].map(i => {
  const room = to - from
  return {
    x: Math.round(from + room * (0.5 + 0.4 * Math.sin(t / (90 + i * 37) + i * 2.1))),
    y: Math.round(7 + i * 2 + 2 * Math.sin(t / (23 + i * 9) + i)),
    up: Math.floor(t / 2 + i) % 7 < 2,
  }
})

const byte = (v: number) => Math.max(0, Math.min(255, Math.round(v)))
const pack = (c: RGB) => (byte(c[0]) << 16) | (byte(c[1]) << 8) | byte(c[2])

/**
 * The yard as it stands still, for one sky, one season and one width: each pixel's
 * colour, packed; and the pixels that read the clock, to paint again every frame. A
 * pixel that did not read it cannot have changed: that much is never painted twice.
 */
type Still = { colours: Int32Array; moving: Int32Array }
const stills = new Map<string, Still>()
const STILLS = 12 // the last few skies and widths kept; the hour and the window change slowly

/**
 * The yard for a frame: `back` the colour behind the actors at (x, y), packed; `light`
 * what the out-of-doors does to an actor's colour there. `working`: the wheel turns faster.
 */
export function outdoors(w: World, working: boolean, from: number, to: number) {
  const sky = skyOf(w)
  const se = season(w), winter = se === 'winter'
  // the clock, read only through `now`: a pixel that never calls it stands still
  let moved = false
  const now = () => { moved = true; return w.clock }
  const { wheel, tree, well, falls } = layout(from, to)
  // a named colour under this frame's light, worked out once a frame
  const known = new Map<number, Map<RGB, RGB>>()
  const lit = (c: RGB, far = 0): RGB => {
    let byColour = known.get(far)
    if (byColour === undefined) known.set(far, (byColour = new Map()))
    let v = byColour.get(c)
    if (v === undefined) {
      v = mix(mul(c, sky.amb), sky.low, far)
      if (named.has(c)) byColour.set(c, v)
    }
    return v
  }

  /**
   * Water falling past (x, y), seen through by `alpha`: streaks of foam going down
   * `speed` pixels a tick; ice that stays put in winter. `far` pales it into the sky.
   */
  const falling = (c: RGB, x: number, y: number, speed: number, alpha: number, far = 0, white = false, drips = false): RGB => {
    if (winter) return mix(c, lit(rand(x * 3 + Math.floor(y / 2) * 7) > 0.5 ? hex(0xe4f2fa) : hex(0xbfe0f0), far), alpha)
    const k = rand(x * 3 + Math.floor((y - now() * speed) / 2) * 7)
    if (drips && k < 0.45) return c // threads that break into drops
    return mix(c, lit(k > (white ? 0.35 : 0.65) ? hex(0xf2fbff) : white ? hex(0xb4e2f6) : hex(0x4fb0f0), far), alpha)
  }

  /**
   * The falls: streams springing from the far range's cols, paled by the distance,
   * that go on behind the near ridge; and one down the near range, wider, from below
   * its ridge to the firs. None falls quite straight: it steps aside at its ledges,
   * a cleft of shadow at its left.
   */
  const jog = (y: number, seed: number) => Math.round((roll(y / 3, seed) - 0.5) * 3.2)
  const fallsAt = (x: number, y: number, c: RGB, near: boolean): RGB => {
    for (const f of falls) {
      if (near !== f.near || x < f.x - 3 || x > f.x + 4) continue
      const top = near ? nearRidge(f.x) + 1.5 : farRidge(f.x) + 1.5
      if (y < top - 1 || y > 26) continue
      const at = f.x + jog(y, f.x * 7 + (near ? 3 : 0)), far = near ? 0.12 : 0.3
      if (y < top) { if (x === at) c = mul(c, [0.75, 0.75, 0.8]) } // the cleft it springs from
      else if (x === at - 1) c = mul(c, [0.82, 0.82, 0.86])
      else if (x === at) c = falling(c, x, y + f.x, near ? 1.2 : 0.7, 1, far, true)
      else if (x === at + 1) c = falling(c, x, y + f.x + 3, near ? 1.2 : 0.7, near ? 0.9 : 0.55, far, true)
      else if (near && x === at + 2) c = falling(c, x, y + f.x + 5, 1.2, 0.4, far, true)
    }
    return c
  }
  /** And the spray where the near one lands, among the firs' tips. */
  const spray = falls.find(f => f.near)
  const sprayBase = spray === undefined ? 0 : firTop(spray.x), sprayAt = spray === undefined ? 0 : spray.x + jog(sprayBase, spray.x * 7 + 3) + 0.5
  const mist = (x: number, y: number, c: RGB): RGB => {
    if (spray === undefined || winter || Math.abs(y - sprayBase) >= 1.3) return c
    const base = sprayBase, d = Math.hypot((x - sprayAt) / 2.6, (y - base) / 1.3)
    return d < 1 ? mix(c, lit(hex(0xf4f9fc), 0.2), (1 - d) * 0.6 * (0.7 + 0.3 * Math.sin(now() / 3 + x))) : c
  }

  // the forge's glow through the door, evening and night: a warm pool on the floor
  const spill = (x: number, y: number, c: RGB): RGB => {
    if (sky.spill <= 0 || y < 22) return c
    const d = Math.hypot((x - from) / 1.6, (y - 31) * 1.4)
    const f = Math.max(0, 1 - d / 14) ** 2 * sky.spill
    return [c[0] + 230 * f, c[1] + 135 * f, c[2] + 63 * f]
  }

  // the clouds' noise, rolling over a plane: along a column, at whole rows of the plane,
  // that two or three rows of pixels share, worked out once a frame
  const broad = new Float64Array(HEIGHT * Math.max(0, to - from)), fine = new Float64Array(broad.length)
  broad.fill(NaN); fine.fill(NaN)
  const rolled = (known: Float64Array, x: number, along: number, down: number, seed: number) => {
    const j = Math.floor(down), f = down - j, u = f * f * (3 - 2 * f), i = j * (to - from) + x - from
    let a = known[i] ?? NaN, b = known[i + to - from] ?? NaN
    if (a !== a) known[i] = a = roll(along, seed + j * 131)
    if (b !== b) known[i + to - from] = b = roll(along, seed + (j + 1) * 131)
    return a * (1 - u) + b * u
  }
  const skyAt = (x: number, y: number): RGB => {
    let c = mix(sky.top, sky.low, clamp01(y / HORIZON))
    if (sky.night && !sky.storm && y < 18 && rand(x * 97 + y * 13) > 0.975) c = rand(x + y) > 0.6 ? [250, 245, 220] : [170, 180, 220]
    // the clouds, drifting right to left; thick and quick in a storm. Their noise stays
    // under 1: a row whose band cannot reach the threshold has none, whatever the hour.
    const speed = sky.storm ? 0.04 : 0.012, band = 1 - Math.abs(y - 6) / 11, thin = sky.storm ? 0.22 : 0.5
    if (band <= thin) return c
    const t = now()
    const d = (rolled(broad, x, x / 10 + t * speed, y / 3, 11) * 0.65 + rolled(fine, x, x / 5 + t * speed * 1.4, y / 2, 23) * 0.35) * band
    const a = smooth(thin, sky.storm ? 0.4 : 0.66, d)
    if (a <= 0) return c
    const under = clamp01((y - 2) / 9) // a cloud is darker underneath
    const cloud = sky.storm ? mix(hex(0x4e5260), hex(0x2c303c), under)
      : sky.night ? mix(hex(0x2c3554), hex(0x1a2240), under)
      : sky.dusk ? mix(hex(0xf2b8a6), hex(0x8a5a7a), under)
      : mix(hex(0xf6f8fb), hex(0xbcc6d4), under)
    return mix(c, sky.flash ? mix(cloud, [230, 232, 245], 0.4) : cloud, a * (sky.night ? 0.8 : 1))
  }

  /**
   * Two ranges and the firs over the sky: each ridge smoothed where it crosses a pixel.
   * What one of them covers whole, the sky and the far range behind, is not painted.
   */
  const hills = (x: number, y: number): RGB => {
    if (y >= firTop(x) || y >= 26) return mist(x, y, lit(winter && (x + y) % 3 === 0 ? hex(0xdfe8f0) : hex(0x23402c), 0.05))
    const near = nearRidge(x), nearColour = () => lit(winter && y < near + 1 ? hex(0xc8d4e0) : hex(0x3e5560), 0.38)
    let c: RGB
    if (y >= near) c = nearColour()
    else {
      const far = farRidge(x)
      const farColour = () => lit(y < far + (winter ? 4 : 1.5) || y < 11 ? hex(0xe8eef4) : hex(0x5d6e86), 0.6) // the high peaks keep their snow
      c = y >= far ? farColour() : y + 1 > far ? mix(skyAt(x, y), farColour(), y + 1 - far) : skyAt(x, y)
      c = fallsAt(x, y, c, false)
      if (y + 1 > near) c = mix(c, nearColour(), y + 1 - near)
    }
    c = fallsAt(x, y, c, true)
    return mist(x, y, c)
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

  /** The tree: its crown sways with the wind (harder in a storm), and keeps the season. What it puts on (x, y), or nothing. */
  const crown = (x: number, y: number): RGB | undefined => {
    const { x: tx, r } = tree, ty = r + 0.5
    const gust = sky.storm ? 1.4 : 0.6
    if (Math.abs(x - tx) >= (r + 2.5) / 0.85 + gust) return undefined
    // the sway, read only where it can carry a leaf or a bough onto this pixel or off it
    const give = clamp01((ty + r - y) / (2 * r)), least = Math.abs(x - tx) - gust * give - 1e-6
    const still = give <= 0 || (least > r - 1 + 0.55 && Math.hypot(least * 0.85, y - ty) >= r + 2.5)
    const sx = still ? x : x - Math.sin(now() / (sky.storm ? 3 : 7) + y / 3) * gust * give
    const leafy = y < ty + r * 0.75 && Math.hypot((sx - tx) * 0.85, y - ty) < r + roll(Math.atan2(y - ty, sx - tx) * 3 + 5, 5) * 2.5
    if (leafy && !winter) {
      const k = rand(Math.round(sx) * 13 + y * 29)
      const leaf = se === 'autumn' ? (k > 0.66 ? 0xe07a22 : k > 0.33 ? 0xc04a1a : 0xe3a63c)
        : se === 'spring' ? (k > 0.85 ? 0xf6b8d0 : k > 0.4 ? 0x6fb04a : 0x4f8a36)
        : k > 0.6 ? 0x3c7a30 : 0x2f6228
      return lit(leafOf(leaf, sx > tx + 2 || y > ty + 4)) // in the shade away from the light, from the left like the room's
    }
    const trunk = Math.abs(x - tx) <= 1 && y >= ty + 3 && y < GROUND + 2
    // the boughs: three pairs fanning out from the trunk, seen whole once the leaves are down
    const up = ty + 3 - y, off = Math.abs(sx - tx)
    let bough = false
    if (up >= 0 && y > ty - r * 0.8 && off <= r - 1) for (let i = 0; i < BOUGHS.length && !bough; i++) bough = Math.abs(off - up * (BOUGHS[i] ?? 0)) < 0.55 && up > i * 1.5
    if (trunk || bough) return lit(x === tx - 1 ? hex(0x6a4228) : hex(0x4a2c16))
    if (winter && leafy && y % 3 === 0 && rand(x * 5 + y) > 0.55) return lit(hex(0xeef2f8)) // snow on the bare twigs
    return undefined
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

  // the wheel: water from the race on the right falls into the buckets at its crown and
  // rides them down the near side, spilling as they pass the bottom; it turns faster
  // while he works (it drives the bellows), and stands frozen in winter
  const turn = () => winter ? 0.4 : now() * (working ? 0.22 : 0.09) // frozen in winter, the clock unread
  const CY = 19, R = 7, STEP = (2 * Math.PI) / 10
  /** How far down the near side an angle on the wheel is: 0 at the crown, π at the foot; out of [0, π] on the far side. */
  const down = (th: number) => th < 0 ? -Math.PI / 2 - th : 1.5 * Math.PI - th
  /** How full the buckets are there: filled at the crown, full down the side, emptied past the bottom. */
  const fill = (q: number) => smooth(0.05, 0.45, q) * (1 - smooth(1.5, 2.3, q))
  // the buckets spilling now: where each one's lip is (within R + 1.6 of the axle), and how much it pours
  let pouring: Array<{ x: number; y: number; f: number }> | undefined
  const spills = () => {
    if (wheel === undefined || winter) return []
    const turned = turn()
    return (pouring ??= Array.from({ length: 10 }, (_, k) => {
      const th = Math.atan2(Math.sin(k * STEP - turned), Math.cos(k * STEP - turned))
      const q = down(th), f = smooth(1.3, 1.6, q) * (1 - smooth(2.1, 2.4, q))
      return { x: Math.round(wheel + (R + 1.6) * Math.cos(th)), y: CY + (R + 1.6) * Math.sin(th), f }
    }).filter(s => s.f > 0.05))
  }
  const millAt = (x: number, y: number, c: RGB): RGB => {
    if (wheel === undefined || x < wheel - 19) return c
    const cx = wheel
    // the race on its trestle, from beyond the edge to over the wheel
    if (x === to - 2 && y > 11 && y < GROUND) c = lit(hex(0x5a3a1e))
    if (y === 11 && x >= cx - 1) return lit(hex(0x7a4a24))
    if (y === 10 && x >= cx) return winter ? lit(hex(0xbfe0f0)) : rand(Math.floor(x + now() * 0.6) * 5) > 0.75 ? lit(hex(0xdff6ff)) : lit(hex(0x7fd4ff))
    if (winter && y === 12 && x >= cx && x % 2 === 0) return lit(hex(0xcfe8f6)) // icicles
    const under = () => spills().find(s => Math.abs(s.x - x) <= 1)
    const b = shore(x, y, c)
    if (b) return b
    // the pond at its foot, white where the spill lands
    if (wet(x, y)) {
      if (winter) return lit(hex(0xd8ecf6))
      const u = y === GROUND - 1 ? under() : undefined
      if (u && rand(x * 7 + now() * 13) < 0.3 + u.f * 0.6) return lit(hex(0xf2fbff))
      if (y > GROUND) {
        // the brook running out of it toward us, a little faster at its middle
        const u = Math.floor((y - x) / 2 - now() * 0.45)
        return rand(u * 7 + ((x + y) % 3)) > 0.82 ? lit(hex(0xdff6ff)) : mul(lit(hex(0x5fb4e0)), [0.86, 0.92, 1])
      }
      return rand(Math.floor(x + now() * 0.3) * 11 + y) > 0.85 ? lit(hex(0xdff6ff)) : mul(lit(hex(0x5fb4e0)), [0.9, 0.95, 1])
    }
    // the splashes jumping off it, a pixel high
    const u = y === GROUND - 2 ? under() : undefined
    if (u && rand(x * 5 + now() * 17) > 1 - u.f * 0.4) return mix(c, lit(hex(0xeef8ff)), 0.8)
    c = wheelAt(x, y, c)
    if (winter) return c
    // off the race's lip and down into the bucket going by under it
    if ((y === 10 && x === cx - 1) || (x === cx - 2 && (y === 11 || y === 12))) return falling(c, x, y, 1.6, 0.9)
    // what the spilling buckets let go, straight down to the pond in threads
    if (Math.abs(x - cx) <= R + 2 && y > CY - R - 2 && y < GROUND - 1) {
      for (const s of spills()) if (x === s.x && y > s.y) return falling(c, x, y + x * 2, 1.6, 0.9 * s.f, 0, false, true)
    }
    return c
  }
  /** The pond under the wheel, and the brook it lets out down the yard toward us, widening as it comes. */
  const wet = (x: number, y: number) => {
    if (wheel === undefined) return false
    const dx = x - wheel
    return y === GROUND - 1 ? dx >= -8 : y === GROUND ? dx >= -10
      : y === GROUND + 1 ? dx >= -13 && dx <= -8 : y === GROUND + 2 ? dx >= -15 && dx <= -10 : y === GROUND + 3 && dx >= -17 && dx <= -11
  }
  /** Its banks (wet grass, a pebble here and there), and its reeds that bend with the wind. */
  const shore = (x: number, y: number, c: RGB): RGB | undefined => {
    if (wheel === undefined) return undefined
    const dx = x - wheel
    // the reeds on the pond's near shore and by the brook, their heads swaying
    for (const [rx, h] of [[-11, 4], [-13, 3], [-7, 2]] as const) {
      const base = rx === -7 ? GROUND + 1 : GROUND
      if (dx === rx && y <= base && y > base - h) return lit(winter ? hex(0xb8a57c) : hex(0x3e6a2a))
      // a head sways a pixel at most either way: the wind is read only that near
      if (h < 3 || Math.abs(dx - rx) > 1 || (y !== base - h && y !== base - h - 1)) continue
      const sway = sky.storm ? Math.round(Math.sin(now() / 2 + rx)) : Math.round(Math.sin(now() / 6 + rx) * 0.6)
      if (dx === rx + sway) return lit(winter && y === base - h - 1 ? hex(0xeef2f8) : hex(0x6a4022))
    }
    if (y < GROUND || wet(x, y)) return undefined
    if (wet(x - 1, y) || wet(x + 1, y) || wet(x, y - 1) || wet(x, y + 1)) {
      if (winter) return lit(hex(0xdde6f0))
      return rand(x * 13 + y * 7) > 0.8 ? lit(hex(0x8a8494)) : mul(c, [0.72, 0.74, 0.7]) // wet grass, a pebble here and there
    }
    return undefined
  }
  const wheelAt = (x: number, y: number, c: RGB): RGB => {
    if (wheel === undefined) return c
    const cx = wheel
    const d = Math.hypot(x - cx, y - CY)
    if (d > R + 2) return c
    if (d < 1.5) return lit(hex(0x4a4e5c))
    // the water comes along the race from the right onto its top: the top goes left (anticlockwise)
    const th = Math.atan2(y - CY, x - cx), ang = th + turn()
    const near = (n: number) => { const s = (2 * Math.PI) / n, a = ((ang % s) + s) % s; return Math.min(a, s - a) * d }
    // drawn with soft edges, so it turns smoothly rather than by whole pixels
    const paddle = d > R && d < R + 2 ? clamp01(1.5 - near(10) / 0.7) : 0
    const rim = clamp01(1.6 - Math.abs(d - R) / 0.6)
    const inner = clamp01(1.2 - Math.abs(d - R + 2.5) / 0.5)
    const spoke = d < R ? clamp01(1.4 - near(6) / 0.6) : 0
    c = mix(c, lit(hex(0x3a2214)), Math.max(spoke, inner * 0.8))
    // the water in the buckets, resting on each paddle's upper face, turning with it
    if (!winter && d > R + 0.3) {
      const u = ((ang % STEP) + STEP) % STEP, depth = fill(down(th)) * STEP * 0.6
      if (u > 0.05 && u < 0.05 + depth) c = mix(c, lit(rand(Math.round(u * 20) + Math.round(d) * 3 + now()) > 0.85 ? hex(0xdff6ff) : hex(0x4fb0f0)), 0.9)
    }
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
      if (sky.storm && y === eave - 1 && x > from && (x - Math.floor(now() / 2)) % 3 === 0) return lit(hex(0x7fd4ff))
    }
    if (sky.storm && x === EAVE_END) {
      // and off the end, drop after drop, down to the ground
      const lip = Math.floor((EAVE_END - from) / 3) + 3
      for (let k = 0; k < 2; k++) if (y === lip + ((Math.floor(now() * 1.5) + k * 9) % (GROUND - lip))) return lit(hex(0x9fdcff))
    }
    if (x <= from + 1 && y >= 2) return lit(x === from ? hex(0x4a2c16) : hex(0x6a4228))
    if (y >= 30 && x <= from + 2) return lit(y === 30 ? hex(0x8a8494) : hex(0x5d5866))
    return undefined
  }

  /**
   * The Route du Rhum: the bay of Saint-Malo where the mountains were, the fleet on it (tacking
   * about before the gun, racing out to the Atlantic after it), two gulls over it, in a wide
   * yard an islet and its lighthouse, and the quay's granite under foot. Neither tree, well nor
   * wheel: the sea has their place.
   */
  const rhum = w.rhum, room = to - from
  const fleet = rhum === undefined ? [] : fleetOf(room)
  const lamps = sky.night || sky.dusk || sky.storm // the boats' lights, the lighthouse's
  let placed: Array<{ s: Sail; x: number; dir: 1 | -1 }> | undefined
  /** Where the boats are in this frame: read by every pixel that could carry one, so that one moves. */
  const boats = () => {
    const t = now() // read by every pixel asking, not only the first: each one is marked moving
    return (placed ??= fleet.map(s => ({ s, ...sailing(s, t, rhum ?? 'race', from, to) })))
  }
  const seaAt = (x: number, y: number): RGB => {
    const k = (y - SEA) / (GROUND - 1 - SEA) // 0 at the horizon, 1 against the quay
    let c = lit(mix(sky.storm ? hex(0x34424e) : hex(0x1d5a78), sky.storm ? hex(0x4a5a64) : hex(0x2c8090), k), 0.45 * (1 - k))
    // the wavelets: dashes of light going with the swell, longer and quicker toward us; more whitecaps in a storm
    const len = 2 + k * 4, speed = (0.04 + k * 0.12) * (sky.storm ? 2 : 1)
    if (rand(Math.floor((x + now() * speed) / len) * 13 + y * 101) > (sky.storm ? 0.8 : 0.88)) c = mix(c, lit(hex(0xe2f2f6), 0.3 * (1 - k)), sky.storm ? 0.8 : 0.5)
    // the surf against the quay
    if (y === GROUND - 1 && rand(x * 7 + Math.floor(now() / 2) * 3) > 0.62) c = mix(c, lit(hex(0xf2fbff)), 0.7)
    return c
  }
  const boatAt = (x: number, y: number): RGB | undefined => {
    const all = boats()
    // the nearest first: it hides those behind
    for (let i = all.length - 1; i >= 0; i--) {
      const { s, x: bx, dir } = all[i] as { s: Sail; x: number; dir: 1 | -1 }
      const rows = s.hull.rows, wide = rows[0]?.length ?? 0, top = s.lane - (rows.length - 1 - s.hull.under)
      if (y < top || y > s.lane + s.hull.under || x < bx || x >= bx + wide) continue
      const ch = rows[y - top]?.[dir > 0 ? x - bx : bx + wide - 1 - x]
      if (ch === undefined || ch === '.') continue
      // after dusk, the masthead light and the bow's, green or red by the side we see
      if (lamps && ch === 'm' && y === top) return hex(0xfff6d0)
      if (lamps && ch === 'h') return dir > 0 ? hex(0x6fe07a) : hex(0xff5a4a)
      const own = s.colours
      const c = ch === 'm' ? hex(0x2a2a30) : ch === 'b' ? hex(0x3a3a44) : ch === 'J' ? mul(hex(own.M), [0.88, 0.9, 0.94])
        : ch === 'f' ? mul(hex(own.H), [0.7, 0.7, 0.75]) : hex(own[ch as 'H' | 'h' | 'M' | 'S'] ?? own.H)
      const seen = lit(c, s.far)
      return y > s.lane ? mix(seaAt(x, y), seen, 0.5) : seen // what is in the water, half seen
    }
    return undefined
  }
  /** How white the water is at (x, y): the bow wave, and the wake behind, long when they race. */
  const foamAt = (x: number, y: number): number => {
    let f = 0
    for (const { s, x: bx, dir } of boats()) {
      if (y !== s.lane && y !== s.lane + 1) continue
      const wide = s.hull.rows[0]?.length ?? 0, len = Math.round(wide * (rhum === 'race' ? 1.4 : 0.5))
      const bow = dir > 0 ? bx + wide : bx - 1, back = ((dir > 0 ? bx - 1 : bx + wide) - x) * dir
      if (rhum === 'race' && x === bow && y === s.lane) f = Math.max(f, 0.9)
      if (back >= 0 && back < len && rand(x * 13 + y * 7 + Math.floor(now() * 0.7) * 31) < (1 - back / len) * (y === s.lane ? 0.8 : 0.45)) f = Math.max(f, 1 - back / len)
    }
    return f
  }
  // the islet and its lighthouse, out on the right of a wide bay, its light turning after dusk
  const isle = room >= 44 ? to - 10 : undefined
  const beacon = () => lamps && Math.floor(now()) % 28 < 3
  const isleAt = (x: number, y: number): RGB | undefined => {
    if (isle === undefined || y > SEA || y < SEA - 11) return undefined
    const d = Math.abs(x - isle - 0.5)
    if (d < 7 && y >= SEA + 0.5 - (3 - d * 0.45 + roll(x / 2, 61) * 1.2)) return lit(rand(x * 5 + y * 3) > 0.7 ? hex(0x8a8494) : hex(0x5d5866), 0.35)
    if (x !== isle && x !== isle + 1) return undefined
    if (y === SEA - 11) return lit(hex(0x2a2a30), 0.3) // the lantern's cap
    if (y === SEA - 10) return lamps ? (beacon() ? hex(0xfff2a0) : lit(hex(0x7a7050), 0.3)) : lit(hex(0x9ab0c0), 0.3)
    if (y <= SEA - 3) return lit(y <= SEA - 8 ? hex(0xc0392b) : hex(0xeef0f4), 0.3)
    return undefined
  }
  let flock: ReturnType<typeof gulls> | undefined
  const gullAt = (x: number, y: number) => {
    if (sky.night || y < 3 || y > 13) return false // they sleep at night
    const t = now()
    for (const g of (flock ??= gulls(t, from, to))) {
      const dx = x - g.x, dy = y - g.y
      if (g.up ? (Math.abs(dx) === 1 && dy === -1) || (dx === 0 && dy === 0) : (Math.abs(dx) <= 1 && dy === 0) || (Math.abs(dx) === 2 && dy === 1)) return true
    }
    return false
  }
  const quay = (x: number, y: number): RGB => {
    if (y === GROUND) return lit(winter ? hex(0xeef2f8) : (x - from) % 6 === 0 ? hex(0x5d5866) : hex(0xa8a4ae)) // its coping stones, along the edge
    if (winter && rand(x * 3 + y * 5) > 0.35) return lit(hex(0xeaf0f7))
    const row = y % 2, stone = Math.floor((x + row * 2) / 4)
    return lit((x + row * 2) % 4 === 0 ? hex(0x4a4048) : rand(stone * 13 + y) > 0.5 ? hex(0x8a8490) : hex(0x76707c))
  }
  const bollard = room >= 26 ? from + 19 : undefined
  const buoys = rhum === 'before' && room >= 20 ? [from + Math.round(room * 0.3), from + Math.round(room * 0.72)] : []
  // the bunting, from the eave's end to the yard's edge, as on the quays the week of the start
  const BUNTING = [0xd04a2a, 0xf2c14e, 0x1d5a9a, 0xf4f4f0, 0x3c8a3c]
  const flagAt = (x: number, y: number): RGB | undefined => {
    const x0 = from + 12, x1 = to - 1
    if (x <= x0 || x >= x1 || y > 7) return undefined
    const rope = 2 + Math.round(2.5 * Math.sin((Math.PI * (x - x0)) / (x1 - x0)))
    if (y === rope) return lit(hex(0x3a3a44))
    const k = x - x0
    return k % 3 !== 0 && (y === rope + 1 || (y === rope + 2 && k % 3 === 1)) ? lit(hex(BUNTING[Math.floor(k / 3) % BUNTING.length] ?? 0xf4f4f0)) : undefined
  }
  const harbour = (x: number, y: number): RGB => {
    if (y >= GROUND) return quay(x, y)
    const flag = flagAt(x, y)
    if (flag) return flag
    if (bollard !== undefined && y >= GROUND - 2 && (x === bollard || x === bollard + 1 || (y === GROUND - 2 && x === bollard - 1))) return lit(y === GROUND - 2 ? hex(0x3a3a44) : hex(0x24242c))
    const boat = y >= 14 ? boatAt(x, y) : undefined
    if (boat) return boat
    if (y < SEA || (y === SEA && isle !== undefined)) {
      const rock = isleAt(x, y)
      if (rock) return rock
    }
    if (y < SEA) {
      if (gullAt(x, y)) return lit(hex(0xf4f4f0))
      const c = skyAt(x, y)
      // the lighthouse's beam, sweeping out over the bay as it flashes
      return isle !== undefined && y === SEA - 10 && x < isle && isle - x < 14 && beacon() ? mix(c, hex(0xfff2a0), 0.55 * (1 - (isle - x) / 14)) : c
    }
    let c = seaAt(x, y)
    // the start line's buoys, bobbing
    for (const bx of buoys) {
      const top = x === bx && y >= 22 && y <= 24 ? 22 + (Math.sin(now() / 4 + bx) > 0 ? 0 : 1) : -1
      if (y === top || y === top + 1) c = lit(y === top ? hex(0xf2c14e) : hex(0xc08a20), 0.15)
    }
    const f = y >= 18 ? foamAt(x, y) : 0
    return f > 0 ? mix(c, lit(hex(0xf2fbff)), 0.45 + 0.45 * f) : c
  }

  const paint = (x: number, y: number): number => {
    let c = door(x, y)
    if (c === undefined && rhum !== undefined) c = harbour(x, y)
    else if (c === undefined) {
      // under the leaves, neither sky nor hills are painted
      c = crown(x, y) ?? (y >= GROUND ? ground(x, y) : hills(x, y))
      c = millAt(x, y, wellAt(x, y, c))
    }
    return pack(spill(x, y, c))
  }
  // painted whole once a sky, a season and a width; then, frame after frame, only what moves
  const width = Math.max(0, to - from)
  const key = `${from},${to},${se},${sky.night},${sky.dusk},${sky.storm},${sky.flash},${rhum}`
  let still = stills.get(key)
  const colours = new Int32Array(width * HEIGHT)
  if (still === undefined) {
    const moving: number[] = []
    for (let y = 0, i = 0; y < HEIGHT; y++) {
      for (let x = from; x < to; x++, i++) {
        moved = false
        colours[i] = paint(x, y)
        if (moved) moving.push(i)
      }
    }
    if (stills.size >= STILLS) stills.delete(stills.keys().next().value ?? '')
    stills.set(key, { colours: colours.slice(), moving: Int32Array.from(moving) })
  } else {
    colours.set(still.colours)
    for (const i of still.moving) colours[i] = paint(from + (i % width), Math.floor(i / width))
  }
  return {
    back: (x: number, y: number): number | undefined => (x >= from && x < to && y >= 0 && y < HEIGHT ? colours[y * width + x - from] : undefined),
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
  if ((se === 'autumn' || se === 'spring') && w.rhum === undefined) { // by the sea, no tree
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
