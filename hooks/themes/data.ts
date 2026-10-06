// A theme from a JSON file: the pictures as letter grids, where things stand,
// a few effects, and the words. Everything the engine knows of the work shows
// in it the same way it shows in the forge: the character's poses, the bubbles,
// the helpers for subagents, the guests from other sessions, the pet, the sky.
//
// THEMES.md describes the format; checkTheme says, in French, what is wrong
// with one, and refuses it rather than draw a broken scene.

import type { Activity } from '../../types'
import { BASE_PALETTE, BASE_VARIANTS, BUBBLE, CLAWS, GIFTS, PUFF, RAVEN, SAYING, SPARKS, THOUGHT, Z_BIG, Z_SMALL, drawDream, drawSky, drawSparks, rareFor } from '../props.ts'
import { blank, mirror, put, rand, stamp, toCells } from '../pixels.ts'
import type { Grid } from '../pixels.ts'
import { CALM, CROP_ORDER } from '../world.ts'
import type { Crop, Lexicon, Theme, Variant, Weapon, World } from '../world.ts'
import { NEUTRAL_WORDS } from './neutral-words.ts'

type Rows = readonly string[]
type Frames = readonly Rows[]
type Point = readonly [number, number]
type PoseName = Activity | 'walk'
type PoseSpec = Frames | { frames: Frames; every?: number; at?: Point }

type Effect =
  | { type: 'fire'; at: Point; size: Point; colors?: readonly string[] }
  | { type: 'smoke'; at: Point; colors?: readonly string[] }
  | { type: 'sparks'; at: Point }
  | { type: 'glow'; at: Point; colors?: readonly string[]; night?: boolean }

/** A theme file, as read: see THEMES.md. */
export type ThemeSpec = {
  diorama: number
  name: string
  width: number
  height?: number
  crops?: Partial<Record<Crop, Point>>
  palette?: Record<string, string>
  sky?: { at: Point; size: Point; shutters?: boolean }
  background?: Rows
  foreground?: Rows
  character: {
    home: Point
    every?: number
    mirror?: boolean
    recolor?: readonly [string, string]
    bubble?: Point
    poses: Partial<Record<PoseName, PoseSpec>>
  }
  accessories?: Record<string, { at: Point; draw: Rows }>
  variants?: Record<string, { colors: readonly [string, string]; meaning?: string }>
  stage?: { min?: number; max?: number; idle?: readonly number[]; work?: number; build?: number; offstage?: number; guests?: readonly number[] }
  effects?: readonly Effect[]
  gauges?: ReadonlyArray<{ of: 'ctx' | '5h' | '7d' | 'cost'; points: ReadonlyArray<readonly [number, number, string]> }>
  trophies?: { slots: readonly Point[]; draw: Partial<Record<'tests' | 'builds' | 'short' | 'other', Rows>>; honour?: { at: Point; draw: Rows } }
  helpers?: { walk: Frames; work?: Frames; area: readonly [number, number, number]; recolor?: string }
  pet?: { walk: Frames; stand?: Rows; sleep: Rows; perch: Point; floor: number; gate: number; facing?: 'left' | 'right' }
  messenger?: { fly: Frames }
  gift?: { at: Point }
  feast?: { at: Point; draw: Rows }
  names?: readonly string[]
  helperNames?: readonly string[]
  words?: Partial<{ [K in keyof Lexicon]: Lexicon[K] extends string ? string : Partial<Lexicon[K]> }>
}

const ACTIVITIES: readonly Activity[] = ['idle', 'thinking', 'running', 'review', 'waiting', 'failed', 'jumping', 'waving', 'quench', 'ship', 'sweep', 'sleep', 'waking', 'away']

/** The pose a moment falls back to when the theme did not draw its own. */
const FALLBACK: Record<Activity, readonly PoseName[]> = {
  idle: ['idle'],
  thinking: ['thinking', 'idle'],
  running: ['running', 'idle'],
  review: ['review', 'thinking', 'idle'],
  waiting: ['waiting', 'idle'],
  failed: ['failed', 'idle'],
  jumping: ['jumping', 'waving', 'idle'],
  waving: ['waving', 'jumping', 'idle'],
  quench: ['quench', 'running', 'idle'],
  ship: ['ship', 'running', 'idle'],
  sweep: ['sweep', 'running', 'idle'],
  sleep: ['sleep', 'idle'],
  waking: ['waking', 'waving', 'idle'],
  away: ['idle'],
}

// How the fire burns and the smoke rises by moment, as at the forge
const FIRE: Record<Activity, number> = {
  idle: 1, thinking: 2, running: 2, review: 1, waiting: 1, failed: 0, jumping: 3, waving: 2,
  quench: 2, ship: 2, sweep: 1, sleep: 0, waking: 1, away: 1,
}
const SMOKE: Record<Activity, number> = {
  idle: 5, thinking: 4, running: 2, review: 4, waiting: 5, failed: 0, jumping: 2, waving: 4,
  quench: 3, ship: 4, sweep: 5, sleep: 0, waking: 3, away: 5,
}

const TROPHY: Record<Weapon, 'tests' | 'builds' | 'short' | 'other'> = { sword: 'tests', axe: 'builds', dagger: 'short', mace: 'other' }

const DEFAULT_NAMES = ['Ada', 'Alan', 'Grace', 'Linus', 'Margaret', 'Dennis', 'Barbara', 'Ken', 'Hedy', 'Edsger', 'Frances', 'Niklaus']
const DEFAULT_HELPER_NAMES = ['Pip', 'Bip', 'Zou', 'Tic', 'Tac', 'Lou', 'Mo', 'Kiwi', 'Pixel', 'Bulle', 'Nano', 'Plume']

type Pose = { frames: Frames; every?: number; at?: Point }
const posesOf = (p: PoseSpec | undefined): Pose | undefined =>
  p === undefined ? undefined : Array.isArray(p) ? { frames: p as Frames } : (p as Pose)

const widthOf = (rows: Rows) => rows.reduce((w, r) => Math.max(w, r.length), 0)

/** Letters swapped: a session's look on the character drawn in the theme's own colours. */
const recolor = (rows: Rows, from: readonly [string, string] | undefined, to: readonly [string, string]) =>
  from === undefined ? rows : rows.map(r => [...r].map(ch => (ch === from[0] ? to[0] : ch === from[1] ? to[1] : ch)).join(''))

/** Words: the neutral ones, and over them whatever the theme says its own way. */
function wordsOf(spec: ThemeSpec): Lexicon {
  const w = spec.words ?? {}
  const n = NEUTRAL_WORDS
  return {
    persona: w.persona ?? n.persona,
    helper: w.helper ?? n.helper,
    placeOf: w.placeOf ?? n.placeOf,
    lines: { ...n.lines, ...(w.lines ?? {}) },
    labels: { ...n.labels, ...(w.labels ?? {}) },
    atWork: { ...n.atWork, ...(w.atWork ?? {}) },
    moods: { ...n.moods, ...(w.moods ?? {}) },
    details: { ...n.details, ...(w.details ?? {}) },
    notes: { ...n.notes, ...(w.notes ?? {}), gifts: { ...n.notes.gifts, ...(w.notes?.gifts ?? {}) } },
    teaToast: w.teaToast ?? n.teaToast,
    today: { ...n.today, ...(w.today ?? {}) },
    helpers: w.helpers ?? n.helpers,
  }
}

const hexOf = (s: string) => Number.parseInt(s.slice(1), 16)

/**
 * What is wrong with a theme file, in French, one line each; empty when it is
 * fit to draw. Every grid's letters must be in the palette, every place in the scene.
 */
export function checkTheme(raw: unknown): string[] {
  const errors: string[] = []
  if (typeof raw !== 'object' || raw === null) return ['le fichier ne contient pas un objet JSON']
  const spec = raw as Partial<ThemeSpec>
  if (spec.diorama !== 1) errors.push('« diorama » doit valoir 1 (la version du format)')
  if (typeof spec.name !== 'string' || spec.name.trim() === '') errors.push('« name » manque : le nom du thème')
  const width = spec.width
  if (typeof width !== 'number' || !Number.isInteger(width) || width < 24 || width > 120) errors.push('« width » doit être un entier entre 24 et 120 (pixels)')
  const height = spec.height ?? 32
  if (!Number.isInteger(height) || height < 8 || height > 32 || height % 2 !== 0) errors.push('« height » doit être pair, entre 8 et 32 (pixels ; deux par ligne de terminal)')
  const palette: Record<string, number> = { ...BASE_PALETTE }
  for (const [k, v] of Object.entries(spec.palette ?? {})) {
    if ([...k].length !== 1 || k === '.') errors.push(`palette : « ${k} » doit être une seule lettre, autre que le point`)
    else if (typeof v !== 'string' || !/^#[0-9a-fA-F]{6}$/.test(v)) errors.push(`palette : « ${k} » doit être une couleur #rrggbb`)
    else palette[k] = hexOf(v)
  }
  const grid = (where: string, rows: unknown): rows is Rows => {
    if (!Array.isArray(rows) || rows.length === 0 || !rows.every(r => typeof r === 'string')) {
      errors.push(`${where} : un dessin est une liste de lignes de texte`)
      return false
    }
    rows.forEach((row: string, j) => {
      for (const ch of row) if (ch !== '.' && palette[ch] === undefined) {
        errors.push(`${where}, ligne ${j + 1} : la lettre « ${ch} » n'est pas dans la palette`)
        return
      }
    })
    return true
  }
  const frames = (where: string, list: unknown): list is Frames => {
    if (!Array.isArray(list) || list.length === 0) {
      errors.push(`${where} : il faut au moins une image (une liste de dessins)`)
      return false
    }
    return list.every((f, i) => grid(`${where}[${i}]`, f))
  }
  const point = (where: string, p: unknown, w = width ?? 0, h = height): p is Point => {
    if (!Array.isArray(p) || p.length < 2 || !p.every(n => typeof n === 'number' && Number.isInteger(n))) {
      errors.push(`${where} : une position s'écrit [x, y], en pixels entiers`)
      return false
    }
    if (p[0] < -w || p[0] > 2 * w || p[1] < -h || p[1] > 2 * h) errors.push(`${where} : [${p[0]}, ${p[1]}] est loin hors de la scène`)
    return true
  }
  if (spec.background !== undefined) grid('background', spec.background)
  if (spec.foreground !== undefined) grid('foreground', spec.foreground)
  if (spec.sky) {
    point('sky.at', spec.sky.at)
    point('sky.size', spec.sky.size)
    if (Array.isArray(spec.sky.size) && ((spec.sky.size[0] ?? 0) < 4 || (spec.sky.size[1] ?? 0) < 4)) errors.push('sky.size : un ciel fait au moins 4 × 4 pixels')
  }
  for (const [name, c] of Object.entries(spec.crops ?? {})) {
    if (!CROP_ORDER.includes(name as Crop)) errors.push(`crops : « ${name} » n'est pas un cadrage (full, compact, solo)`)
    else if (!Array.isArray(c) || c.length !== 2 || !(c[0] >= 0 && c[1] <= (width ?? 0) && c[1] - c[0] >= 8)) errors.push(`crops.${name} : [début, fin] dans la scène, 8 pixels au moins`)
  }
  const ch = spec.character
  if (!ch || typeof ch !== 'object') {
    errors.push('« character » manque : le personnage et ses poses')
  } else {
    point('character.home', ch.home)
    if (!ch.poses || typeof ch.poses !== 'object' || ch.poses.idle === undefined) errors.push('character.poses.idle manque : la pose de repos est obligatoire')
    for (const [name, p] of Object.entries(ch.poses ?? {})) {
      if (name !== 'walk' && !ACTIVITIES.includes(name as Activity)) {
        errors.push(`character.poses : « ${name} » n'est pas une pose connue (${['walk', ...ACTIVITIES].join(', ')})`)
        continue
      }
      const pose = posesOf(p as PoseSpec)
      if (pose) frames(`character.poses.${name}`, pose.frames)
      if (pose?.at !== undefined) point(`character.poses.${name}.at`, pose.at)
    }
    if (ch.recolor && (!Array.isArray(ch.recolor) || ch.recolor.length !== 2)) errors.push('character.recolor : deux lettres, [sombre, claire]')
  }
  for (const [name, a] of Object.entries(spec.accessories ?? {})) {
    if (name === 'aucun') errors.push('accessories : « aucun » est réservé (sans accessoire)')
    point(`accessories.${name}.at`, a.at)
    grid(`accessories.${name}.draw`, a.draw)
  }
  for (const [name, v] of Object.entries(spec.variants ?? {})) {
    if (!Array.isArray(v.colors) || v.colors.length !== 2 || v.colors.some(c => palette[c] === undefined)) errors.push(`variants.${name}.colors : deux lettres de la palette, [sombre, claire]`)
  }
  for (const [i, e] of (spec.effects ?? []).entries()) {
    if (!['fire', 'smoke', 'sparks', 'glow'].includes(e.type)) errors.push(`effects[${i}] : « ${e.type} » n'est pas un effet (fire, smoke, sparks, glow)`)
    point(`effects[${i}].at`, e.at)
    if (e.type === 'fire') point(`effects[${i}].size`, e.size)
    for (const c of ('colors' in e ? e.colors : undefined) ?? []) if (palette[c] === undefined) errors.push(`effects[${i}].colors : « ${c} » n'est pas dans la palette`)
  }
  for (const [i, gauge] of (spec.gauges ?? []).entries()) {
    if (!['ctx', '5h', '7d', 'cost'].includes(gauge.of)) errors.push(`gauges[${i}].of : ctx, 5h, 7d ou cost`)
    for (const p of gauge.points ?? []) if (!Array.isArray(p) || p.length !== 3 || palette[p[2]] === undefined) errors.push(`gauges[${i}].points : chaque point s'écrit [x, y, "lettre"]`)
  }
  if (spec.trophies) {
    spec.trophies.slots?.forEach((p, i) => point(`trophies.slots[${i}]`, p))
    for (const [k, rows] of Object.entries(spec.trophies.draw ?? {})) grid(`trophies.draw.${k}`, rows)
    if (spec.trophies.honour) grid('trophies.honour.draw', spec.trophies.honour.draw)
  }
  if (spec.helpers) {
    frames('helpers.walk', spec.helpers.walk)
    if (spec.helpers.work !== undefined) frames('helpers.work', spec.helpers.work)
    if (!Array.isArray(spec.helpers.area) || spec.helpers.area.length !== 3) errors.push('helpers.area : [x de début, x de fin, y des pieds]')
  }
  if (spec.pet) {
    frames('pet.walk', spec.pet.walk)
    grid('pet.sleep', spec.pet.sleep)
    if (spec.pet.stand !== undefined) grid('pet.stand', spec.pet.stand)
    point('pet.perch', spec.pet.perch)
  }
  if (spec.messenger) frames('messenger.fly', spec.messenger.fly)
  if (spec.feast) grid('feast.draw', spec.feast.draw)
  for (const key of ['names', 'helperNames'] as const) {
    const list = spec[key]
    if (list !== undefined && (!Array.isArray(list) || list.length === 0 || !list.every(n => typeof n === 'string' && n.length > 0 && n.length <= 14))) errors.push(`${key} : une liste de prénoms, 14 lettres au plus`)
  }
  if (errors.length > 0) return errors
  // output contract: the character must show; a blank scene is a broken theme
  try {
    const theme = dataTheme(spec as ThemeSpec, 'check')
    const cells = theme.frame('full', 'idle', 0, CALM)
    let painted = 0
    for (let i = 0; i < cells.length; i += 3) if (cells[i] !== 0x20) painted++
    if (painted < 20) errors.push(`la scène de repos ne montre presque rien (${painted} cases peintes) : le personnage est-il dans le cadre ?`)
  } catch (e) {
    errors.push(`le thème ne se dessine pas : ${e instanceof Error ? e.message : String(e)}`)
  }
  return errors
}

/** The theme a checked spec makes; `id` is how the person named it. */
export function dataTheme(spec: ThemeSpec, id: string): Theme {
  const W = spec.width
  const H = spec.height ?? 32
  const palette: Record<string, number> = { ...BASE_PALETTE }
  for (const [k, v] of Object.entries(spec.palette ?? {})) palette[k] = hexOf(v)
  const ch = spec.character
  const [homeX, homeY] = ch.home
  const every = Math.max(1, ch.every ?? 4)
  const idle = posesOf(ch.poses.idle)?.frames ?? [['k']]
  const charW = widthOf(idle[0] ?? [])
  const bubbleAt = ch.bubble ?? [charW + 1, -6]
  const st = spec.stage ?? {}
  const offstage = st.offstage ?? W - homeX + 2
  const stage = {
    roam: { min: st.min ?? -10, max: st.max ?? 10 },
    idleSpots: st.idle ?? [-8, -4, 0, 0, 4, 8],
    offstage,
    guestSpots: st.guests ?? [offstage - charW - 6, offstage - 2 * charW - 10],
  }
  const work = st.work ?? 0
  const build = st.build ?? work
  const variants: Record<string, Variant> = spec.variants
    ? Object.fromEntries(Object.entries(spec.variants).map(([k, v]) => [k, { colors: v.colors, meaning: v.meaning ?? k }]))
    : ch.recolor ? { ...BASE_VARIANTS } : { defaut: { colors: ['o', 'O'], meaning: 'tout sujet' } }
  const accessories = ['aucun', ...Object.keys(spec.accessories ?? {})]
  const crops: Theme['crops'] = { full: spec.crops?.full ?? [0, W], ...(spec.crops?.compact ? { compact: spec.crops.compact } : {}), ...(spec.crops?.solo ? { solo: spec.crops.solo } : {}) }
  const sky = spec.sky
  const panes = sky ? { x: sky.at[0], y: sky.at[1], w: sky.size[0], h: sky.size[1] } : undefined
  const effects = spec.effects ?? []
  const sparksAt = effects.find(e => e.type === 'sparks')?.at

  /**
   * The character's picture for a moment: its pose's frame, in a session's look,
   * its accessory on, and where it sits from the home spot (a pose may reach out).
   */
  function figure(activity: Activity, tick: number, w: World): { rows: Rows; at: Point } {
    let pose: Pose = posesOf(ch.poses[FALLBACK[activity].find(p => ch.poses[p] !== undefined) ?? 'idle']) ?? { frames: idle }
    let index = Math.floor(tick / (pose.every ?? every))
    const walk = posesOf(ch.poses.walk)
    if (w.walking && walk) {
      pose = walk
      const dir = w.facing < 0 ? -1 : 1
      index = ((w.x * dir) % walk.frames.length + walk.frames.length) % walk.frames.length // a step a pixel: the feet keep to the ground
    }
    let rows = recolor(pose.frames[index % pose.frames.length] ?? idle[0] ?? [], ch.recolor, w.cape)
    const at = pose.at ?? [0, 0]
    const acc = spec.accessories?.[w.accessory]
    if (acc) {
      // the accessory sits on the character, wherever the pose put it in its frame
      const width = Math.max(widthOf(rows), acc.at[0] - at[0] + widthOf(acc.draw))
      const g = rows.map(r => [...r.padEnd(width, '.')])
      stamp(g, acc.at[0] - at[0], acc.at[1] - at[1], recolor(acc.draw, ch.recolor, w.cape))
      rows = g.map(r => r.join(''))
    }
    if (ch.mirror && w.walking && w.facing < 0) rows = mirror(rows.map(r => r.padEnd(charW, '.')))
    return { rows, at }
  }

  function drawFire(g: Grid, e: Extract<Effect, { type: 'fire' }>, level: number, tick: number) {
    const [fx, fy] = e.at, [fw, fh] = e.size
    const [core = 'Y', bright = 'f', flame = 'F', tip = 'E'] = e.colors ?? []
    for (let c = 0; c < fw; c++) {
      const centre = 1 - Math.abs(c - (fw - 1) / 2) / (fw / 2)
      const flicker = rand(tick * 31 + c * 7) * 2 - 0.6
      const h = level === 0
        ? (rand(tick * 13 + c) > 0.7 ? 1 : 0)
        : Math.max(1, Math.min(fh, Math.round(((level * 2 * centre + level + flicker) * fh) / 8)))
      for (let k = 0; k < h; k++) {
        const f = k / Math.max(1, h - 1)
        put(g, fx + c, fy + fh - 1 - k, level === 0 ? tip : f < 0.25 ? core : f < 0.55 ? bright : f < 0.85 ? flame : tip)
      }
    }
  }

  function drawSmoke(g: Grid, at: Point, colors: readonly string[], rate: number, tick: number) {
    if (rate <= 0) return
    const [light = 'X', dark = 'x'] = colors
    for (let born = tick - 10; born <= tick; born++) {
      if (born < 0 || born % rate !== 0) continue
      const age = tick - born
      const drift = Math.round(rand(born) * 2 - 1 + age * 0.3)
      const x = at[0] + drift, y = at[1] - age
      put(g, x, y, age < 3 ? light : dark)
      put(g, x + 1, y, age < 3 ? light : dark)
      if (age > 1 && age < 7) { put(g, x, y - 1, dark); put(g, x - 1, y, dark) }
    }
  }

  /** The helpers' yard: three places across the area, each helper pacing round its own. */
  function drawHelpers(g: Grid, w: World) {
    const hs = spec.helpers
    if (!hs) return
    const [x0, x1, feet] = hs.area
    const walkH = hs.walk[0]?.length ?? 0
    const hw = widthOf(hs.walk[0] ?? [])
    const span = Math.max(0, x1 - x0 - hw)
    const slots = [{ x: 0, back: 2 }, { x: Math.round(span / 2), back: 0 }, { x: span, back: 1 }]
    const live = w.apprentices.filter(c => c.leaving === undefined).slice(0, slots.length)
    const going = w.apprentices.filter(c => c.leaving !== undefined).slice(0, 2)
    const pack = [...live.map((c, i) => ({ c, slot: i })), ...going.map(c => ({ c, slot: c.seed % slots.length }))]
    pack.sort((a, b) => (slots[b.slot]?.back ?? 0) - (slots[a.slot]?.back ?? 0))
    for (const { c, slot } of pack) {
      const place = slots[slot] ?? { x: 0, back: 0 }
      const pace = 0.3 + rand(c.seed) * 0.3
      const cycles = Math.floor(c.age / 40)
      const walked = cycles * 28 + Math.min(c.age % 40, 28)
      const d = (walked * pace + rand(c.seed * 3) * 12) % 12
      const out = d < 6
      let x = x0 + place.x + Math.round((out ? d : 12 - d) - 3)
      let dir = out ? 1 : -1
      let working = c.age % 40 >= 28 && c.leaving === undefined && hs.work !== undefined
      let travel = walked * pace
      if (c.leaving !== undefined) {
        x += Math.round(c.leaving * 1.3)
        dir = 1
        working = false
        travel = c.leaving * 1.3
      }
      const list = working ? hs.work ?? hs.walk : hs.walk
      const frame = list[working ? Math.floor(c.age / 2) % list.length : Math.floor(travel) % list.length] ?? []
      const rows = frame.map(r => r.replaceAll(hs.recolor ?? 'C', c.colour))
      const y = feet - walkH + 1 - place.back
      stamp(g, x, y, dir > 0 ? rows : mirror(rows.map(r => r.padEnd(hw, '.'))))
      if (c.age < 8 && c.leaving === undefined) stamp(g, x + Math.floor(hw / 2) - 3, y, PUFF[Math.min(2, c.age >> 2)] ?? [])
    }
  }

  function drawPet(g: Grid, cat: NonNullable<World['cat']>) {
    const pet = spec.pet
    if (!pet) return
    if (cat.mode === 'roof') {
      stamp(g, pet.perch[0], pet.perch[1], pet.sleep)
      if (cat.age % 40 < 12) put(g, pet.perch[0] + widthOf(pet.sleep), pet.perch[1] - Math.floor((cat.age % 40) / 4), 'u')
      return
    }
    const frame = cat.step === undefined ? pet.stand ?? pet.walk[0] ?? [] : pet.walk[cat.step % pet.walk.length] ?? []
    const goingRight = (cat.dir ?? (cat.mode === 'flee' ? 1 : -1)) > 0
    const drawnRight = pet.facing === 'right'
    const rows = goingRight === drawnRight ? frame : mirror(frame.map(r => r.padEnd(widthOf(frame), '.')))
    stamp(g, cat.x, pet.floor - frame.length + 1, rows)
  }

  function compose(activity: Activity, tick: number, w: World): Grid {
    const g = blank(W, H)
    const clock = w.clock
    const night = w.hour < 6 || w.hour >= 21
    if (panes) drawSky(g, w, clock, panes, sky?.shutters ?? false)
    if (spec.background) stamp(g, 0, 0, spec.background)
    // the days' trophies, and the masterpiece at the place of honour
    const tr = spec.trophies
    if (tr) {
      w.rack.slice(-tr.slots.length).forEach((weapon, i, all) => {
        const rows = tr.draw[TROPHY[weapon]] ?? tr.draw.other
        const at = tr.slots[i]
        if (!rows || !at) return
        stamp(g, at[0], at[1], rows)
        if (w.forging && i === all.length - 1 && clock % 6 < 3) put(g, at[0] + Math.floor(widthOf(rows) / 2), at[1], 'Y')
      })
      if (w.masterpiece && tr.honour) {
        stamp(g, tr.honour.at[0], tr.honour.at[1], tr.honour.draw)
        if (clock % 20 < 3) put(g, tr.honour.at[0], tr.honour.at[1] + (clock % 4), 'w')
      }
    }
    for (const gauge of spec.gauges ?? []) {
      const n = gauge.points.length
      const count = gauge.of === 'ctx' ? Math.round((Math.min(100, Math.max(0, w.ctx)) / 100) * n)
        : gauge.of === '5h' ? (w.fuel5h === undefined ? n : Math.round(n * (1 - Math.min(100, w.fuel5h) / 100)))
        : gauge.of === '7d' ? (w.fuel7d === undefined ? n : Math.round(n * (1 - Math.min(100, w.fuel7d) / 100)))
        : w.usd === undefined || w.usd <= 0 ? 0 : Math.min(n, 1 + Math.floor(Math.log2(1 + w.usd) * 2))
      gauge.points.slice(0, count).forEach(([x, y, c]) => put(g, x, y, c))
    }
    for (const e of effects) {
      if (e.type === 'fire') {
        const level = w.exhausted ? 0 : Math.min(4, FIRE[activity] + (activity === 'running' ? w.effort + (w.flavor === 'build' ? 1 : 0) : 0))
        drawFire(g, e, level, tick)
      } else if (e.type === 'smoke') {
        drawSmoke(g, e.at, e.colors ?? [], w.exhausted ? 0 : activity === 'running' && w.flavor === 'build' ? 1 : SMOKE[activity], tick)
      } else if (e.type === 'glow' && (!e.night || night || w.lanternLit)) {
        const [a = 'Y', b = 'f'] = e.colors ?? []
        put(g, e.at[0], e.at[1], rand(clock) > 0.5 ? a : b)
      }
    }

    // the character, unless it is away visiting
    const cx = homeX + w.x, cy = homeY
    if (!w.absent) {
      const { rows, at } = figure(activity, tick, w)
      stamp(g, cx + at[0], cy + at[1], rows)
      const bx = cx + bubbleAt[0], by = Math.max(0, cy + bubbleAt[1])
      if (activity === 'waiting' || (activity === 'waking' && tick < 4)) stamp(g, bx, Math.max(0, by + (tick % 4 < 2 ? 0 : 1)), BUBBLE)
      else if (activity === 'thinking') stamp(g, bx, by, THOUGHT[Math.floor(tick / 3) % 2] ?? [])
      else if (w.speaking) stamp(g, bx, by + 1, SAYING)
      if (activity === 'sleep') {
        if (w.dream) drawDream(g, cx + Math.max(0, Math.floor((charW - 13) / 2)), Math.max(0, cy - 9), w.dream)
        for (let i = 0; i < 2; i++) {
          const a = (tick + i * 10) % 20
          stamp(g, bx + Math.floor(a / 4), cy + 1 - Math.floor(a / 3), i ? Z_SMALL : Z_BIG)
        }
      }
      if (w.pressure >= 85 && activity !== 'sleep') {
        put(g, cx + charW, cy + 3 + (tick % 6 > 2 ? 1 : 0), 'c')
        put(g, cx + charW, cy + 4 + (tick % 6 > 2 ? 1 : 0), 'c')
      }
      if (activity === 'failed' && tick < 7) stamp(g, cx + Math.floor(charW / 2) - 3, Math.max(0, cy - 2 - Math.min(2, tick >> 1)), PUFF[Math.min(2, tick >> 1)] ?? [])
      if (activity === 'running' && sparksAt) {
        const c = tick % 8
        if (c >= 3) drawSparks(g, c - 3, Math.floor(tick / 8), w.hammer === 'sledge' ? 16 : w.hammer === 'small' ? 8 : 12, sparksAt[0], sparksAt[1], SPARKS[w.flavor])
      }
      if (activity === 'jumping') {
        drawSparks(g, tick % 7, Math.floor(tick / 7) + 99, 16, cx + Math.floor(charW / 2), Math.max(2, cy - 2), SPARKS.forge)
        for (let i = 0; i < 6; i++) {
          const sx = Math.floor(rand(Math.floor(tick / 2) * 11 + i) * W)
          const sy = Math.floor(rand(Math.floor(tick / 2) * 23 + i) * Math.min(12, H))
          put(g, sx, sy, i % 2 ? 's' : 'Y')
        }
      }
    }
    if (spec.foreground) stamp(g, 0, 0, spec.foreground)
    drawHelpers(g, w)

    // guests from other sessions: the same character, each in its own look
    for (const v of w.guests.slice(0, 2)) {
      const gw: World = { ...w, x: v.x, walking: v.walking, facing: v.facing, cape: v.cape, accessory: v.accessory }
      const { rows, at } = figure(v.waving ? 'waving' : 'idle', v.tick, gw)
      stamp(g, homeX + v.x + at[0], homeY + at[1], rows)
    }
    if (w.banquet && spec.feast) stamp(g, spec.feast.at[0], spec.feast.at[1], spec.feast.draw)
    if (w.cat) drawPet(g, w.cat)
    if (w.gift && spec.gift) {
      stamp(g, spec.gift.at[0], spec.gift.at[1], GIFTS[w.gift.kind])
      if (w.gift.age % 16 < 3) put(g, spec.gift.at[0] + 2, spec.gift.at[1] - 2, 'w')
    }
    if (w.messenger) {
      const fly = spec.messenger?.fly ?? RAVEN
      const x = W - Math.round(w.messenger.age * 1.6), y = Math.round(Math.sin(w.messenger.age / 3))
      stamp(g, x, y, fly[w.messenger.age % fly.length] ?? [])
      if (!spec.messenger) stamp(g, x + CLAWS.x, y + CLAWS.y, w.messenger.kind === 'waiting' ? ['WFW', '.F.'] : ['WWV'])
    }
    return g
  }

  return {
    id,
    title: spec.name,
    width: W,
    rows: H / 2,
    crops,
    palette,
    frame(crop, activity, tick, world = CALM) {
      const [x0, x1] = crops[crop] ?? crops.full
      return toCells(compose(activity, tick, world), palette, x0, x1)
    },
    anchorOf(activity, w) {
      switch (activity) {
        case 'running': return w.flavor === 'build' ? build : work
        case 'quench': case 'ship': case 'failed': return work
        case 'idle': return w.routine === 'lantern' ? stage.roam.min : w.routine === 'lunch' || w.hourglasses > 0 || w.exhausted || w.tea || w.duck ? work : undefined
        default: return undefined
      }
    },
    stage,
    pet: spec.pet ? { gate: spec.pet.gate, away: W + 12, follow: homeX - 9 } : undefined,
    sky: crop => {
      if (!panes) return false
      const [x0, x1] = crops[crop] ?? crops.full
      return panes.x >= x0 && panes.x + panes.w <= x1
    },
    rareFor: panes ? rareFor(panes.w + 2) : 0,
    looks: { variants, accessories },
    names: spec.names ?? DEFAULT_NAMES,
    helperNames: spec.helperNames ?? DEFAULT_HELPER_NAMES,
    words: wordsOf(spec),
  }
}
