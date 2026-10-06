// The contract between Diorama's engine and a theme.
//
// The engine follows Claude Code (the turn, the tools, the subagents, the gauges,
// the other sessions) and keeps a World: what is going on, in plain facts. A theme
// turns a World into pixels, and gives the words that go with its pictures. The
// forge (themes/forge.ts) is one theme, drawn in code; any other is a JSON file
// read by themes/data.ts. Nothing in the engine knows what a forge is.

import type { Activity, Flavor } from '../types'

/** Horizontal windows over the scene, widest first: the band shows the widest that fits. */
export type Crop = 'full' | 'compact' | 'solo'
export const CROP_ORDER: readonly Crop[] = ['full', 'compact', 'solo']

/** What the character is doing about the work, whatever it is drawn as. */
export type World = {
  flavor: Flavor
  /** The fullest gauge, percent: worry shows past 85. */
  pressure: number
  /** Context fill, percent. */
  ctx: number
  /** 5 h window used, percent. */
  fuel5h?: number
  /** 7 day window used, percent. */
  fuel7d?: number
  /** Subagents at work, and those just done walking out: one helper each. */
  apprentices: readonly Cub[]
  /** Commands running in the background. */
  hourglasses: number
  /** Ticks since the main turn began; 0 outside a turn. */
  turnTicks: number
  /** Ticks since the session began: what keeps going between moments. */
  clock: number
  /** Ticks since a background job came back. */
  bellAge?: number
  /** How the last tests went. */
  outcome?: 'pass' | 'fail'
  /** What git just did. */
  ship?: 'commit' | 'push'
  /** Local time, hours 0..24. */
  hour: number
  /** Local date. */
  month: number
  day: number
  /** A run of failures brings a storm, a run of successes a rainbow. */
  weather?: 'storm' | 'rainbow'
  /** The model at work: Haiku small, Sonnet normal, Opus and Fable big. */
  hammer: 'small' | 'normal' | 'sledge'
  /** The effort asked of the model, 0..2. */
  effort: number
  /** What the session has cost, US dollars. */
  usd?: number
  /** Where the character stands, pixels from its home spot. */
  x: number
  /** Whether it is on its way somewhere. */
  walking: boolean
  /** Which way it walks: -1 left, 1 right. */
  facing: number
  /** The trophies of the last days worked, oldest first, at most seven; the last may be in the making. */
  rack: readonly Weapon[]
  /** Whether today's trophy, the last, is still being made. */
  forging: boolean
  /** The project's banner: its cloth, its emblem colour and the emblem's ten bits. */
  banner?: { cloth: string; emblem: string; bits: number }
  /** It has something to say: a bubble by its head. */
  speaking: boolean
  /** Three failures running. */
  duck: boolean
  /** A limit reached. */
  exhausted: boolean
  /** Two hours into the session. */
  tea: boolean
  /** This session's look: its two colours, [dark, light], and an accessory. */
  cape: readonly [string, string]
  accessory: string
  /** It has gone to visit another session: its own place stands empty. */
  absent: boolean
  /** Other sessions' characters visiting this one (two at most drawn). */
  guests: readonly Guest[]
  /** Three sessions or more idle at once: a feast here. */
  banquet: boolean
  /** The machine's one pet, when it is here. `step`: pixels walked; undefined while it stands still. */
  cat?: { mode: 'roof' | 'walk' | 'flee'; x: number; age: number; dir?: 1 | -1; step?: number }
  /** What a guest left behind. */
  gift?: { kind: 'coin' | 'cookie' | 'ore'; age: number }
  /** A messenger from another session: its task is done, or it waits for the person. */
  messenger?: { kind: 'done' | 'waiting'; age: number }
  /** The day's habits: coffee in the morning, a bite at noon, the lantern at dusk. */
  routine?: 'coffee' | 'lunch' | 'lantern'
  /** Whether the lantern is lit this evening. */
  lanternLit: boolean
  /** What it dreams of when it naps. */
  dream?: Dream
  /** This session's temper. */
  personality: Personality
  /** A long turn ended well: the conversation's masterpiece. */
  masterpiece: boolean
  /** A rare sight in the sky: a shooting star by night, a dragon by day. */
  rare?: { kind: 'star' | 'dragon'; age: number }
}

export type Personality = 'calme' | 'jovial' | 'grognon' | 'reveur' | 'applique'
export const PERSONALITIES: readonly Personality[] = ['jovial', 'grognon', 'reveur', 'applique']

/** A day's trophy: tests a sword, builds an axe, a short day a dagger, else a mace (the forge's names). */
export type Weapon = 'sword' | 'axe' | 'dagger' | 'mace'

/** A visiting character: where it stands (as World.x), its look, and its own beat. */
export type Guest = {
  x: number
  walking: boolean
  facing: number
  cape: readonly [string, string]
  accessory: string
  tick: number
  waving: boolean
}

/**
 * A helper for a subagent: `seed` fixes its pace and its path, `colour` its
 * colour letter (its agent type), `age` the ticks since it came in, `leaving`
 * the ticks since its work ended.
 */
export type Cub = { seed: number; colour: string; age: number; leaving?: number }

export type Dream =
  | 'voiture' | 'fusee' | 'bug' | 'circuit' | 'livre' | 'montagne' | 'poisson' | 'etoile' | 'gateau' | 'dragon'
  | 'drapeau' | 'carburant' | 'cle' | 'ecran' | 'musique' | 'base' | 'reseau' | 'manette' | 'graphique' | 'photo'
  | 'carte' | 'maison' | 'horloge' | 'cadenas' | 'nuage' | 'ampoule' | 'engrenage' | 'robot' | 'plante' | 'cafe'

export const CALM: World = {
  flavor: 'forge', pressure: 0, ctx: 0, apprentices: [], hourglasses: 0, turnTicks: 0, clock: 0,
  hour: 12, month: 6, day: 15, hammer: 'normal', effort: 0, x: 0, walking: false, facing: 0,
  rack: [], forging: false, speaking: false, duck: false, exhausted: false, tea: false,
  cape: ['o', 'O'], accessory: 'aucun', absent: false,
  guests: [], banquet: false, lanternLit: false, personality: 'calme', masterpiece: false,
}

/** The kinds of lines the character says, each a pool to pick from; `{n}` a number or a name. */
export type LineKind =
  | 'morning' | 'afternoon' | 'evening' | 'answer' | 'failed' | 'pass' | 'crack' | 'commit' | 'push' | 'level'
  | 'night' | 'wake' | 'tea' | 'duck' | 'cold' | 'leave' | 'welcome' | 'dragon' | 'star'

/**
 * The words that carry a theme's picture. Plain French that fits any theme
 * (« réfléchit », « erreur API ») stays in the engine; what names the forge,
 * its hammer, its raven or its cat comes from here. `{name}` marks are filled in.
 */
export type Lexicon = {
  /** What the character is, as Haiku is told: « un loup-dragon forgeron ». */
  persona: string
  /** What a helper is, as Haiku names one: « louveteau ». */
  helper: string
  /** Another session's place, by its project alone: « la forge {project} ». */
  placeOf: string
  lines: Record<LineKind, readonly string[]>
  /** The word under the name, by moment. */
  labels: Record<Activity, string>
  /** At work, by kind of work. */
  atWork: Record<Flavor, string>
  /** At rest, by what weighs on it. */
  moods: { cold: string; breathless: string; gauges: string; hourglass: string; duck: string; tea: string; yawn: string }
  /** The moment's detail beside the label. */
  details: { pass: string; crack: string; compact: string }
  /** The log's lines. */
  notes: {
    look: string // {cape} {accessory} {temper}
    twin: string // {who}
    helperIn: string // {name} {type} {errand}
    helperOut: string // {name}
    level: string // {n}
    petIn: string
    petFrom: string // {who}: a name
    petFromPlace: string // {project}
    petTo: string // {who}
    tripOut: string // {who}
    tripBack: string // {who}
    newsDone: string // {who}
    newsWaiting: string // {who}
    guestIn: string // {who}
    guestOut: string // {who} {gift}
    gifts: { coin: string; cookie: string; ore: string }
    feast: string
    feastOut: string // {who}
    coffee: string
    lunch: string
    lantern: string
    testsPass: string // {cmd}
    testsFail: string // {cmd}
    push: string
    commit: string
    compact: string
    masterpiece: string
    cold: string
  }
  /** The toast two hours in. */
  teaToast: string
  /** The day's tally: {n} a count, {t} a duration. */
  today: { edits: string; tests: string; builds: string; shells: string; time: string }
  /** The helpers' heading: {n} how many. */
  helpers: string
}

/** A look a session may wear: its two colours, and what it means (Haiku dresses by the subject). */
export type Variant = { colors: readonly [string, string]; meaning: string }

/** A theme: the pictures and the words for a World. */
export type Theme = {
  /** How the person names it in the settings: « forge », or the JSON file's path. */
  id: string
  /** Its title, for the log. */
  title: string
  /** The scene's size: pixels across, terminal rows (two pixels each) down. */
  width: number
  rows: number
  /** The windows over the scene; `full` always, the narrower ones when the theme has them. */
  crops: Partial<Record<Crop, readonly [number, number]>> & { full: readonly [number, number] }
  /** Letter -> 0xRRGGBB. */
  palette: Readonly<Record<string, number>>
  /** The scene for a moment, as Raster cells. */
  frame(crop: Crop, activity: Activity, tick: number, world: World): Uint32Array
  /** Where a moment holds the character (pixels from home), or undefined: free to wander. */
  anchorOf(activity: Activity, world: World): number | undefined
  /** Where things stand and go, pixels from the character's home spot. */
  stage: {
    roam: { min: number; max: number }
    idleSpots: readonly number[]
    /** Out of the scene, where anyone leaves to (a visit). */
    offstage: number
    /** Where guests stand; the second one only at a feast. */
    guestSpots: readonly number[]
  }
  /** The machine's pet, when the theme has one: where it comes in, leaves to, and follows from. */
  pet?: { gate: number; away: number; follow: number }
  /** Whether a crop shows the sky, where rare sights pass; and how many ticks one lasts. */
  sky(crop: Crop): boolean
  rareFor: number
  looks: { variants: Readonly<Record<string, Variant>>; accessories: readonly string[] }
  /** The characters' own names, one a session, and the helpers' names before Haiku gives theirs. */
  names: readonly string[]
  helperNames: readonly string[]
  words: Lexicon
}

/** Fills `{key}` marks. */
export const fill = (template: string, values: Readonly<Record<string, string | number>>) =>
  template.replace(/\{(\w+)\}/g, (all, key: string) => (key in values ? String(values[key]) : all))
