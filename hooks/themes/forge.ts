// The forge: Scorpheus' theme, the first, drawn by hand in pixel art after his
// Codex sheet. A wolf-dragon smith at his anvil, a furnace, a window, a rack
// for the days' work, the apprentices' yard. Everything here is code: a theme
// of one's own is a JSON file instead (themes/data.ts reads it).
//
// Two inputs make a frame: the moment (an Activity and the ticks since it
// began) and the World. The moment poses the smith; the world dresses the
// workshop around him whatever he is doing.
//
// Every grid is one letter per pixel, '.' transparent; each letter must be in PALETTE.

import type { Activity } from '../../types'
import { BASE_PALETTE, BASE_VARIANTS, CLAWS, GIFTS, PUFF, RAVEN, SAYING, SPARKS, BUBBLE, THOUGHT, Z_BIG, Z_SMALL, drawDream, drawSky, drawSparks, feast, rareFor, season } from '../props.ts'
import { erase, line, put, rand, stamp, toCells } from '../pixels.ts'
import type { Grid } from '../pixels.ts'
import { CALM } from '../world.ts'
import type { Crop, Cub, Dream, Guest, Theme, Weapon, World } from '../world.ts'
import { FORGE_WORDS } from './forge-words.ts'

export { CALM }
export const PALETTE = BASE_PALETTE

/**
 * The smiths' names, one a session: Scorpheus is the person's own name, each
 * wolf-dragon at a forge has his. Scientists, philosophers and folk of Middle-earth,
 * short, for the line under the scene.
 */
export const SMITHS: readonly string[] = [
  'Curie', 'Tesla', 'Newton', 'Lovelace', 'Darwin', 'Kepler', 'Faraday', 'Galilée', 'Euler', 'Turing', 'Pasteur', 'Noether',
  'Socrate', 'Spinoza', 'Diogène', 'Montaigne', 'Épicure', 'Hypatie', 'Platon', 'Kant', 'Descartes', 'Pascal',
  'Gandalf', 'Aragorn', 'Gimli', 'Elrond', 'Faramir', 'Éomer', 'Legolas', 'Galadriel', 'Bilbon', 'Sylvebarbe',
]

export type Accessory = 'aucun' | 'lunettes' | 'tablier' | 'bandeau'
export const ACCESSORIES: readonly Accessory[] = ['aucun', 'lunettes', 'tablier', 'bandeau']

export const SW = 72 // scene width in pixels (= columns): the forge, then the apprentices' yard
export const YARD = 54 // where the yard begins, past the banner
export const SH = 32 // scene height in pixels (two per row)
export const ROWS = SH / 2

/** Horizontal windows over the scene, widest first: the band shows the widest that fits. */
export const CROPS = {
  full: [0, SW],
  compact: [14, SW], // the anvil, Scorpheus and the yard
  solo: [29, YARD], // Scorpheus alone: the cubs then live in the words only
} as const
export const cropWidth = (c: Crop) => CROPS[c][1] - CROPS[c][0]

/** Where a guest stands, from the anvil: the yard; and where anyone leaves the scene. */
export const GUEST_SPOT = 26
export const OFFSTAGE = 44

// --- the cast -----------------------------------------------------------------

// The skull without its features: they are drawn on it, slid sideways when he turns his head.
const SKULL = [
  '.k............k.',
  'kwk.....k....kwk',
  'kgwk..nkbk..kwgk',
  'kgdwknBbbBnkkdgk',
  'kgdlnBbBbBBnldgk',
  '.kglnBbBBbnBlgk.',
  '.klllnBnBnBllldk',
  'kglllllnlnllllgk',
  'kgllllllllllllgk',
  'kllllllllllllllk',
  'kllllllllllllllk',
  'kgllllllllllllgk',
  '.kgllllllllllgk.',
  '.kgllllllllllgkB',
]

// Muzzle, nose, gold moon and teal scales, facing front.
const MARKS: ReadonlyArray<readonly [number, number, string]> = [
  [5, 11, 'w'], [6, 11, 'w'], [7, 11, 'm'], [8, 11, 'm'], [9, 11, 'w'], [10, 11, 'w'],
  ...[5, 6, 7, 8, 9, 10].map(x => [x, 12, 'w'] as const),
  ...[6, 7, 8, 9].map(x => [x, 13, 'w'] as const),
  [14, 8, 'y'], [14, 9, 'y'], [14, 10, 'y'], [11, 11, 'y'], [12, 11, 'y'], [13, 11, 'y'],
  [1, 10, 't'], [2, 11, 't'], [12, 12, 't'],
]

const BODY = [
  '.oOkkllwwllkkOoB', // collar of the hood
  '..koOrwwwwrOok..',
  '..koOOrwwrOOok..',
  '..koOOOrrOOOok..',
  '..krrrryyrrrrk..', // belt, gold buckle
  '..kOrrkOOkrrOk..', // pouches
  '..koOOkOOkOOok..',
  '...kwwwkkwwwk...', // fur
  '...kwlwkkwlwk...',
  '...kglgkkglgk...',
  '..kaaaakkaaaak..', // boots
  '..kkkkk..kkkkk..',
]

// Seated on his stool: the cloak down to the pouches, then folded legs.
const BODY_SEATED = [...BODY.slice(0, 7), '..kwwwwkkwwwwk..']

// The tail, white with the blue band and the gold stripe; two sways.
const TAIL = [
  ['....kk.', '...kwwk', '..kwbwk', '..kwBbk', '.kwwBbk', '.kwyBwk', 'kwwyBwk', 'kwwBbwk', '.kwbwk.', '.kwwk..', '..kk...'],
  ['.....kk', '....kwk', '...kwbk', '..kwBbk', '.kwwBbk', '.kwyBwk', 'kwwyBwk', 'kwwBbwk', '.kwbwk.', '.kwwk..', '..kk...'],
]

const FURNACE = [
  '....kkkkk......',
  '....kQqqk......',
  '....kqQqk......',
  '....kqqQk......',
  '....kQqqk......',
  '..kkkkkkkkkk...',
  '.kQqqQqqqQqqk..',
  'kqqzqqqzqqqzqk.',
  'kqQqqqQqqqQqqk.',
  'kzqqqzqqqzqqzk.',
  'kqqkkkkkkkkqqk.',
  ...Array<string>(9).fill('kqkzzzzzzzzkqk.'), // the hearth; its bottom row holds the logs
  'kqqkkkkkkkkqqk.',
  'kQqqzqqqzqqqQk.',
  'kkkkkkkkkkkkkk.',
]
const FURNACE_AT = { x: 0, y: 8 }
const MOUTH = { x: 3, y: 18, w: 8, h: 8 } // the fire's room
const LOGS_Y = 26 // the hearth's floor: one log per 12.5 % of the 5 h window left
// The woodpile on the roof, bottom first: one log per 10 % of the week left.
const WOODPILE: ReadonlyArray<readonly [number, number]> = [
  [0, 12], [1, 12], [2, 12], [3, 12], [0, 11], [1, 11], [2, 11], [3, 11], [1, 10], [2, 10],
]
// Mounted on the furnace's right wall, its nozzle in the hearth, its lever up to the right.
const BELLOWS = { x: 13, y: 20 }
const BELLOWS_SPOT = -8 // where Scorpheus stands: at arm's length, the bellows in sight

const ANVIL = [
  'kkkkkkkkkkkkk',
  'kAAAAAAAAAAAk',
  '.kkaaaaaaakk.',
  '...kaaaaak...',
  '...kaaaaak...',
  '..kaaaaaaak..',
  '.kkkkkkkkkkk.',
]
const ANVIL_AT = { x: 16, y: 25 }
const INGOT = { x: 21, y: 24, w: 4 }
const IMPACT = { x: 22, y: 23 }

const C = { x: 30, y: 6 } // Scorpheus' head, top-left

const SCROLL = ['kWWWWWWk', 'kWVVVVWk', 'kWWWWWWk', 'kWVVVWWk', '.kkkkkk.']
const BUCKET = ['kcccccck', 'kAAAAAAk', 'krRrRrRk', 'krRrRrRk', '.kkkkkk.']
const BUCKET_AT = { x: 18, y: 27 }
const CRATE = ['krrrrrrk', 'kRrRRrRk', 'krrrrrrk', 'kRrRRrRk', 'kkkkkkkk']
const CRATE_AT = { x: 18, y: 19 }
const HOURGLASS_X = [17, 21, 25] // on the anvil, left to right
const BELL_AT = { x: 25, y: 0 }

// Scraps that pile up on the floor as the context fills, in the order they fall.
const SCRAPS: ReadonlyArray<readonly [number, number, string]> = [
  [15, 31, 'a'], [29, 31, 'A'], [16, 31, 'r'], [14, 30, 'E'], [28, 31, 'a'], [15, 30, 'R'],
  [17, 30, 'A'], [30, 31, 'r'], [27, 30, 'E'], [14, 31, 'a'], [16, 30, 'r'], [28, 30, 'a'],
]

// The session's gold, bottom first: one coin per step of the cost's logarithm.
const COINS: ReadonlyArray<readonly [number, number]> = [
  [49, 31], [50, 31], [51, 31], [52, 31], [53, 31], [50, 30], [51, 30], [52, 30], [51, 29], [52, 29], [51, 28],
]

// The workshop's window onto the world, between the bell and the furnace.
const WINDOW = { x: 13, y: 0, w: 12, h: 10 } // frame included; panes x 14..23, y 1..8
const LANTERN = { x: 10, y: 0 }
const HATS = {
  party: ['..Y..', '..M..', '.MyM.', '.yMy.', 'MMyMM'],
  santa: ['...w.', '..FF.', '.FFF.', 'FFFFE', 'wwwww'],
}
const HAMMERS = { small: ['AA', 'aa'], normal: ['AAA', 'aaa'], sledge: ['AAAA', 'aaaa', 'aaaa'] }

// The week's rack under the window: a rail and seven pegs, one weapon a day worked.
const RACK = { x: 13, y: 11, slots: [15, 17, 19, 21, 23, 25, 27] }
// Each weapon top down from under the rail; [dx, dy, ch] beside the shaft.
const WEAPONS: Record<Weapon, { shaft: string; side: ReadonlyArray<readonly [number, number, string]> }> = {
  sword: { shaft: 'wAAAyr', side: [[-1, 4, 'y'], [1, 4, 'y']] },
  axe: { shaft: 'AArrrr', side: [[-1, 0, 'A'], [-1, 1, 'a']] },
  dagger: { shaft: '..wAyr', side: [] },
  mace: { shaft: 'Aarrrr', side: [[-1, 0, 'a'], [1, 0, 'a']] },
}
// The project's banner, hanging right of Scorpheus.
const BANNER = { x: 47, y: 8 }
const DUCK = ['.yy..', 'Fky..', '.yyyy', '..yyy']
const DUCK_AT = { x: 25, y: 21 }
const SLATE_AT = { x: 18, y: 17 }

// A wolf cub in profile, facing right: grey coat, light belly, long snout, the
// blue tuft of its father, a scarf in its type's colour ('C' here).
// Small beside its father: 12 x 7, so the floor holds several without hiding him.
const CUB_BODY = [
  '........d.d.',
  '.......dgBd.',
  'dd....dggpgd',
  'dgdd.CCglwwm',
  '.dddCCgggww.',
  '..gwwwwwgl..',
]
// on the trot's passing steps the tail dips
const CUB_TAIL_LOW = ['d.....dggpgd', 'dddd.CCglwwm']
// a four-step trot (near legs light, far in shadow), then sitting to work
const CUB_LEGS = [['..d.g..d.g..'], ['...gd...gd..'], ['..g.d..g.d..'], ['...dg...dg..'], ['..dddd.d.d..']]
const CUB_SIT = 4
const CUB_W = 12
const VISIT_EVERY = 3200 // ticks between visitors: eight minutes
const VISIT_FOR = 56

// --- poses ---------------------------------------------------------------------

type Eyes = 'open' | 'look' | 'blink' | 'happy' | 'sad'
type Mouth = 'none' | 'smile' | 'open' | 'frown' | 'grit'
type LeftArm = 'rest' | 'raise' | 'strike' | 'scroll' | 'drop' | 'pull' | 'blade' | 'dip' | 'seal' | 'broom' | 'lap'
type RightArm = 'rest' | 'wave1' | 'wave2' | 'up' | 'scroll' | 'scratch' | 'wipe' | 'drink' | 'broom' | 'lap' | 'chin' | 'tea' | 'bread'

export type Pose = {
  eyes: Eyes
  mouth: Mouth
  worried?: boolean
  focused?: boolean // brows drawn down: at the anvil, the bellows, the blade
  ear?: boolean
  look?: number // head turn: -1 toward the forge .. +1 toward the words
  headDy?: number // head bob, pixels down
  dy?: number // whole Scorpheus, pixels down (negative: in the air)
  seated?: boolean // on his stool
  tap?: boolean // right foot up: tapping
  left: LeftArm
  right: RightArm
  tail: 0 | 1
  fire: number // 0 embers .. 4 roaring
  ingot: 'cold' | 'hot' | 'flash' | 'none'
  smokeEvery: number // ticks between puffs, 0 none
  sparksAge?: number // ticks since a hammer blow
  sparkCount?: number
  embers?: boolean // sparks rising from the chimney
  bellows?: boolean // squeezed
  thought?: boolean // a gear turning in a cloud: thinking
  glints?: boolean // far-sight twinkles around the scroll
  bubble?: number // '!' bubble, vertical bounce
  scroll?: boolean
  puff?: number // failure cloud, its age
  stars?: boolean
  fireworks?: number // ticks since the burst
  sweat?: boolean
  // quench
  bucket?: boolean
  steam?: number // its age
  blade?: 'hot' | 'proven' | 'cracked'
  // git
  crate?: 'open' | 'sealed' | 'gone'
  raven?: number // ticks since it took off
  // compaction
  sweep?: number // the broom's swing
  // sleep
  zzz?: boolean
  stool?: boolean
}

// Eyes three pixels tall, as on his Codex sheet: a dark lash line on top, then
// the blue iris with its white glint. Blue on top read as a heavy lid (a tired
// look); white on top melted into the light fur. The brow sits two rows up,
// fur between: brow and lash together read as one black bar.
const EYES: Record<Exclude<Eyes, 'sad'>, readonly string[]> = {
  open: ['kk', 'eh', 'ee'],
  look: ['kk', 'he', 'ee'], // the glint on the other side: a black pupil in a 2 px eye read as a hole
  blink: ['ll', 'll', 'kk'], // the lid down
  happy: ['ll', 'kk', 'll'], // a closed, smiling line
  // no 'wide' eye: a dark pupil read as a hole, a white one as a blind eye; worry
  // and surprise show in the brows, the mouth and the sweat instead
}
const MOUTHS: Record<Mouth, readonly string[]> = {
  none: ['wwww', 'wwww'],
  smile: ['wwww', 'wmmw'],
  open: ['wmmw', 'wrrw'],
  frown: ['wmmw', 'wwww'],
  grit: ['wwww', 'wmmw'],
}

function drawFire(g: Grid, level: number, tick: number) {
  for (let c = 0; c < MOUTH.w; c++) {
    const centre = 1 - Math.abs(c - (MOUTH.w - 1) / 2) / (MOUTH.w / 2)
    const flicker = rand(tick * 31 + c * 7) * 2 - 0.6
    const h = level === 0
      ? (rand(tick * 13 + c) > 0.7 ? 1 : 0)
      : Math.max(1, Math.min(MOUTH.h, Math.round(level * 2 * centre + level + flicker)))
    for (let k = 0; k < h; k++) {
      const f = k / Math.max(1, h - 1)
      const ch = level === 0 ? 'E' : f < 0.25 ? 'Y' : f < 0.55 ? 'f' : f < 0.85 ? 'F' : 'E'
      put(g, MOUTH.x + c, MOUTH.y + MOUTH.h - 1 - k, ch)
    }
  }
}

function drawSmoke(g: Grid, every: number, tick: number) {
  if (every <= 0) return
  const top = FURNACE_AT.y - 1
  for (let born = tick - 10; born <= tick; born++) {
    if (born < 0 || born % every !== 0) continue
    const age = tick - born
    const drift = Math.round(rand(born) * 2 - 1 + age * 0.3)
    const x = FURNACE_AT.x + 6 + drift, y = top - age
    const ch = age < 3 ? 'X' : 'x'
    put(g, x, y, ch)
    put(g, x + 1, y, ch)
    if (age > 1 && age < 7) { put(g, x, y - 1, 'x'); put(g, x - 1, y, 'x') }
    if (age > 3 && age < 8) put(g, x + 2, y - 1, 'x')
  }
}

function drawEmbers(g: Grid, tick: number) {
  for (let born = tick - 7; born <= tick; born++) {
    if (born < 0 || rand(born * 5) < 0.4) continue
    const age = tick - born
    const x = FURNACE_AT.x + 5 + Math.round(rand(born * 3) * 3 + Math.sin(age + born) * 0.8)
    put(g, x, FURNACE_AT.y - 1 - age, age < 3 ? 'Y' : age < 5 ? 'S' : 'E')
  }
}

function drawSteam(g: Grid, age: number, ox: number, oy: number) {
  for (let i = 0; i < 6; i++) {
    const a = age - i * 0.7
    if (a < 0 || a > 8) continue
    const x = ox + Math.round((rand(i * 13) * 2 - 1) * 3 + Math.sin(a + i) * 0.7)
    const y = oy - Math.round(a * 1.2)
    put(g, x, y, a < 3 ? 'w' : a < 6 ? 'X' : 'x')
    if (a < 5) put(g, x + 1, y, 'X')
  }
}

function arm(g: Grid, sx: number, sy: number, px: number, py: number) {
  line(g, sx, sy, px, py, CAPE[1])
  line(g, sx, sy + 1, px, py + 1, CAPE[0])
  put(g, px, py, 'l')
  put(g, px, py + 1, 'g')
}

let HEAD_OF: readonly string[] = HAMMERS.normal // the hammer the model swings, set per frame
let CAPE: readonly [string, string] = ['o', 'O'] // the cape this session wears, set per frame
const dress = (rows: readonly string[]) => rows.map(r => r.replace(/o/g, CAPE[0]).replace(/O/g, CAPE[1]))

function hammer(g: Grid, px: number, py: number, hx: number, hy: number) {
  line(g, px, py, hx + 1, hy + 1, 'r')
  const w = HEAD_OF[0]?.length ?? 3
  stamp(g, hx - Math.floor((w - 3) / 2), hy - (HEAD_OF.length - 2), HEAD_OF)
}

/** Ticks a rare sight lasts: the dragon crosses the panes a pixel every three ticks. */
export const RARE_FOR = rareFor(WINDOW.w)

/** The window: the sky by the hour, the season's weather, the run's weather. */
function drawWindow(g: Grid, w: World, tick: number) {
  const { x, y, w: ww, h } = WINDOW
  drawSky(g, w, tick, { x: x + 1, y: y + 1, w: ww - 2, h: h - 2 }, true)
  const fe = feast(w)
  // the frame, a cross of mullions, the sill
  for (let i = 0; i < ww; i++) { put(g, x + i, y, 'q'); put(g, x + i, y + h - 1, 'Q') }
  for (let j = 0; j < h; j++) { put(g, x, y + j, 'q'); put(g, x + ww - 1, y + j, 'q') }
  for (let j = 1; j < h - 1; j++) put(g, x + ww / 2, y + j, 'q')
  for (let i = 1; i < ww - 1; i++) put(g, x + i, y + Math.floor(h / 2), 'q')
  if (fe === 'halloween') stamp(g, x + ww - 4, y + h - 3, ['.r.', 'FfF', 'fYf']) // the pumpkin on the sill
}

function drawLantern(g: Grid, lit: boolean, tick: number) {
  put(g, LANTERN.x + 1, LANTERN.y, 'a')
  put(g, LANTERN.x + 1, LANTERN.y + 1, 'a')
  const flame = lit ? (rand(tick) > 0.5 ? 'Y' : 'f') : 'a'
  stamp(g, LANTERN.x, LANTERN.y + 2, ['kak', `k${flame}k`, `k${lit ? 'F' : 'a'}k`, 'kkk'])
  if (lit && tick % 4 < 2) { put(g, LANTERN.x - 1, LANTERN.y + 3, 'E'); put(g, LANTERN.x + 3, LANTERN.y + 3, 'E') }
}

function drawRack(g: Grid, w: World) {
  if (w.rack.length === 0 && !w.masterpiece) return
  for (let i = RACK.x; i <= RACK.x + 15; i++) put(g, i, RACK.y, 'R')
  if (w.masterpiece) {
    // the place of honour: a golden greatsword, catching the light
    const x = RACK.x
    ;['Y', 'y', 'y', 'Y', 'y', 'r'].forEach((ch, j) => put(g, x, RACK.y + 1 + j, ch))
    put(g, x - 1, RACK.y + 5, 'y')
    put(g, x + 1, RACK.y + 5, 'y')
    if (w.clock % 20 < 3) put(g, x, RACK.y + 1 + (w.clock % 4), 'w')
  }
  w.rack.slice(-RACK.slots.length).forEach((weapon, i, all) => {
    const x = RACK.slots[i] ?? 0
    const hot = w.forging && i === all.length - 1
    const { shaft, side } = WEAPONS[weapon]
    ;[...shaft].forEach((ch, j) => put(g, x, RACK.y + 1 + j, hot && (ch === 'A' || ch === 'w') ? (j % 2 ? 'H' : 'Y') : ch))
    for (const [dx, dy, ch] of side) put(g, x + dx, RACK.y + 1 + dy, hot && ch === 'A' ? 'H' : ch)
  })
}

function drawBanner(g: Grid, b: NonNullable<World['banner']>, tick: number) {
  const { x, y } = BANNER
  for (let i = -1; i <= 6; i++) put(g, x + i, y, 'r') // the rod
  for (let j = 1; j <= 7; j++) for (let i = 0; i < 6; i++) put(g, x + i, y + j, b.cloth)
  // the emblem: ten bits, mirrored into a 3 x 5 sign
  for (let j = 0; j < 5; j++) {
    for (let i = 0; i < 2; i++) {
      if ((b.bits >> (j * 2 + i)) & 1) { put(g, x + 1 + i, y + 2 + j, b.emblem); put(g, x + 4 - i, y + 2 + j, b.emblem) }
    }
  }
  const flap = Math.floor(tick / 8) % 2 // the swallowtail stirs in the forge's draught
  stamp(g, x, y + 8, [flap ? `${b.cloth}.${b.cloth}${b.cloth}.${b.cloth}` : `${b.cloth}${b.cloth}..${b.cloth}${b.cloth}`])
}

function drawSlate(g: Grid, tick: number) {
  const { x, y } = SLATE_AT
  stamp(g, x, y, ['rrrrrrr', 'rzzzzzr', 'rzwwwzr', 'rzwzwzr', 'rzwwwzr', 'rzzzzzr', 'rrrrrrr'])
  // the chalk clock's hand goes round: waiting for the coal
  const hands: ReadonlyArray<readonly [number, number]> = [[3, 2], [4, 3], [3, 4], [2, 3]]
  const [hx, hy] = hands[Math.floor(tick / 6) % 4] ?? [3, 2]
  put(g, x + 3, y + 3, 'w')
  put(g, x + hx, y + hy, 'Y')
}

/** Who passes by at this tick, if anyone: a raven by day, a moth by night (the cat lives here). */
export function visitorOf(w: World, clock: number): 'raven' | 'moth' | undefined {
  if (clock < VISIT_EVERY || clock % VISIT_EVERY >= VISIT_FOR) return undefined
  return w.hour < 6 || w.hour >= 21 ? 'moth' : 'raven'
}

function drawVisitor(g: Grid, w: World, clock: number) {
  const kind = visitorOf(w, clock)
  if (kind === undefined) return
  const age = clock % VISIT_EVERY
  if (kind === 'raven') {
    const x = SW - Math.round(age * 1.3), y = 1 + Math.round(Math.sin(age / 3))
    stamp(g, x, y, RAVEN[age % RAVEN.length] ?? [])
  } else {
    // a moth around the lantern
    const a = age * 0.6
    put(g, LANTERN.x + 1 + Math.round(Math.cos(a) * 3), LANTERN.y + 4 + Math.round(Math.sin(a) * 2), age % 2 ? 'W' : 'X')
  }
}

/** How far Scorpheus may wander: a few steps toward the furnace, a few toward the banner. */
export const ROAM = { min: -12, max: 5 } as const

/**
 * The station a moment holds Scorpheus to: the bellows for a build, the anvil
 * for the hammer, the blade, the crate, the stool and the duck. Undefined: he
 * is free to move (idle, thinking, reading, sweeping, waiting, napping...).
 */
export function anchorOf(activity: Activity, w: World): number | undefined {
  switch (activity) {
    case 'running': return w.flavor === 'build' ? BELLOWS_SPOT : 0
    case 'quench': case 'ship': case 'failed': return 0
    case 'idle': return w.routine === 'lantern' ? ROAM.min : w.routine === 'lunch' || w.hourglasses > 0 || w.exhausted || w.tea || w.duck ? 0 : undefined
    case 'away': return undefined
    default: return undefined
  }
}

/** A blade held by its hilt at (px, py), pointing up (dir -1) or down (dir 1). */
function blade(g: Grid, px: number, py: number, dir: 1 | -1, state: 'hot' | 'proven' | 'cracked', tick: number) {
  put(g, px - 1, py, 'r')
  put(g, px + 1, py, 'r')
  for (let i = 1; i <= 5; i++) {
    if (state === 'cracked' && i === 5) continue // the tip broke off
    const ch = state === 'hot' ? (i > 3 ? 'Y' : 'H') : state === 'cracked' && i === 3 ? 'E' : i === ((tick >> 1) % 5) + 1 ? 'w' : 'A'
    put(g, px, py + dir * i, ch)
  }
  if (state === 'cracked') put(g, px + 1, py + dir * 4, 'a') // the shard
}

// The apprentices' lane: the floor in front of the forge, feet on the last row.
// Their own yard right of the forge: on the floor in front of him they hid his legs and the anvil.
const LANE = { from: YARD - 8, to: SW - CUB_W, y: SH - 7 }
const CUB_CYCLE = 40 // ticks: 28 walking, then 12 at the hammer
const CUB_WALK = 28

/** Where a cub is at an age: back and forth along the lane at its own pace, pausing to work. */
// Three places in the yard, staggered in depth; a cub roams a few steps around its own.
const SLOTS = [{ x: 0, back: 2 }, { x: 7, back: 0 }, { x: 14, back: 1 }]
const ROAM_STEPS = 3

/** Where a cub is at an age: pacing around its place at its own speed, pausing to work. */
function cubPlace(c: Cub, slot: number): { x: number; y: number; dir: 1 | -1; working: boolean; travel: number } {
  const place = SLOTS[slot % SLOTS.length] ?? { x: 0, back: 0 }
  const pace = 0.3 + rand(c.seed) * 0.3
  const cycles = Math.floor(c.age / CUB_CYCLE)
  const walked = cycles * CUB_WALK + Math.min(c.age % CUB_CYCLE, CUB_WALK)
  const span = ROAM_STEPS * 2
  const d = (walked * pace + rand(c.seed * 3) * span * 2) % (span * 2)
  const out = d < span
  return {
    x: LANE.from + place.x + Math.round((out ? d : span * 2 - d) - ROAM_STEPS),
    y: LANE.y - place.back,
    dir: out ? 1 : -1,
    working: c.age % CUB_CYCLE >= CUB_WALK,
    travel: walked * pace, // pixels walked so far: the legs keep step with the ground
  }
}

// The machine's one cat: a spotted panther, gold with dark rosettes, a green eye.
// It sleeps on the windowsill, warm in the sun; on its way it walks the floor.
const PANTHER = {
  walk: [
    // four steps on long legs, near ones gold, far ones in shadow; the tail sways with them
    ['.11...........', '1111.........1', '14111.......1.', '3311121121111.', '.111211121111.', '.1.5.....1.5..', '1...5...1...5.'],
    ['.11...........', '1111..........', '14111.......11', '3311121121111.', '.111211121111.', '..15......15..', '..51......51..'],
    ['.11...........', '1111.........1', '14111.......1.', '3311121121111.', '.111211121111.', '.5.1.....5.1..', '5...1...5...1.'],
    ['.11...........', '1111..........', '14111.......11', '3311121121111.', '.111211121111.', '..51......51..', '..15......15..'],
  ],
  // standing still: all four legs straight under it, the tail up
  stand: ['.11...........', '1111.........1', '14111.......1.', '3311121121111.', '.111211121111.', '.15.......15..', '.15.......15..'],
  asleep: ['..1.1....', '.1111..1.', '12111.1..', '311121211', '.1121211.'],
}
export const CAT_SILL = { x: 15, y: 4 } // on the windowsill, before the glass
export const CAT_FLOOR_Y = SH - 7

function drawCat(g: Grid, cat: NonNullable<World['cat']>) {
  if (cat.mode === 'roof') {
    // curled up asleep on the sill, a z now and then
    stamp(g, CAT_SILL.x, CAT_SILL.y, PANTHER.asleep)
    if (cat.age % 40 < 12) put(g, CAT_SILL.x + 9, CAT_SILL.y - Math.floor((cat.age % 40) / 4), 'u')
    return
  }
  // its legs move as it moves: a cat arrived beside him stands, it does not tread the air
  const frame = cat.step === undefined ? PANTHER.stand : PANTHER.walk[cat.step % PANTHER.walk.length] ?? PANTHER.stand
  const right = (cat.dir ?? (cat.mode === 'flee' ? 1 : -1)) > 0
  stamp(g, cat.x, CAT_FLOOR_Y, right ? frame.map(r => [...r].reverse().join('')) : frame)
}

function drawGift(g: Grid, gift: NonNullable<World['gift']>) {
  stamp(g, SW - 6, SH - 3, GIFTS[gift.kind])
  if (gift.age % 16 < 3) put(g, SW - 4, SH - 5, 'w') // a glint
}

function drawMessenger(g: Grid, m: NonNullable<World['messenger']>) {
  // from the other forge, right to left over the room, the news in its claws
  const x = SW - Math.round(m.age * 1.6), y = Math.round(Math.sin(m.age / 3))
  stamp(g, x, y, RAVEN[m.age % RAVEN.length] ?? [])
  stamp(g, x + CLAWS.x, y + CLAWS.y, m.kind === 'waiting' ? ['WFW', '.F.'] : ['WWV'])
}

/** The yard by the weather: a puddle after the storm, a snowman in winter, leaves in autumn. */
function drawYard(g: Grid, w: World) {
  if (w.weather === 'storm') stamp(g, YARD + 2, SH - 1, ['.cbbbc.'])
  if (season(w) === 'winter') stamp(g, SW - 5, SH - 8, ['.kFk', '.ww.', 'wwww', '.ww.', 'wwww', 'wwww', '.ww.', '....'].slice(0, 7))
  if (season(w) === 'autumn') {
    ;[[YARD + 1, 'F'], [YARD + 4, 'y'], [YARD + 9, 'f'], [YARD + 12, 'F'], [SW - 3, 'y']].forEach(([x, ch]) => put(g, x as number, SH - 1, ch as string))
  }
}

function drawBanquet(g: Grid, clock: number) {
  // a long table in the yard: a roast in the middle, mugs, the feast's steam
  const x = YARD - 6, y = SH - 6
  stamp(g, x, y, ['......SEHHES......', '.y..ESHHHHSE...y..', 'rrrrrrrrrrrrrrrrrr', '.R..............R.', '.R..............R.'])
  if (clock % 6 < 3) { put(g, x + 8, y - 1, 'X'); put(g, x + 9, y - 2, 'x') }
}

/** A cub in profile at (x, y), facing `dir`, legs in `step` (0..3 trotting, CUB_SIT working). */
function drawWolf(g: Grid, x: number, y: number, dir: 1 | -1, step: number, scarf: string) {
  const body = step % 2 === 1 && step !== CUB_SIT ? [...CUB_BODY.slice(0, 2), ...CUB_TAIL_LOW, ...CUB_BODY.slice(4)] : CUB_BODY
  const rows = [...body, ...(CUB_LEGS[step] ?? [])].map(r => {
    const row = r.replace(/C/g, scarf)
    return dir > 0 ? row : [...row].reverse().join('')
  })
  stamp(g, x, y, rows)
}

function drawCubs(g: Grid, cubs: readonly Cub[], puddle = false) {
  // three at most in the yard, each at its place; the farther back drawn first
  const live = cubs.filter(c => c.leaving === undefined).slice(0, SLOTS.length)
  const going = cubs.filter(c => c.leaving !== undefined).slice(0, 2)
  const pack = [...live.map((c, i) => ({ c, slot: i })), ...going.map(c => ({ c, slot: c.seed % SLOTS.length }))]
  pack.sort((a, b) => (SLOTS[b.slot]?.back ?? 0) - (SLOTS[a.slot]?.back ?? 0))
  pack.forEach(({ c, slot }) => {
    const at = cubPlace(c, slot)
    let x = at.x, dir = at.dir, working = at.working && c.leaving === undefined, travel = at.travel
    if (c.leaving !== undefined) {
      // its work is done: it trots out to the right, the scroll in its mouth
      x = at.x + Math.round(c.leaving * 1.3)
      dir = 1
      working = false
      travel = c.leaving * 1.3
    }
    const y = at.y - (puddle && !working && c.age % 6 < 2 ? 1 : 0) // splashing in the puddle
    // a step for each pixel walked, so a paw set down stays where it is
    const step = working ? CUB_SIT : Math.floor(travel) % 4
    drawWolf(g, x, y, dir, step, c.colour)
    const nose = dir > 0 ? x + CUB_W - 1 : x
    if (working) {
      // sitting, its little hammer goes up and down, sparks in its colour
      const up = c.age % 4 < 2
      const hx = dir > 0 ? x + CUB_W : x - 1
      put(g, hx, y + (up ? 1 : 3), 'A')
      put(g, hx, y + (up ? 2 : 4), 'r')
      if (!up) { put(g, hx + dir, y + 4, c.colour); put(g, hx + 2 * dir, y + 3, 'Y') }
    }
    if (c.leaving !== undefined) stamp(g, dir > 0 ? nose + 1 : nose - 2, y + 3, ['WV']) // the scroll
    if (c.age < 8 && c.leaving === undefined) stamp(g, x + 3, y + 1, PUFF[Math.min(2, c.age >> 2)] ?? [])
  })
}

function drawHourglasses(g: Grid, n: number, clock: number) {
  for (let i = 0; i < Math.min(n, HOURGLASS_X.length); i++) {
    const x = HOURGLASS_X[i] ?? 0, y = 18
    const phase = (clock + i * 13) % 40 // the sand runs for 6 s, then he turns it
    const top = phase < 14 ? 's' : 'k'
    const bottom = phase > 26 ? 's' : 'k'
    stamp(g, x, y, ['aaa', `k${top}k`, '.k.', 'k.k', `k${bottom}k`, 'aaa'])
    if (phase < 38) put(g, x + 1, y + 2 + (phase % 2), 's') // the falling grain
    if (phase >= 14) put(g, x + 1, y + 4, 's')
  }
}

function drawBell(g: Grid, age: number) {
  const swing = age < 18 ? [0, 1, 0, -1][age % 4] ?? 0 : 0
  put(g, BELL_AT.x + 2, BELL_AT.y, 'r')
  put(g, BELL_AT.x + 2, BELL_AT.y + 1, 'r')
  stamp(g, BELL_AT.x + swing, BELL_AT.y + 2, ['.yyy.', 'yyyyy', 'yyyyy', 'kkkkk'])
  put(g, BELL_AT.x + 2 - swing, BELL_AT.y + 6, 'k') // the clapper
  if (age < 18 && age % 4 < 2) {
    put(g, BELL_AT.x - 2, BELL_AT.y + 3, 's'); put(g, BELL_AT.x - 3, BELL_AT.y + 2, 's')
    put(g, BELL_AT.x + 6, BELL_AT.y + 3, 's'); put(g, BELL_AT.x + 7, BELL_AT.y + 2, 's')
  }
}

/**
 * The scene for a frame. `actorOnly` draws Scorpheus alone, without the room or
 * the moment's effects: how a guest is drawn into another forge.
 */
export function compose(p: Pose, world: World, tick: number, actorOnly = false): Grid {
  const pal = SPARKS[world.flavor]
  const g: Grid = Array.from({ length: SH }, () => Array<string>(SW).fill('.'))
  const clock = world.clock
  HEAD_OF = HAMMERS[world.hammer]
  CAPE = world.cape
  const night = world.hour < 6 || world.hour >= 21

  if (!actorOnly) {
  // the room
  drawWindow(g, world, clock)
  drawLantern(g, world.lanternLit, clock)
  drawRack(g, world)
  if (world.banner) drawBanner(g, world.banner, clock)

  // the forge
  stamp(g, FURNACE_AT.x, FURNACE_AT.y, FURNACE)
  if (season(world) === 'winter') for (let i = 2; i <= 11; i++) put(g, FURNACE_AT.x + i, FURNACE_AT.y + 4, 'w')
  const logs = world.fuel5h === undefined ? 8 : Math.round(8 * (1 - Math.min(100, world.fuel5h) / 100))
  for (let i = 0; i < logs; i++) put(g, MOUTH.x + i, LOGS_Y, i % 3 === 1 ? 'r' : 'R')
  const pile = world.fuel7d === undefined ? WOODPILE.length : Math.round(WOODPILE.length * (1 - Math.min(100, world.fuel7d) / 100))
  WOODPILE.slice(0, pile).forEach(([wx, wy], i) => put(g, wx, wy, i % 2 ? 'V' : 'r'))
  drawFire(g, world.exhausted ? 0 : p.fire, tick)
  drawSmoke(g, world.exhausted ? 0 : p.smokeEvery, tick)
  if (p.embers) drawEmbers(g, tick)
  if (night && p.fire > 0) {
    // at night the hearth lights the stones around it
    for (let j = MOUTH.y - 1; j <= LOGS_Y; j += 2) if (rand(tick * 3 + j) > 0.5) { put(g, MOUTH.x - 2, j, 'E'); put(g, MOUTH.x + MOUTH.w + 1, j, 'E') }
  }
  // the bellows: squeezed, it blows a puff into the hearth
  // open: lever up, folds spread; squeezed: lid down a pixel, folds pressed, base still
  stamp(g, BELLOWS.x, BELLOWS.y, p.bellows
    ? ['......', '......', '.....r', '.kRRr.', 'kRRRRk', 'kkkkkk']
    : ['.....r', '....r.', '.kRRr.', 'kRrrRk', 'kRrrRk', 'kkkkkk'])
  put(g, BELLOWS.x - 1, BELLOWS.y + 3, 'a') // the nozzle
  if (p.bellows) { put(g, MOUTH.x + MOUTH.w - 1, BELLOWS.y + 3, 'X'); put(g, MOUTH.x + MOUTH.w - 2, BELLOWS.y + 2, 'x') }
  stamp(g, ANVIL_AT.x, ANVIL_AT.y, ANVIL)
  if (p.ingot !== 'none') {
    const ingot = p.ingot === 'flash' ? 'Y' : p.ingot === 'hot' ? 'H' : 'E'
    for (let i = 0; i < INGOT.w; i++) put(g, INGOT.x + i, INGOT.y, i === 0 || i === INGOT.w - 1 ? (ingot === 'Y' ? 'H' : 'E') : ingot)
  }
  const scraps = p.sweep !== undefined ? 0 : Math.round(Math.max(0, world.ctx - 50) / 50 * SCRAPS.length)
  SCRAPS.slice(0, scraps).forEach(([sx, sy, ch]) => put(g, sx, sy, ch))
  const coins = world.usd === undefined || world.usd <= 0 ? 0 : Math.min(COINS.length, 1 + Math.floor(Math.log2(1 + world.usd) * 2))
  COINS.slice(0, coins).forEach(([cx, cy], i) => put(g, cx, cy, (i + Math.floor(clock / 10)) % 7 === 0 ? 'Y' : i % 2 ? 's' : 'y'))
  if (world.hourglasses > 0) drawHourglasses(g, world.hourglasses, clock)
  if (world.exhausted && world.hourglasses === 0) drawSlate(g, clock)
  if (world.duck) stamp(g, DUCK_AT.x, DUCK_AT.y, DUCK)
  if (p.crate && p.crate !== 'gone') {
    stamp(g, CRATE_AT.x, CRATE_AT.y, CRATE)
    if (p.crate === 'sealed') { put(g, CRATE_AT.x + 3, CRATE_AT.y + 2, 'y'); put(g, CRATE_AT.x + 4, CRATE_AT.y + 2, 'y') }
  }
  }

  // Scorpheus, unless he is away visiting
  if (!world.absent) {
  const walking = world.walking
  // his stride follows the ground, not the clock: a step each pixel walked, so a planted boot
  // stays put while he moves over it; the head nods as each boot lands, the tail swings
  const stride = ((world.x * (world.facing < 0 ? -1 : 1)) % 4 + 4) % 4
  if (walking) p = { ...p, look: world.facing, headDy: stride % 2 === 0 ? 1 : 0, tail: stride < 2 ? 0 : 1 }
  const x = C.x + world.x, y = C.y + (p.dy ?? 0) + (p.seated ? 2 : 0)
  if (p.stool) stamp(g, x + 1, C.y + 23, ['RRRRRRRRRRRRRR', '.R..........R.', '.R..........R.'])
  stamp(g, x + 13, y + 13, TAIL[p.tail] ?? [])
  if (p.seated) {
    stamp(g, x, y + 14, dress(BODY_SEATED))
    stamp(g, x + 2, C.y + 24, ['kaaaak'])
    stamp(g, x + 8, C.y + 24 - (p.tap ? 1 : 0), ['kaaaak'])
    stamp(g, x + 2, C.y + 25, ['kkkkk'])
    stamp(g, x + 9, C.y + 25 - (p.tap ? 1 : 0), ['kkkkk'])
  } else {
    stamp(g, x, y + 14, dress(BODY))
    if (world.accessory === 'tablier') {
      // the leather apron over his chest
      stamp(g, x + 4, y + 15, ['.rrrrrr.', '.rRrrRr.', '.rrrrrr.', '..rrrr..'])
    }
    if (walking) {
      // a side-step in four: apart, the trailing boot lifts and closes, together, the leading one lifts and opens
      for (let i = 1; i <= 14; i++) { erase(g, x + i, y + 24); erase(g, x + i, y + 25) }
      const leftLifted = stride === (world.facing < 0 ? 3 : 1), rightLifted = stride === (world.facing < 0 ? 1 : 3)
      stamp(g, x + ([1, 2, 3, 2][stride] ?? 2), y + 24 - (leftLifted ? 1 : 0), ['kaaaak', 'kkkkk'])
      stamp(g, x + ([9, 8, 7, 8][stride] ?? 8), y + 24 - (rightLifted ? 1 : 0), ['kaaaak', '.kkkkk'])
    }
    if (p.tap) {
      for (let i = 8; i <= 13; i++) { erase(g, x + i, y + 24); erase(g, x + i, y + 25) }
      stamp(g, x + 8, y + 23, ['kaaaak', '.kkkkk'])
    }
  }
  const head = SKULL.map(r => r.split(''))
  // a 16 px face turns by one pixel at most: further, an eye runs into the outline
  const dx = Math.max(-1, Math.min(1, p.look ?? 0))
  // a feature lands only on fur, never on the outline: turning slides the face inside the skull
  const feature = (fx: number, fy: number, ch: string) => {
    const row = head[fy], under = row?.[fx + dx]
    if (row && ch !== '.' && under !== undefined && under !== 'k' && under !== '.') row[fx + dx] = ch
  }
  const paste = (fx: number, fy: number, rows: readonly string[]) =>
    rows.forEach((row, j) => [...row].forEach((ch, i) => feature(fx + i, fy + j, ch)))
  for (const [mx, my, ch] of MARKS) feature(mx, my, ch)
  if (p.worried) {
    paste(2, 5, ['..k', 'kkl'])
    paste(10, 5, ['k..', 'lkk'])
  } else if (p.focused) {
    // brows drawn down toward the muzzle: concentration, with the eyes left whole
    paste(2, 5, ['k..', 'lkk'])
    paste(10, 5, ['..k', 'kkl'])
  } else {
    paste(2, 6, ['kkk'])
    paste(10, 6, ['kkk'])
  }
  paste(2, 8, p.eyes === 'sad' ? ['ll', 'kk', 'cl'] : EYES[p.eyes])
  paste(10, 8, p.eyes === 'sad' ? ['ll', 'kk', 'lc'] : EYES[p.eyes])
  paste(6, 12, MOUTHS[p.mouth])
  if (p.ear) {
    const r0 = head[0], r1 = head[1], r2 = head[2]
    if (r0 && r1 && r2) { r0[1] = '.'; r1[0] = '.'; r1[1] = 'k'; r1[2] = 'k'; r2[1] = 'k'; r2[2] = 'w' }
  }
  const hy = y + (p.headDy ?? 0)
  stamp(g, x, hy, head.map(r => r.join('')))
  const fe = feast(world)
  if (world.accessory === 'lunettes') {
    // goggles pushed up on the brow: a lens each side, the strap between
    stamp(g, x + 1, hy + 6, ['RAcA'])
    stamp(g, x + 11, hy + 6, ['AcAR'])
    for (let i = 5; i <= 10; i++) put(g, x + i, hy + 6, 'R')
  }
  if (world.accessory === 'bandeau') {
    for (let i = 1; i <= 14; i++) put(g, x + i, hy + 6, CAPE[1])
    put(g, x + 15, hy + 7, CAPE[1]); put(g, x + 15, hy + 8, CAPE[0]) // its knot's tails
  }
  if (fe === 'birthday') stamp(g, x + 5, hy - 4, HATS.party)
  if (fe === 'christmas') stamp(g, x + 5, hy - 4, HATS.santa)

  // his left arm (screen left): hammer, lever, blade, seal, broom
  const ls = { x: x + 1, y: y + 15 }
  const left = walking ? 'rest' : p.left
  const right = walking ? 'rest' : p.right
  switch (left) {
    case 'rest':
      arm(g, ls.x, ls.y, x, y + 18)
      hammer(g, x, y + 19, x - 1, y + 21)
      break
    case 'raise':
      arm(g, ls.x, ls.y, x - 1, y + 11)
      hammer(g, x - 1, y + 10, x - 3, y + 6)
      break
    case 'strike':
      arm(g, ls.x, ls.y, x - 3, y + 15)
      hammer(g, x - 4, y + 16, IMPACT.x - 1, IMPACT.y - 1 + (p.dy ?? 0))
      break
    case 'drop':
      arm(g, ls.x, ls.y, x, y + 19)
      stamp(g, x - 5, C.y + 24, ['r...', '.rAA', '..aa']) // fallen at his feet
      break
    case 'scroll':
      arm(g, ls.x, ls.y, x + 4, y + 18)
      break
    case 'pull': {
      // the bellows' lever: his paw rides it down, the bellows squeeze
      const lx = BELLOWS.x + 5, ly = BELLOWS.y + (p.bellows ? 2 : 0) // the lever's tip
      arm(g, ls.x, ls.y, lx + 1, ly)
      stamp(g, IMPACT.x - 1, IMPACT.y, ['.r.AA', 'rr.aa']) // the hammer laid on the anvil
      break
    }
    case 'blade':
      arm(g, ls.x, ls.y, x - 2, y + 11)
      blade(g, x - 2, y + 10, -1, p.blade ?? 'hot', tick)
      break
    case 'dip':
      arm(g, ls.x, ls.y, BUCKET_AT.x + 4, BUCKET_AT.y - 6)
      blade(g, BUCKET_AT.x + 4, BUCKET_AT.y - 6, 1, p.blade ?? 'hot', tick)
      break
    case 'seal':
      arm(g, ls.x, ls.y, CRATE_AT.x + 5, CRATE_AT.y - 1 + (p.crate === 'sealed' ? 1 : 0))
      break
    case 'broom':
      arm(g, ls.x, ls.y, x + 2, y + 17)
      break
    case 'lap':
      arm(g, ls.x, ls.y, x + 4, y + 19)
      break
  }

  // his right arm
  const rs = { x: x + 14, y: y + 15 }
  switch (right) {
    case 'rest': arm(g, rs.x, rs.y, x + 15, y + 18); break
    case 'wave1': arm(g, rs.x, rs.y, x + 17, y + 9); break
    case 'wave2': arm(g, rs.x, rs.y, x + 18, y + 11); break
    case 'up': arm(g, rs.x, rs.y, x + 17, y + 8); break
    case 'scroll': arm(g, rs.x, rs.y, x + 11, y + 18); break
    case 'scratch': arm(g, rs.x, rs.y, x + 15, y + 6 + (tick % 2)); break
    case 'wipe': arm(g, rs.x, rs.y, x + 12 - (tick % 2), hy + 7); break
    case 'drink':
      arm(g, rs.x, rs.y, x + 11, hy + 13)
      stamp(g, x + 10, hy + 10, ['A', 'a', 'a']) // the flask, tipped to the muzzle
      break
    case 'broom': {
      // both paws on the handle, the bristles sweep left and right
      const bx = x + 2 + (p.sweep ?? 0)
      arm(g, rs.x, rs.y, x + 6, y + 19)
      line(g, x + 6, y + 18, bx, C.y + 25, 'r')
      stamp(g, bx - 1, C.y + 25, ['yVy'])
      break
    }
    case 'lap': arm(g, rs.x, rs.y, x + 11, y + 19); break
    case 'chin': arm(g, rs.x, rs.y, x + 10, hy + 14); break
    case 'bread':
      arm(g, rs.x, rs.y, x + 11, hy + 13)
      stamp(g, x + 9, hy + 12, ['VWV', 'rVr']) // his bread, at the muzzle
      break
    case 'tea': {
      arm(g, rs.x, rs.y, x + 12, y + 17)
      stamp(g, x + 11, y + 15, ['WW', 'WV']) // the cup
      const a = tick % 6
      put(g, x + 11 + (a % 2), y + 14 - Math.floor(a / 2), a < 3 ? 'X' : 'x') // its steam
      break
    }
  }
  if (p.thought) {
    put(g, x + 15, hy + 3, 'w')
    put(g, x + 16, hy + 1, 'w')
    stamp(g, x + 17, Math.max(0, hy - 6), THOUGHT[Math.floor(tick / 3) % 2] ?? [])
  }
  if (p.scroll) stamp(g, x + 4, y + 16, SCROLL)
  if (p.glints) {
    for (let i = 0; i < 3; i++) {
      const a = (tick + i * 5) % 9
      const gx = x + 3 + Math.round(rand(i * 7 + Math.floor((tick + i * 5) / 9)) * 9)
      put(g, gx, y + 15 - Math.floor(a / 2), a < 4 ? pal[0] : pal[1])
    }
  }

  if (p.bubble !== undefined) stamp(g, x + 17, Math.max(0, y - 6 + p.bubble), BUBBLE)
  else if (world.speaking && !p.thought) stamp(g, x + 17, Math.max(0, hy - 5), SAYING)

  // the moment's effects
  if (!actorOnly) {
  if (p.bucket) stamp(g, BUCKET_AT.x, BUCKET_AT.y, BUCKET)
  if (p.steam !== undefined) drawSteam(g, p.steam, BUCKET_AT.x + 4, BUCKET_AT.y - 1)
  if (p.raven !== undefined && p.raven < 16) {
    const rx = CRATE_AT.x + 2 - Math.round(p.raven * 1.4), ry = CRATE_AT.y - 4 - Math.round(p.raven * 1.3)
    stamp(g, rx, ry, RAVEN[p.raven % RAVEN.length] ?? [])
    stamp(g, rx + CLAWS.x, ry + CLAWS.y, ['rRr', 'kkk']) // the crate in its claws
  }
  if (p.sweep !== undefined) {
    for (let i = 0; i < 4; i++) {
      const a = (tick + i * 3) % 8
      put(g, x - 2 + i * 5 + Math.round(a * 0.4), C.y + 25 - Math.floor(a / 2), a < 4 ? 'X' : 'x')
    }
  }
  // over his head, clear of the z's on his right: the narrowest scene (to x 54) holds it whole
  if (p.zzz && world.dream) drawDream(g, x + 2, Math.max(0, hy - 9), world.dream)
  if (p.zzz) {
    for (let i = 0; i < 2; i++) {
      const a = (tick + i * 10) % 20
      stamp(g, x + 15 + Math.floor(a / 4), hy + 1 - Math.floor(a / 3), i ? Z_SMALL : Z_BIG)
    }
  }
  if (p.sparksAge !== undefined && p.sparksAge < 6) {
    drawSparks(g, p.sparksAge, Math.floor(tick / 8), p.sparkCount ?? 12, IMPACT.x + 1, IMPACT.y, pal)
  }
  if (p.fireworks !== undefined) {
    drawSparks(g, p.fireworks % 7, Math.floor(p.fireworks / 7) + 99, 16, IMPACT.x + 1, IMPACT.y - 2, SPARKS.forge)
  }
  if (p.puff !== undefined) stamp(g, IMPACT.x - 2, IMPACT.y - 4 - Math.min(2, p.puff >> 1), PUFF[Math.min(2, p.puff >> 1)] ?? [])
  if (p.stars) {
    for (let i = 0; i < 6; i++) {
      const sx = Math.floor(rand(Math.floor(tick / 2) * 11 + i) * SW)
      const sy = Math.floor(rand(Math.floor(tick / 2) * 23 + i) * 12)
      put(g, sx, sy, i % 2 ? 's' : 'Y')
      if (i % 3 === 0) { put(g, sx - 1, sy, 's'); put(g, sx + 1, sy, 's'); put(g, sx, sy - 1, 's'); put(g, sx, sy + 1, 's') }
    }
  }
  if (p.sweat) {
    put(g, x + 16, hy + 3 + (tick % 6 > 2 ? 1 : 0), 'c')
    put(g, x + 16, hy + 4 + (tick % 6 > 2 ? 1 : 0), 'c')
  }
  }
  }
  if (actorOnly) return g

  drawYard(g, world)
  if (world.cat) drawCat(g, world.cat)
  if (world.gift) drawGift(g, world.gift)

  // guests from other sessions: the same Scorpheus, each in his own look, in the yard
  for (const v of world.guests.slice(0, 2)) {
    const gw: World = {
      ...world, x: v.x, walking: v.walking, facing: v.facing, cape: v.cape, accessory: v.accessory,
      absent: false, guests: [], speaking: false, hammer: 'normal', routine: undefined, dream: undefined,
      hourglasses: 0, exhausted: false, tea: false, duck: false, pressure: 0, personality: 'calme',
    }
    const guest = compose(poseOf(v.waving ? 'waving' : 'idle', v.tick, gw), gw, v.tick, true)
    guest.forEach((row, gy) => row.forEach((ch, gx) => { if (ch !== '.') put(g, gx, gy, ch) }))
  }
  if (world.banquet) drawBanquet(g, clock)
  if (world.messenger) drawMessenger(g, world.messenger)

  // what goes on above whatever he does
  drawVisitor(g, world, clock)
  if (world.bellAge !== undefined && world.bellAge < 40) drawBell(g, world.bellAge)
  drawCubs(g, world.apprentices, world.weather === 'storm')
  return g
}

// --- the moments, one entry per 150 ms tick (t counts from the moment's start) --

const at = <T>(list: readonly T[], i: number): T => list[((i % list.length) + list.length) % list.length] as T
const sway = (t: number, every: number) => (Math.floor(t / every) % 2) as 0 | 1

// Idle runs a 24 s cycle: he watches the fire, glances at the words, scratches
// an ear and stretches, breathing and wagging all along. With jobs in the
// background he sits on his stool and taps his foot, an eye on the hourglass.
function idle(t: number, w: World): Pose {
  const c = t % 160
  const within = (a: number, b: number) => c >= a && c < b
  const blink = c === 14 || c === 24 || c === 40 || c === 42 || c === 104
  const base: Pose = {
    eyes: blink ? 'blink' : 'open',
    mouth: 'none',
    look: within(16, 32) || within(88, 102) ? -1 : within(44, 54) ? 1 : 0,
    ear: within(52, 54),
    headDy: c % 16 < 8 ? 0 : 1,
    left: 'rest', right: 'rest',
    tail: sway(t, within(90, 100) ? 2 : 6),
    fire: 1, ingot: 'cold', smokeEvery: 5,
  }
  if (w.hourglasses > 0) {
    return { ...base, seated: true, stool: true, tap: t % 4 < 2, look: c % 40 < 20 ? -1 : 0, left: 'lap', right: 'lap', headDy: 0 }
  }
  if (w.exhausted) {
    // no coal left: he sits by the cold forge, an eye on the slate
    return { ...base, seated: true, stool: true, left: 'lap', right: 'lap', look: -1, eyes: blink ? 'blink' : 'open', worried: true, fire: 0, smokeEvery: 0, headDy: 0 }
  }
  if (w.duck) {
    // three failures running: he explains the problem to the duck
    return { ...base, look: -1, mouth: c % 6 < 3 ? 'open' : 'none', eyes: blink ? 'blink' : 'open', worried: true }
  }
  if (w.tea && c % 160 >= 20 && c % 160 < 120) {
    return { ...base, seated: true, stool: true, left: 'lap', right: 'tea', eyes: c % 30 < 4 ? 'happy' : base.eyes, headDy: 0 }
  }
  if (w.routine === 'lantern') {
    // dusk: he reaches up to light the lantern
    return { ...base, left: 'raise', right: 'up', look: -1, eyes: 'open', headDy: 0 }
  }
  if (w.routine === 'lunch') {
    return { ...base, seated: true, stool: true, left: 'lap', right: 'bread', mouth: c % 8 < 4 ? 'open' : 'none', headDy: 0 }
  }
  if (w.routine === 'coffee') {
    return { ...base, right: 'tea', eyes: c % 30 < 4 ? 'happy' : base.eyes }
  }
  if (w.hour < 5 && c % 48 >= 40) {
    // the small hours: he yawns more and more often
    return { ...base, left: 'raise', right: 'up', eyes: 'blink', mouth: 'open', dy: -1, headDy: 0 }
  }
  if (within(64, 76)) Object.assign(base, { right: 'scratch', eyes: 'happy', ear: c % 2 === 0 })
  if (within(120, 128)) Object.assign(base, { left: 'raise', right: 'up', eyes: 'blink', mouth: 'open', dy: -1, headDy: 0 })
  // his temper shows when nothing else does
  if (w.personality === 'jovial' && base.mouth === 'none' && base.eyes === 'open') base.mouth = 'smile'
  if (w.personality === 'grognon' && base.eyes === 'open') base.focused = true
  if (w.personality === 'reveur' && base.look === 0 && c % 40 < 30) base.look = -1 // toward the window
  if (w.personality === 'applique' && base.look === 0 && c % 40 < 20) Object.assign(base, { look: -1, focused: true })
  if (w.pressure >= 85) Object.assign(base, { worried: true, sweat: true })
  if (w.pressure >= 95) Object.assign(base, { mouth: 'frown' })
  return base
}

// At work: the hammer, or for a build the bellows. A long turn tires him:
// past 5 minutes he wipes his brow now and then, past 15 he drinks from his flask.
function running(t: number, w: World): Pose {
  const c = t % 8
  const strike = c >= 3 && c <= 5
  let pose: Pose
  if (w.flavor === 'build') {
    const squeeze = t % 6 < 3
    const heat = Math.min(4, 2 + w.effort + Math.floor(t / 40)) // the furnace climbs as the build goes on
    pose = {
      eyes: t % 30 === 0 ? 'blink' : 'open', focused: true, mouth: squeeze ? 'grit' : 'none', look: -1,
      headDy: squeeze ? 1 : 0,
      left: 'pull', right: 'rest', bellows: squeeze,
      tail: sway(t, 3),
      fire: squeeze ? heat : heat - 1, ingot: 'hot', smokeEvery: 1, embers: true,
    }
  } else {
    pose = {
      eyes: c === 7 ? 'blink' : 'open', focused: true,
      mouth: c === 3 ? 'grit' : 'none',
      look: -1,
      headDy: strike ? 1 : 0,
      left: strike ? 'strike' : 'raise', right: 'rest',
      tail: sway(t, 4),
      fire: Math.min(4, 2 + w.effort), ingot: c === 3 ? 'flash' : 'hot', smokeEvery: 2,
      sparksAge: c >= 3 ? c - 3 : undefined,
      sparkCount: w.hammer === 'sledge' ? 16 : w.hammer === 'small' ? 8 : 12,
    }
  }
  const long = w.turnTicks
  if (long > 6000 && long % 64 < 10) {
    return { ...pose, right: 'drink', left: pose.left === 'pull' ? 'pull' : 'rest', eyes: 'blink', look: 0, sparksAge: undefined, headDy: 0 }
  }
  if (long > 2000 && long % 48 < 6) return { ...pose, right: 'wipe', sweat: true }
  return pose
}

// The model is thinking: paw under the chin, a gear turning in a cloud.
function thinking(t: number, w: World): Pose {
  const c = t % 30
  return {
    eyes: c === 11 ? 'blink' : c % 10 < 6 ? 'look' : 'open',
    mouth: 'none', look: c % 10 < 6 ? 1 : 0, headDy: c % 20 < 10 ? 0 : 1,
    left: 'rest', right: 'chin', thought: true,
    tail: sway(t, 6),
    fire: Math.min(3, 1 + w.effort), ingot: 'hot', smokeEvery: 4,
  }
}

function review(t: number, w: World): Pose {
  const c = t % 24
  return {
    glints: w.flavor === 'web' || w.flavor === 'mcp' || w.flavor === 'agent',
    eyes: c === 13 ? 'blink' : c % 8 < 4 ? 'look' : 'open',
    mouth: 'none', headDy: 1,
    left: 'scroll', right: 'scroll', scroll: true,
    tail: sway(t, 6),
    fire: 1, ingot: 'hot', smokeEvery: 4,
  }
}

// Waiting on the person; past 30 s he taps his foot.
function waiting(t: number): Pose {
  return {
    eyes: t % 20 === 9 ? 'blink' : 'open',
    mouth: 'none', worried: true,
    ear: t % 9 < 2,
    look: t % 12 < 6 ? 1 : 0,
    tap: t > 200 && t % 4 < 2,
    left: 'rest', right: 'up',
    tail: sway(t, 3),
    fire: 1, ingot: 'hot', smokeEvery: 5,
    bubble: at([0, 0, 1, 1], t),
  }
}

function failed(t: number): Pose {
  return {
    eyes: 'sad', mouth: 'frown', worried: true, headDy: 1,
    left: t < 1 ? 'strike' : 'drop', right: 'rest',
    tail: 0, fire: 0, ingot: 'cold', smokeEvery: 0,
    puff: t < 7 ? t : undefined,
  }
}

function jumping(t: number): Pose {
  const c = t % 8
  return {
    eyes: 'happy', mouth: c < 4 ? 'open' : 'smile',
    dy: at([0, -1, -3, -4, -3, -1, 0, 0], c),
    left: 'raise', right: 'up',
    tail: (c % 2) as 0 | 1,
    fire: 3, ingot: 'flash', smokeEvery: 2,
    stars: true, fireworks: t,
  }
}

function waving(t: number): Pose {
  return {
    eyes: 'happy', mouth: 'smile',
    left: 'rest', right: Math.floor(t / 2) % 2 ? 'wave1' : 'wave2',
    tail: sway(t, 2),
    fire: Math.min(2, Math.floor(t / 4)), // the forge lights up
    ingot: 'cold', smokeEvery: t > 6 ? 4 : 0,
    sparksAge: t >= 4 && t < 10 ? t - 4 : undefined,
  }
}

// The tests are over: the blade goes into the bucket, steams, and holds or cracks.
function quench(t: number, w: World): Pose {
  const pass = w.outcome !== 'fail'
  const base: Pose = {
    eyes: 'open', focused: true, mouth: 'none', look: -1,
    left: 'blade', right: 'rest', blade: 'hot',
    tail: sway(t, 4), fire: 2, ingot: 'none', smokeEvery: 3, bucket: true,
  }
  if (t < 3) return base
  if (t < 9) return { ...base, left: 'dip', eyes: 'blink', steam: t - 3 }
  return pass
    ? { ...base, blade: 'proven', eyes: 'happy', mouth: 'smile', look: 0, right: 'up', steam: t - 3, stars: t < 14 }
    : { ...base, blade: 'cracked', eyes: 'sad', mouth: 'frown', worried: true, headDy: 1, steam: t - 3, puff: t - 9 }
}

// git: he seals the crate with gold wax; on a push the raven carries it off.
function ship(t: number, w: World): Pose {
  const base: Pose = {
    eyes: 'open', focused: true, mouth: 'none', look: -1,
    left: 'seal', right: 'rest', crate: t < 3 ? 'open' : 'sealed',
    tail: sway(t, 4), fire: 2, ingot: 'none', smokeEvery: 4,
    sparksAge: t >= 3 && t < 7 ? t - 3 : undefined, sparkCount: 6,
  }
  if (w.ship !== 'push' || t < 6) return base
  return { ...base, crate: 'gone', raven: t - 6, left: 'rest', right: Math.floor(t / 2) % 2 ? 'wave1' : 'wave2', eyes: 'happy', mouth: 'smile', look: 0, sparksAge: undefined }
}

// The context is compacting: he sweeps the workshop clean.
function sweep(t: number): Pose {
  return {
    eyes: t % 20 === 7 ? 'blink' : 'open', mouth: 'none', look: -1,
    left: 'broom', right: 'broom', sweep: at([-3, -2, 0, 2, 3, 2, 0, -2], t),
    tail: sway(t, 3), fire: 1, ingot: 'cold', smokeEvery: 5,
  }
}

// Ten minutes with nothing to do: he dozes on his stool, the fire down to embers.
function sleep(t: number): Pose {
  return {
    eyes: 'blink', mouth: 'none', headDy: t % 24 < 12 ? 1 : 0,
    seated: true, stool: true,
    left: 'lap', right: 'lap', tail: 0,
    fire: 0, ingot: 'cold', smokeEvery: 0, zzz: true,
  }
}

// A message wakes him with a start.
function waking(t: number): Pose {
  return {
    eyes: 'open', mouth: t < 3 ? 'open' : 'none', ear: true,
    dy: at([-3, -2, -1, 0], Math.min(t, 3)),
    left: 'rest', right: 'up', tail: sway(t, 1),
    fire: Math.min(2, t), ingot: 'cold', smokeEvery: 3,
    bubble: t < 4 ? 0 : undefined,
  }
}

export function poseOf(activity: Activity, tick: number, world: World = CALM): Pose {
  switch (activity) {
    case 'idle': return idle(tick, world)
    case 'running': return running(tick, world)
    case 'thinking': return thinking(tick, world)
    case 'review': return review(tick, world)
    case 'waiting': return waiting(tick)
    case 'failed': return failed(tick)
    case 'jumping': return jumping(tick)
    case 'waving': return waving(tick)
    case 'quench': return quench(tick, world)
    case 'ship': return ship(tick, world)
    case 'sweep': return sweep(tick)
    case 'sleep': return sleep(tick)
    case 'waking': return waking(tick)
    case 'away': return idle(tick, world) // not drawn: his forge stands empty
  }
}

// --- to the terminal ---------------------------------------------------------------

export function frame(crop: Crop, activity: Activity, tick: number, world: World = CALM): Uint32Array {
  const [x0, x1] = CROPS[crop]
  return toCells(compose(poseOf(activity, tick, world), world, tick), PALETTE, x0, x1)
}

/** The forge as a theme: the engine's only way in. */
export const forge: Theme = {
  id: 'forge',
  title: 'La forge de Scorpheus',
  width: SW,
  rows: ROWS,
  crops: CROPS,
  palette: PALETTE,
  frame,
  anchorOf,
  stage: { roam: ROAM, idleSpots: [-12, -8, -4, 0, 0, 3, 5], offstage: OFFSTAGE, guestSpots: [GUEST_SPOT, GUEST_SPOT - 14] },
  // the cat jumps down from the sill at 14, goes off past the yard, and walks a little behind him
  pet: { gate: 14, away: SW + 12, follow: 21 },
  sky: crop => crop !== 'solo', // the narrowest scene has no window
  rareFor: RARE_FOR,
  looks: { variants: BASE_VARIANTS, accessories: ACCESSORIES },
  names: SMITHS,
  helperNames: ['Akéla', 'Fenrir', 'Loupiot', 'Gris', 'Nuage', 'Flocon', 'Brume', 'Tison', 'Braise', 'Silex', 'Pépite', 'Rune', 'Grisou', 'Filou', 'Croc', 'Plume'],
  words: FORGE_WORDS,
}
