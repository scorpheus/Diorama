// Diorama's engine: it follows Claude Code (the turn, the tools, the subagents,
// the gauges, the other sessions on this machine) and keeps the World a theme
// draws, and writes the words beside the scene. What the scene looks like and
// how its words name things is the theme's (hooks/world.ts says what a theme gives).
//
// The comments speak the first theme's language: a session is a "forge", its
// character "he", subagents "cubs" or "apprentices", the machine's pet "the cat".

import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register, SessionRateLimit } from 'claude-code'

import type { Act, Activity, Crew, Day, Experience, Flavor, Gauge, Job, Status, Usage } from '../types'
import { DREAM_HINTS, DREAMS } from './props.ts'
import { forge } from './themes/forge.ts'
import { themeFile, themeFrom } from './themes/load.ts'
import { CROP_ORDER, PERSONALITIES, fill } from './world.ts'
import type { Crop, Cub, Dream, Guest, Lexicon, Personality, Theme, Weapon, World } from './world.ts'

const usage = atom({ plugin: 'diorama', key: 'usage' } as const, null)
const act = atom({ plugin: 'diorama', key: 'act' } as const, { activity: 'waving', detail: '' })
const crew = atom({ plugin: 'diorama', key: 'crew' } as const, { apprentices: [], jobs: [] })
const speech = atom({ plugin: 'diorama', key: 'speech' } as const, null)
const experience = atom({ plugin: 'diorama', key: 'xp' } as const, { xp: 0, level: 1 })
const journal = atom({ plugin: 'diorama', key: 'log' } as const, [])
const status = atom({ plugin: 'diorama', key: 'status' } as const, null)

const KEY = 'diorama'
// the `decor` option of /config: the widest window of the scene allowed
const DECOR_OF: Record<string, Crop> = { complet: 'full', compact: 'compact', seul: 'solo' }
const WORDS = 44 // columns kept for the words beside the scene: the gauges, the info line, the log
const TICK_MS = 150
const SLEEP_TICKS = Math.round((10 * 60 * 1000) / TICK_MS) // ten idle minutes
const TEA_TICKS = Math.round((2 * 60 * 60 * 1000) / TICK_MS) // two hours into the session
const SHORT_DAY_TICKS = Math.round((30 * 60 * 1000) / TICK_MS) // under half an hour at work: a dagger
const SPEECH_MS = 6000
const QUIET_TICKS = Math.round(15_000 / TICK_MS) // between two lines, unless it matters
// $.store: the days this conversation worked, its own: a new one starts with an empty rack,
// one that lasts days keeps its progress (a resumed session keeps its id)
const weekKey = () => `week:${anim.sessionId}`
const EDITS = new Set(['Edit', 'Write', 'NotebookEdit', 'MultiEdit'])

/**
 * The theme in play: the forge until the `theme` option names another. Set at
 * the session's start, and again whenever its JSON file changes on disk.
 */
let theme: Theme = forge
const words = (): Lexicon => theme.words
type Accessory = string

// Banner cloths and emblem colours, picked by the project's name.
const BANNERS: ReadonlyArray<readonly [string, string]> = [
  ['B', 'y'], ['F', 'w'], ['O', 'y'], ['P', 'i'], ['t', 'w'], ['E', 's'], ['M', 'w'], ['n', 'b'],
]

/** FNV-1a: a fixed number for a name, so a project keeps its banner. */
function hashOf(text: string): number {
  let h = 0x811c9dc5
  for (let i = 0; i < text.length; i++) h = Math.imul(h ^ text.charCodeAt(i), 0x01000193)
  return h >>> 0
}

function bannerOf(project: string): World['banner'] {
  const h = hashOf(project.toLowerCase())
  const [cloth, emblem] = BANNERS[h % BANNERS.length] ?? ['B', 'y']
  const bits = (h >>> 8) & 0x3ff
  return { cloth, emblem, bits: bits === 0 ? 0x2d5 : bits }
}

const dateKey = (ms: number) => {
  const d = new Date(ms)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

const isDay = (v: unknown): v is Day =>
  typeof v === 'object' && v !== null && typeof (v as Day).date === 'string' && typeof (v as Day).ticks === 'number'

/** The weapon a day made: tests a sword, builds an axe, a short day a dagger, else a mace. */
function weaponOf(d: Day): Weapon {
  if (d.ticks < SHORT_DAY_TICKS) return 'dagger'
  if (d.tests >= 3 && d.tests >= d.builds) return 'sword'
  if (d.builds >= 3) return 'axe'
  return 'mace'
}

const levelOf = (xp: number) => Math.floor(Math.sqrt(xp / 8)) + 1

const LEAVE_TICKS = 40 // a cub done with its work takes six seconds to walk out
const PRESENCE_KEY = 'presence' // $.store: each session's look and project, seen by the others
const NEIGHBOUR_MS = 2 * 60 * 1000 // a session not heard of for two minutes is gone home
const HAIKU = 'claude-haiku-4-5'

type Presence = {
  project: string; cape: string; accessory: Accessory; seen: number; away?: boolean
  name?: string // the smith's own name
  title?: string // the conversation's title: several share a project
  personality?: Personality
  idleSince?: number // since when this forge has had nothing to do: three at once make a banquet
}

/**
 * News a forge sends the others by raven: a long task done, or the person needed.
 * `title` names the conversation: several of them share a project.
 */
type News = { kind: 'done' | 'waiting'; project: string; name?: string; title?: string; at: number }

/** The day's feast: which forge holds it, until when; one a day. */
type Banquet = { date: string; host: string; until: number }

const NEWS_PREFIX = 'news:' // $.store: one key per sending session
const TITLE_PREFIX = 'title:' // $.store: the conversation's title, kept across a reload
const BANQUET_KEY = 'banquet'
const BANQUET_MS = 150_000
const BANQUET_IDLE_MS = 2 * 60 * 1000 // a forge idle two minutes may feast
const RAINBOW_TICKS = Math.round((2 * 60 * 1000) / TICK_MS) // a rainbow shows two minutes, then the sky clears
const LONG_TURN_TICKS = Math.round((2 * 60 * 1000) / TICK_MS) // a turn worth a raven
const MASTER_TICKS = Math.round((15 * 60 * 1000) / TICK_MS) // a turn worth a masterpiece
const STATUS_EVERY_MS = 2 * 60 * 1000 // Haiku says where the task stands every two minutes of work at most
const STATUS_FIRST_MS = 30_000 // the first time, half a minute into the turn
const RARE_ODDS = 1 / 720 // a rare sight, rolled every five seconds: about once an hour

/** A visit, as the visitor writes it under its own key: whose forge, until when, in what look. */
type Visit = {
  host: string; until: number; project: string; cape: string; accessory: Accessory; name?: string; title?: string
  // the handover: 'there' while he visits; 'leaving' when he means to go (the
  // visitor writes it); 'gone' once the host has walked him out (the host writes it)
  phase?: 'there' | 'leaving' | 'gone'
}

/** The machine's one cat: which forge has it, since when, and where it came from. */
type CatRecord = { host: string; since: number; from?: string; fromName?: string }

/** Another forge as the words name it: its smith, and its conversation's title. */
function forgeLabel(p: { project: string; name?: string; title?: string }, withTitle = true): string {
  const title = p.title && withTitle ? `« ${p.title.length > 34 ? `${p.title.slice(0, 33)}…` : p.title} »` : ''
  if (p.name) return title ? `${p.name} (${title})` : p.name
  return title || fill(words().placeOf, { project: p.project }) // a session on an older version sends no name
}
const CAT_KEY = 'cat'
// where the pet comes in and goes out, and how far behind him it follows: the theme's
const petAt = () => theme.pet ?? { gate: 14, away: theme.width + 12, follow: 21 }
const HANDOFF_TICKS = 13 // every two seconds, the visits and the cat are looked at
const HANDOFF_WAIT = 60 // nine seconds for the host to say he is gone, then home anyway

const VISIT_PREFIX = 'visit:' // $.store: one key per visiting session, so no two writers share one
const TRIP_MS = 90_000 // a visit lasts a minute and a half
const TRIP_FIRST_TICKS = Math.round((3 * 60 * 1000) / TICK_MS) // not before three minutes in
const TRIP_AGAIN_TICKS = Math.round((10 * 60 * 1000) / TICK_MS) // and not more than every ten
const POLL_TICKS = 33 // every five seconds he looks whether anyone came by

/** A fixed pseudo-random in [0, 1) for an integer seed (the sprite's own). */
const chance = (n: number) => ((Math.imul(n ^ 0x5bd1e995, 0x27d4eb2d) >>> 0) % 10007) / 10007

/** A cub's colours by its agent type, as the community's posse dresses them. */
function colourOf(type: string): string {
  const known: Record<string, string> = { Explore: 'b', Plan: 'G', 'general-purpose': 'P', claude: 'M', fork: 'y' }
  return known[type] ?? (['t', 'F', 'y', 'M', 'e', 'G'][hashOf(type) % 6] ?? 'P')
}
const xpFor = (level: number) => 8 * (level - 1) * (level - 1)

// Outcomes that play to their end: a plain change of work waits behind them.
// The greeting is not one: the first prompt must not wait on a wave.
const ONE_SHOTS = new Set<Activity>(['failed', 'jumping', 'quench', 'ship', 'waking'])

const TESTS = /\b(test|tests|pytest|gtest|ctest|vitest|jest|mocha)\b|_tests?(\.exe)?\b|--gtest_filter|plugin test/i
const BUILDS = /\b(cmake|make|ninja|msbuild|cl\.exe|gcc|g\+\+|clang|tsc|webpack|vite build|esbuild|go build|gradle|mvn)\b|\b(cargo|dotnet|npm|pnpm|yarn|bun)\s+(run\s+)?build\b/i
const GIT = /^\s*(git|gh)\b/
const COMMIT = /^\s*git\b.*\bcommit\b/
const PUSH = /^\s*git\b.*\bpush\b/
const SHELLS = new Set(['Bash', 'PowerShell'])

/** The flavor of a tool call, from its name and, for a shell, its command. */
function flavorOf(tool: string, input: Record<string, unknown>): Flavor {
  if (SHELLS.has(tool)) {
    const command = typeof input.command === 'string' ? input.command : ''
    return GIT.test(command) ? 'git' : TESTS.test(command) ? 'test' : BUILDS.test(command) ? 'build' : 'shell'
  }
  if (tool === 'WebFetch' || tool === 'WebSearch') return 'web'
  if (tool === 'Agent' || tool === 'Task') return 'agent'
  if (tool.startsWith('mcp__')) return 'mcp'
  return 'forge'
}

// Tools that look rather than make, and tools that hand the floor to the person.
const READS = new Set(['Read', 'Grep', 'Glob', 'LS', 'WebFetch', 'WebSearch', 'ToolSearch', 'NotebookRead'])
const ASKS = new Set(['AskUserQuestion', 'ExitPlanMode'])

// Claude Code's notifications that put a question to the person, and the words for each.
const NEEDS_YOU: Readonly<Record<string, string>> = {
  permission_prompt: 'permission',
  elicitation_dialog: 'question',
  elicitation_url_dialog: 'question',
  agent_needs_input: 'un apprenti attend',
}

const B64 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'

/** Standard padded base64 of the words' little-endian bytes (RasterProps.cells). */
function encode(words: Uint32Array): string {
  const bytes = new Uint8Array(words.buffer, words.byteOffset, words.byteLength)
  let out = ''
  for (let i = 0; i < bytes.length; i += 3) {
    const n = ((bytes[i] ?? 0) << 16) | ((bytes[i + 1] ?? 0) << 8) | (bytes[i + 2] ?? 0)
    out += B64.charAt((n >> 18) & 63) + B64.charAt((n >> 12) & 63)
    out += i + 1 < bytes.length ? B64.charAt((n >> 6) & 63) : '='
    out += i + 2 < bytes.length ? B64.charAt(n & 63) : '='
  }
  return out
}

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v))
const tone = (p: number) => (p >= 85 ? '#ef6b6b' : p >= 60 ? '#f2c14e' : '#6fcf7f')
const DAYS = ['dim', 'lun', 'mar', 'mer', 'jeu', 'ven', 'sam']

function countdown(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000))
  const d = Math.floor(s / 86400), h = Math.floor((s % 86400) / 3600), m = Math.floor((s % 3600) / 60)
  if (d > 0) return `${d}j${String(h).padStart(2, '0')}h`
  if (h > 0) return `${h}h${String(m).padStart(2, '0')}`
  return `${m}min`
}

/** When a window resets, in local time: `15h40`, with the day when it is not today. */
function resetAt(iso: string, now: number, withDay: boolean): string {
  const t = new Date(iso), today = new Date(now)
  const clock = `${String(t.getHours()).padStart(2, '0')}h${String(t.getMinutes()).padStart(2, '0')}`
  return withDay || t.toDateString() !== today.toDateString() ? `${DAYS[t.getDay()]} ${clock}` : clock
}

/** How many columns a window of the scene takes. */
const cropWidth = (c: Crop) => {
  const [x0, x1] = theme.crops[c] ?? theme.crops.full
  return x1 - x0
}

/** The widest window of the scene the option allows (and the theme has) that leaves room for the words. */
function fitCrop(allowed: Crop | undefined, columns: number): Crop | undefined {
  const order = CROP_ORDER.slice(CROP_ORDER.indexOf(allowed ?? 'full')).filter(c => theme.crops[c] !== undefined)
  return order.find(c => cropWidth(c) + 2 + WORDS <= columns)
}

const short = (path: unknown) => (typeof path === 'string' ? path.split(/[\\/]/).pop() ?? '' : '')
const firstWord = (command: string) => short(command.trim().split(/\s+/)[0] ?? '')

/** A few words on what the tool is touching. */
function describe(tool: string, input: Record<string, unknown>): string {
  if (typeof input.file_path === 'string') return `${tool} ${short(input.file_path)}`
  if (typeof input.command === 'string') return `${tool} ${firstWord(input.command)}`
  if (typeof input.pattern === 'string') return `${tool} ${input.pattern.slice(0, 24)}`
  if (typeof input.description === 'string') return `${tool} ${input.description.slice(0, 32)}`
  return tool
}

function toUsage(e: { context: { percent?: number; tokens?: number; window: number }; rateLimits: SessionRateLimit[]; cost?: { usd: number } }, startedAt?: number): Usage {
  return {
    startedAt,
    ctx: e.context.percent,
    tokens: e.context.tokens,
    window: e.context.window,
    limits: e.rateLimits.map(r => ({ kind: r.kind, percent: r.percentUsed, resetsAt: r.resetsAt })),
    usd: e.cost?.usd,
  }
}

const pressureOf = (u: Usage | null) =>
  u ? Math.max(u.ctx ?? 0, ...u.limits.map(l => l.percent)) : 0

type Pending = { activity: Activity; detail: string; flavor: Flavor }

/** The hammer a model swings. */
const hammerOf = (model: string): World['hammer'] =>
  /haiku/i.test(model) ? 'small' : /opus|fable/i.test(model) ? 'sledge' : 'normal'

/** The effort asked of the model, as fuel for the fire: 0 to 2. */
const effortOf = (effort: string | number | undefined): number =>
  typeof effort === 'number' ? (effort >= 0.75 ? 2 : effort >= 0.5 ? 1 : 0)
  : effort === 'max' || effort === 'xhigh' ? 2 : effort === 'high' ? 1 : 0

// The animation lives here (a reload starts it over); what the words read lives in $.state.
const anim = {
  bandId: undefined as string | undefined,
  crop: undefined as Crop | undefined, // the window the mounted Raster was drawn at
  tick: 0, // since the moment began
  clock: 0, // since the session began
  current: 'waving' as Activity,
  flavor: 'forge' as Flavor,
  pending: undefined as Pending | undefined, // the work waiting behind a one-shot
  isWorking: false,
  turnStart: -1,
  lastBusy: 0,
  bellAt: -1,
  outcome: undefined as 'pass' | 'fail' | undefined,
  ship: undefined as 'commit' | 'push' | undefined,
  pressure: 0,
  ctx: 0,
  fuel5h: undefined as number | undefined,
  fuel7d: undefined as number | undefined,
  apprentices: 0,
  jobs: 0,
  usd: undefined as number | undefined,
  nowMs: 0, // local time, kept by the ticks and set again every 30 s
  x: 0, // where he stands: 0 at the anvil, negative toward the furnace
  target: 0, // where he is going
  restUntil: 0, // he stays put until this tick, then roams again
  // each session's own phase: a reload reaches every session at once, and from a
  // shared zero their forges played the same idle, the same walks, the same visits
  phase: 0,
  cubs: new Map<string, { seed: number; colour: string; born: number }>(),
  sessionId: '',
  cape: 'foret',
  accessory: 'aucun' as Accessory,
  styled: false, // whether this session's look was chosen (by Haiku, or kept from before)
  neighbours: [] as Array<Presence & { id: string }>,
  // his own trip: walking out, away at the host's, walking back
  trip: undefined as {
    phase: 'out' | 'away' | 'leaving' | 'back'; host: string; project: string; until: number
    name: string // whom he visits, bare: "chez {name}"
    leavingSince?: number; idleAfter?: boolean
  } | undefined,
  hostAcks: [] as string[], // guests walked out of this forge, to tell their sessions
  lastTrip: -1,
  tripUntil: 0, // a banquet sets how long he stays
  // a guest from another session: who, in what look, since when, leaving since when
  // guests from other sessions: who, in what look, since when, leaving since when
  guests: new Map<string, { project: string; name?: string; title?: string; cape: string; accessory: Accessory; arrived: number; leaving?: number }>(),
  personality: 'calme' as Personality,
  opening: '', // how the conversation began: what his look and his dreams draw on
  idleSince: undefined as number | undefined,
  banquetUntil: 0, // while this forge holds the feast
  banquetGone: '', // the feast he already went to, by date and host
  newsSeen: new Map<string, number>(), // each sender's last news already shown
  startedMs: 0,
  messenger: undefined as { kind: 'done' | 'waiting'; since: number } | undefined,
  gift: undefined as { kind: 'coin' | 'cookie' | 'ore'; since: number } | undefined,
  // the machine's one cat, when it is in this forge
  cat: { mode: 'gone' as 'gone' | 'arrive' | 'roof' | 'follow' | 'flee' | 'home' | 'depart', x: 0, since: 0, steps: 0, movedAt: -1, dir: -1 as 1 | -1 },
  catNext: undefined as { id: string; label: string } | undefined, // where it goes next
  catHandoff: false, // it has walked out: the next forge is to be told
  routine: undefined as { kind: 'coffee' | 'lunch' | 'lantern'; until: number } | undefined,
  coffeeDone: false,
  lunchDone: '', // the date of the last lunch
  lanternLit: false,
  dream: undefined as Dream | undefined,
  turnPassed: false, // tests went green during this turn
  masterpiece: false,
  rare: undefined as { kind: 'star' | 'dragon'; since: number } | undefined,
  themeWanted: 'forge', // the `theme` option
  themeStamp: undefined as number | undefined, // the theme file's last change, when it is one
  doings: [] as string[], // the main thread's actions on the task at hand, for its status
  sinceStatus: 0, // actions since Haiku last said where the task stands
  statusAt: 0, // when it last did
  goal: '', // the task at hand, in a few words
  title: '', // the conversation's title, as Claude Code names it
  name: 'Ysengrin', // his own name, the theme's; never the person's
  goalSeq: 0, // a newer request makes an older answer stale
  lineSeq: 0,
  log: [] as string[],
  names: new Map<string, string>(), // the cubs' names, kept for their departure's line
  leaving: [] as Array<{ seed: number; colour: string; born: number; left: number }>,
  hammer: 'normal' as World['hammer'],
  effort: 0,
  failStreak: 0,
  successStreak: 0,
  rainbowUntil: -1, // the clock at which the run's rainbow fades
  week: [] as Day[], // the store's days, as last read or written
  today: { tests: 0, builds: 0, edits: 0, shells: 0, ticks: 0 }, // not yet written to the store
  xp: 0,
  project: '',
  banner: undefined as World['banner'],
  speaking: false,
  lastSpoke: -1_000_000,
  speechSeq: 0, // the line in the bubble: only its own timer clears it
  saidNight: false,
  saidTea: false,
  exhausted: false,
  exhaustedUntil: undefined as string | undefined,
  revert: undefined as { cancel: () => void } | undefined,
  timers: [] as Array<{ cancel: () => void }>,
  warned: new Set<string>(),
}

/** A look's two colours, or the theme's first look's when this theme has no such look. */
function colorsOf(look: string): readonly [string, string] {
  const looks = theme.looks.variants
  return (looks[look] ?? Object.values(looks)[0])?.colors ?? ['o', 'O']
}

/** The workshop as the drawing sees it now. */
function worldNow(): World {
  const now = new Date(anim.nowMs)
  return {
    hour: now.getHours() + now.getMinutes() / 60,
    month: now.getMonth() + 1,
    day: now.getDate(),
    weather: anim.failStreak >= 2 ? 'storm' : anim.clock < anim.rainbowUntil ? 'rainbow' : undefined,
    hammer: anim.hammer,
    effort: anim.effort,
    usd: anim.usd,
    x: anim.x,
    walking: anim.x !== anim.target,
    facing: anim.target < anim.x ? -1 : 1,
    apprentices: cubsNow(),
    cape: colorsOf(anim.cape),
    accessory: anim.accessory,
    absent: anim.trip?.phase === 'away' || anim.trip?.phase === 'leaving',
    guests: guestsNow(),
    banquet: anim.banquetUntil > anim.nowMs,
    cat: catNow(),
    gift: anim.gift && anim.clock - anim.gift.since < 2000 ? { kind: anim.gift.kind, age: anim.clock - anim.gift.since } : undefined,
    messenger: anim.messenger && anim.clock - anim.messenger.since < 50 ? { kind: anim.messenger.kind, age: anim.clock - anim.messenger.since } : undefined,
    routine: anim.current === 'idle' && anim.routine && anim.routine.until > anim.clock ? anim.routine.kind : undefined,
    lanternLit: anim.lanternLit,
    dream: anim.dream,
    personality: anim.personality,
    masterpiece: anim.masterpiece,
    rare: anim.rare && anim.clock - anim.rare.since < theme.rareFor ? { kind: anim.rare.kind, age: anim.clock - anim.rare.since } : undefined,
    ...rackNow(),
    banner: anim.banner,
    speaking: anim.speaking,
    duck: anim.failStreak >= 3,
    exhausted: anim.exhausted,
    tea: anim.clock >= TEA_TICKS,
    flavor: anim.flavor,
    pressure: anim.pressure,
    ctx: anim.ctx,
    fuel5h: anim.fuel5h,
    fuel7d: anim.fuel7d,
    hourglasses: anim.jobs,
    turnTicks: anim.isWorking && anim.turnStart >= 0 ? anim.clock - anim.turnStart : 0,
    clock: anim.clock + anim.phase,
    bellAge: anim.bellAt >= 0 ? anim.clock - anim.bellAt : undefined,
    outcome: anim.outcome,
    ship: anim.ship,
  }
}

function learn(u: Usage) {
  anim.pressure = pressureOf(u)
  anim.ctx = u.ctx ?? 0
  anim.fuel5h = u.limits.find(l => l.kind === 'five_hour')?.percent
  anim.fuel7d = u.limits.find(l => l.kind === 'seven_day')?.percent
  anim.usd = u.usd
  const over = u.limits.filter(l => l.percent >= 100)
  anim.exhausted = over.length > 0
  anim.exhaustedUntil = over.map(l => l.resetsAt).filter((r): r is string => r !== undefined).sort()[0]
}

/** Today's work, every session's stored part and this one's not yet written. */
function todayNow(): Day {
  const date = dateKey(anim.nowMs)
  const stored = anim.week.find(d => d.date === date)
  const t = anim.today
  return {
    date,
    tests: (stored?.tests ?? 0) + t.tests, builds: (stored?.builds ?? 0) + t.builds,
    edits: (stored?.edits ?? 0) + t.edits, shells: (stored?.shells ?? 0) + t.shells, ticks: (stored?.ticks ?? 0) + t.ticks,
  }
}

/** The rack: the days worked, today's work not yet written counted in. */
function rackNow(): { rack: Weapon[]; forging: boolean } {
  const today = todayNow()
  const days = anim.week.filter(d => d.date !== today.date)
  const worked = today.ticks > 0 || today.tests + today.builds + today.edits + today.shells > 0
  const all = worked ? [...days, today] : days
  return { rack: all.slice(-7).map(weaponOf), forging: worked && anim.isWorking }
}

/** Adds this session's work of the day to the store, which every session shares. */
async function flushWeek($: EngineInterface) {
  const raw = await $.store.get(weekKey())
  const days: Day[] = Array.isArray(raw) ? raw.filter(isDay) : []
  const date = dateKey(anim.nowMs)
  let day = days.find(d => d.date === date)
  if (!day) {
    day = { date, tests: 0, builds: 0, edits: 0, shells: 0, ticks: 0 }
    days.push(day)
  }
  const t = anim.today
  day.tests += t.tests; day.builds += t.builds; day.edits += t.edits; day.shells += t.shells; day.ticks += t.ticks
  anim.today = { tests: 0, builds: 0, edits: 0, shells: 0, ticks: 0 }
  const kept = days
    .filter(d => d.ticks > 0 || d.tests + d.builds + d.edits + d.shells > 0)
    .sort((a, b) => a.date.localeCompare(b.date))
    .slice(-7)
  await $.store.set(weekKey(), kept)
  anim.week = kept
}

/** The forge's log: the last moments worth a line, with their time. */
async function note($: EngineInterface, text: string) {
  const t = new Date(anim.nowMs)
  anim.log = [...anim.log, `${String(t.getHours()).padStart(2, '0')}h${String(t.getMinutes()).padStart(2, '0')}  ${text}`].slice(-6)
  await update($, journal, () => anim.log)
}

/** One short answer from Haiku, or undefined: a refusal or an empty reply falls back to the caller's own. */
async function askHaiku($: EngineInterface, system: string, prompt: string, maxTokens: number): Promise<string | undefined> {
  try {
    const r = await $.model.complete({ model: HAIKU, maxTokens, system, prompt })
    return r.isAnswered ? r.text.trim() || undefined : undefined
  } catch {
    return undefined
  }
}

/**
 * This session's look, chosen by Haiku from how the conversation starts, as a
 * title is: a cape and an accessory, so each session's Scorpheus is his own.
 */
async function chooseLook($: EngineInterface, opening: string) {
  anim.styled = true
  anim.opening = opening
  // "a smith" alone made Haiku pick embers and an apron for every session: it is
  // given what each cape means, and the looks already worn, to stay clear of
  const worn = anim.neighbours.map(n => `${n.cape}+${n.accessory}`).join(', ') || 'aucune'
  const looks = theme.looks
  const answer = await askHaiku($,
    'Tu choisis la tenue et le caractère d\'un personnage pour une conversation de travail, selon son SUJET. ' +
    `Capes (sens) : ${Object.entries(looks.variants).map(([k, v]) => `${k} = ${v.meaning}`).join(' ; ')}. ` +
    `Accessoires : ${looks.accessories.join(', ')}. Caractères : ${PERSONALITIES.join(', ')}. ` +
    `Tenues déjà portées, à éviter : ${worn}. ` +
    'Réponds en JSON seul : {"cape": "...", "accessoire": "...", "caractere": "..."}',
    `Projet : ${anim.project}\nDébut de la conversation : ${opening.slice(0, 600)}`, 80)
  const cape = /"cape"\s*:\s*"([a-z]+)"/.exec(answer ?? '')?.[1]
  const accessory = /"accessoire"\s*:\s*"([a-z]+)"/.exec(answer ?? '')?.[1] as Accessory | undefined
  const temper = /"caractere"\s*:\s*"([a-z]+)"/.exec(answer ?? '')?.[1] as Personality | undefined
  const look = freeLook(cape && looks.variants[cape] ? cape : anim.cape, accessory && looks.accessories.includes(accessory) ? accessory : anim.accessory)
  anim.cape = look.cape
  anim.accessory = look.accessory
  if (temper && PERSONALITIES.includes(temper)) anim.personality = temper
  await note($, fill(words().notes.look, { cape: anim.cape, accessory: anim.accessory === 'aucun' ? '' : `, ${anim.accessory}`, temper: anim.personality }))
  await heartbeat($)
}

/** The look wanted, or the nearest one no session at work is wearing. */
function freeLook(cape: string, accessory: Accessory): { cape: string; accessory: Accessory } {
  const worn = new Set(anim.neighbours.map(n => `${n.cape}+${n.accessory}`))
  const capes = Object.keys(theme.looks.variants)
  const all = capes.flatMap(c => theme.looks.accessories.map(a => ({ cape: c, accessory: a })))
  const from = Math.max(0, all.findIndex(l => l.cape === cape && l.accessory === accessory))
  for (let i = 0; i < all.length; i++) {
    const l = all[(from + i * 5) % all.length] // a step of five: another cape before another accessory
    if (l && !worn.has(`${l.cape}+${l.accessory}`)) return l
  }
  return { cape, accessory }
}

/** Tells the other sessions this one is at work, in its look; learns who else is. */
async function heartbeat($: EngineInterface) {
  const raw = await $.store.get(PRESENCE_KEY)
  const all: Record<string, Presence> = typeof raw === 'object' && raw !== null ? { ...(raw as Record<string, Presence>) } : {}
  const now = anim.nowMs
  all[anim.sessionId] = {
    project: anim.project, cape: anim.cape, accessory: anim.accessory, seen: now, away: anim.trip?.phase === 'away',
    personality: anim.personality, idleSince: anim.idleSince, name: anim.name, ...(anim.title || anim.goal ? { title: anim.title || anim.goal } : {}),
  }
  for (const [id, p] of Object.entries(all)) if (now - p.seen > 24 * 60 * 60 * 1000) delete all[id] // a day gone: forgotten
  await $.store.set(PRESENCE_KEY, all)
  anim.neighbours = Object.entries(all).filter(([id, p]) => id !== anim.sessionId && now - p.seen < NEIGHBOUR_MS).map(([id, p]) => ({ ...p, id }))
  // two forges in one look: the later of the two (by id) changes, the other keeps his
  const twin = anim.neighbours.find(n => n.cape === anim.cape && n.accessory === anim.accessory && n.id < anim.sessionId)
  if (twin) {
    const look = freeLook(anim.cape, anim.accessory)
    anim.cape = look.cape
    anim.accessory = look.accessory
    await note($, fill(words().notes.twin, { who: forgeLabel(twin, false) }))
    all[anim.sessionId] = { ...all[anim.sessionId], cape: anim.cape, accessory: anim.accessory } as Presence
    await $.store.set(PRESENCE_KEY, all)
  }
  // two smiths of one name: the later one takes the first free one after his
  if (anim.neighbours.some(n => n.name === anim.name && n.id < anim.sessionId)) {
    const worn = new Set(anim.neighbours.map(n => n.name))
    const names = theme.names
    const start = hashOf(anim.sessionId) % names.length
    const free = [...names.slice(start), ...names.slice(0, start)].find(n => !worn.has(n))
    if (free) {
      anim.name = free
      all[anim.sessionId] = { ...all[anim.sessionId], name: free } as Presence
      await $.store.set(PRESENCE_KEY, all)
    }
  }
}

/** A cub's name: one from the litter at once, then Haiku's, fitted to its errand. */
async function nameCub($: EngineInterface, id: string, type: string, errand: string) {
  const local = theme.helperNames[hashOf(id) % theme.helperNames.length] ?? 'Loupiot'
  await setCrew($, c => ({ ...c, apprentices: c.apprentices.map(a => (a.id === id ? { ...a, name: local } : a)) }))
  const given = await askHaiku($,
    `Tu donnes un prénom de ${words().helper}, court (une ou deux syllabes), joyeux, en lien avec sa mission. Réponds par le prénom seul.`,
    `Mission (${type}) : ${errand.slice(0, 200)}`, 12)
  const name = given?.split(/\s+/)[0]?.replace(/[^\p{L}-]/gu, '').slice(0, 12)
  const final = name || local
  await setCrew($, c => ({ ...c, apprentices: c.apprentices.map(a => (a.id === id ? { ...a, name: final } : a)) }))
  await note($, fill(words().notes.helperIn, { name: final, type, errand: errand.slice(0, 40) }))
}

/** He says a line for a while; chatter waits 15 s between lines unless it matters. */
async function say($: EngineInterface, text: string, matters = false) {
  if (!matters && anim.clock - anim.lastSpoke < QUIET_TICKS) return
  anim.lastSpoke = anim.clock
  anim.speaking = true
  const seq = ++anim.speechSeq
  await update($, speech, () => text)
  // the line's own number, not the ticks counted since: on Windows a 150 ms tick runs
  // about 156 ms, 38 of them in six seconds, and a count of 39 left the bubble up for good
  $.clock.after(SPEECH_MS, () => {
    if (seq !== anim.speechSeq) return // a newer line holds the bubble
    anim.speaking = false
    void update($, speech, () => null).catch(() => undefined)
  })
}

const pick = (kind: keyof Lexicon['lines'], n?: number | string) => {
  const pool = words().lines[kind] ?? []
  const line = pool[Math.floor(Math.random() * pool.length)] ?? ''
  return n === undefined ? line : line.replace('{n}', String(n))
}

/** His line at the end of a turn: from the pool, or written by Haiku from the answer. */
async function sayAnswer($: EngineInterface, answer: string, voice: string) {
  if (voice !== 'haiku' || answer.trim() === '') return say($, pick('answer'))
  let line: string | undefined
  try {
    const r = await $.model.complete({
      model: 'claude-haiku-4-5',
      maxTokens: 40,
      system: `Tu es ${anim.name}, ${words().persona}, compagnon d'un développeur.` + " Tu réponds par UNE réplique en français, 8 mots au plus, chaleureuse ou malicieuse, sur le travail qui vient d'être fait. Sans guillemets.",
      prompt: answer.slice(0, 1200),
    })
    line = r.isAnswered ? r.text.trim().split('\n')[0]?.replace(/^[«"'\s]+|[»"'\s]+$/g, '').slice(0, 60) : undefined
  } catch {
    line = undefined // a request refused before it left: the pool speaks instead
  }
  return say($, line || pick('answer'))
}

/** Experience for the session: a level up is worth a line and a cheer. */
async function gain($: EngineInterface, points: number) {
  const before = levelOf(anim.xp)
  anim.xp += points
  const level = levelOf(anim.xp)
  await update($, experience, () => ({ xp: anim.xp, level }))
  if (level > before) {
    await note($, fill(words().notes.level, { n: level }))
    await say($, pick('level', level), true)
    if (!anim.isWorking) await setAct($, 'jumping', `niveau ${level}`, 1800)
  }
}

/** The cubs on the floor: those at work, and those walking out with their scroll. */
function cubsNow(): Cub[] {
  anim.leaving = anim.leaving.filter(l => anim.clock - l.left < LEAVE_TICKS)
  return [
    ...[...anim.cubs.values()].map(c => ({ seed: c.seed, colour: c.colour, age: anim.clock - c.born })),
    ...anim.leaving.map(l => ({ seed: l.seed, colour: l.colour, age: l.left - l.born, leaving: anim.clock - l.left })),
  ]
}

/**
 * Where he heads next. A moment with a station holds him there; otherwise
 * idle he roams between his favourite spots, thinking he paces, sweeping he
 * sweeps the floor end to end, and the rest of the time he stays where he is.
 */
function steer() {
  const anchor = theme.anchorOf(anim.current, worldNow())
  const { roam, idleSpots, offstage } = theme.stage
  const trip = anim.trip
  if (trip) {
    anim.target = trip.phase === 'back' ? 0 : offstage
    if (trip.phase === 'away' || trip.phase === 'leaving') anim.x = offstage
  } else if (anchor !== undefined) {
    anim.target = anchor
  } else if (anim.x === anim.target && anim.clock >= anim.restUntil) {
    if (anim.current === 'idle') {
      const roll = anim.clock + anim.phase
      anim.target = idleSpots[Math.floor(chance(roll) * idleSpots.length)] ?? 0
      anim.restUntil = anim.clock + 80 + Math.floor(chance(roll * 7) * 120) // 12 to 30 s
    } else if (anim.current === 'thinking') {
      anim.target = anim.x <= -4 ? 3 : -6
      anim.restUntil = anim.clock + 10
    } else if (anim.current === 'sweep') {
      anim.target = anim.x <= -8 ? 4 : -10
      anim.restUntil = anim.clock + 2
    }
  }
  anim.target = Math.max(roam.min, Math.min(trip ? offstage : roam.max, anim.target))
  // a pixel a tick, half that when he paces deep in thought
  const slow = anim.current === 'thinking' && anim.clock % 2 === 1
  if (anim.x !== anim.target && !slow) anim.x += anim.x < anim.target ? 1 : -1
  if (trip?.phase === 'back' && anim.x === 0) {
    anim.trip = undefined // home
    anim.lastTrip = anim.clock
  }
}

// Moments that loop: their beat starts at the session's phase. The others (a
// quench, a crate, a cheer) play from their own start, whatever the phase.
const LOOPING = new Set<Activity>(['idle', 'thinking', 'review', 'sleep', 'sweep'])
const poseTick = () => anim.tick + (LOOPING.has(anim.current) ? anim.phase : 0)

/** The guests in the yard: walking in, greeting, standing about, walking out. */
function guestsNow(): Guest[] {
  const out: Guest[] = []
  const { guestSpots, offstage } = theme.stage
  let i = 0
  for (const [id, g] of anim.guests) {
    const spot = guestSpots[Math.min(i, guestSpots.length - 1)] ?? 0
    i += 1
    const age = anim.clock - g.arrived
    let x = Math.max(spot, offstage - age)
    let walking = x > spot
    let facing = -1
    if (g.leaving !== undefined) {
      x = spot + (anim.clock - g.leaving)
      if (x >= offstage) {
        anim.guests.delete(id)
        anim.hostAcks.push(id) // walked out: his session may bring him home now
        continue
      }
      walking = true
      facing = 1
    }
    out.push({
      x, walking, facing,
      cape: colorsOf(g.cape),
      accessory: g.accessory,
      tick: age + (hashOf(id) % 997),
      waving: !walking && age - (offstage - spot) < 16,
    })
  }
  return out
}

/**
 * The cat, when this forge has it: in from the yard and up on the sill to sleep,
 * at his heels a while, off when the furnace roars, out again when it moves on.
 */
function catNow(): World['cat'] {
  const c = anim.cat
  if (c.mode === 'gone' || !theme.pet) return undefined
  const { gate, away, follow } = petAt()
  const building = anim.current === 'running' && anim.flavor === 'build'
  if (building && c.mode !== 'flee' && c.mode !== 'depart') {
    if (c.mode === 'roof') c.x = gate // down from the sill
    c.mode = 'flee'
    c.since = anim.clock
  }
  if (!building && c.mode === 'flee' && c.x >= away && !anim.catNext) { c.mode = 'arrive'; c.since = anim.clock } // the roar is over
  if (c.mode === 'follow' && anim.clock - c.since > 200) { c.mode = 'home'; c.since = anim.clock }
  const target = c.mode === 'follow' ? follow + anim.x : c.mode === 'flee' || c.mode === 'depart' ? away : gate
  const pace = c.mode === 'flee' ? 2 : 1
  if (c.mode !== 'roof' && c.x !== target) {
    c.dir = target > c.x ? 1 : -1 // it keeps facing the way it last walked
    c.x += c.x < target ? Math.min(pace, target - c.x) : -Math.min(pace, c.x - target)
    c.steps += 1
    c.movedAt = anim.clock
  }
  if ((c.mode === 'arrive' || c.mode === 'home') && c.x === gate) { c.mode = 'roof'; c.since = anim.clock }
  if ((c.mode === 'flee' || c.mode === 'depart') && c.x >= away && anim.catNext) {
    c.mode = 'gone' // out of the scene: on its way to the next forge
    anim.catHandoff = true
    return undefined
  }
  if (c.mode === 'roof') return { mode: 'roof', x: c.x, age: anim.clock - c.since }
  // moved this tick or the last: walking; else it stands where it is
  const step = anim.clock - c.movedAt <= 1 ? c.steps : undefined
  return { mode: c.mode === 'flee' ? 'flee' : 'walk', x: c.x, age: anim.clock - c.since, dir: c.dir, step }
}

/**
 * The machine has one cat; the store says which forge has it. Nobody's (or its
 * forge closed): the first forge at work takes it in. This forge's: it stays a
 * while, then goes round to the next forge, or flees there when the furnace roars.
 */
async function catPoll($: EngineInterface) {
  if (!theme.pet) return // a theme without a pet leaves it to the others
  const { gate, away } = petAt()
  const rec = (await $.store.get(CAT_KEY)) as CatRecord | undefined
  const others = anim.neighbours.filter(n => !n.away)
  const alive = (id: string) => id === anim.sessionId || anim.neighbours.some(n => n.id === id)
  const c = anim.cat
  if (!rec || !alive(rec.host)) {
    if ([anim.sessionId, ...others.map(n => n.id)].sort()[0] === anim.sessionId) {
      await $.store.set(CAT_KEY, { host: anim.sessionId, since: anim.nowMs } satisfies CatRecord)
      Object.assign(c, { mode: 'arrive', x: away, since: anim.clock })
      await note($, words().notes.petIn)
    }
    return
  }
  if (rec.host !== anim.sessionId) {
    if (c.mode !== 'gone') c.mode = 'gone' // another forge has it now
    anim.catNext = undefined
    return
  }
  if (anim.catHandoff && anim.catNext) {
    const next = anim.catNext
    await $.store.set(CAT_KEY, { host: next.id, since: anim.nowMs, from: anim.project, fromName: anim.name } satisfies CatRecord)
    anim.catHandoff = false
    anim.catNext = undefined
    await note($, fill(words().notes.petTo, { who: next.label }))
    return
  }
  if (c.mode === 'gone') {
    Object.assign(c, { mode: 'arrive', x: away, since: anim.clock })
    await note($, rec.fromName ? fill(words().notes.petFrom, { who: rec.fromName })
      : rec.from ? fill(words().notes.petFromPlace, { project: rec.from }) : words().notes.petIn)
    return
  }
  // time to move on (it stays eight to fifteen minutes), or fleeing the roar to another forge
  const stay = (8 + (hashOf(String(rec.since)) % 8)) * 60 * 1000
  if (others.length > 0 && !anim.catNext && ((c.mode === 'roof' && anim.nowMs - rec.since > stay) || c.mode === 'flee')) {
    const ids = [anim.sessionId, ...others.map(n => n.id)].sort()
    const next = others.find(n => n.id === ids[(ids.indexOf(anim.sessionId) + 1) % ids.length])
    if (next) {
      anim.catNext = { id: next.id, label: forgeLabel(next, false) }
      if (c.mode === 'roof') Object.assign(c, { mode: 'depart', x: gate, since: anim.clock })
    }
  }
}

/** He sets off to another session's forge: out through the yard first. */
async function startTrip($: EngineInterface, host: Presence & { id: string }, voice: string) {
  anim.trip = { phase: 'out', host: host.id, project: host.project, name: host.name ?? host.project, until: 0 }
  await note($, fill(words().notes.tripOut, { who: forgeLabel(host) }))
  if (voice !== 'aucune') await say($, pick('leave', forgeLabel(host, false)), true)
}

/** Out of sight: he is at the host's now, which its session learns from the store. */
async function arriveAway($: EngineInterface) {
  const trip = anim.trip
  if (!trip || trip.phase !== 'out') return
  trip.phase = 'away'
  trip.until = anim.tripUntil > anim.nowMs ? anim.tripUntil : anim.nowMs + TRIP_MS
  anim.tripUntil = 0
  const title = anim.title || anim.goal
  const visit: Visit = {
    host: trip.host, until: trip.until, project: anim.project, cape: anim.cape, accessory: anim.accessory, phase: 'there',
    name: anim.name, ...(title ? { title } : {}),
  }
  await $.store.set(VISIT_PREFIX + anim.sessionId, visit)
  await setAct($, 'away', trip.name)
  await heartbeat($)
}

/** The visit is over, or work called him back: he leaves the host and walks home. */
async function endTrip($: EngineInterface, idle: boolean) {
  const trip = anim.trip
  if (!trip) return
  if (trip.phase === 'out') {
    trip.phase = 'back' // not gone yet: he just turns round
    return
  }
  if (trip.phase !== 'away') {
    if (trip.phase === 'leaving' && !idle) trip.idleAfter = false // work came meanwhile
    return
  }
  // he says he is leaving; the host walks him out and answers; then he comes home
  trip.phase = 'leaving'
  trip.leavingSince = anim.clock
  trip.idleAfter = idle
  const key = VISIT_PREFIX + anim.sessionId
  const v = (await $.store.get(key)) as Visit | undefined
  if (v) await $.store.set(key, { ...v, phase: 'leaving' } satisfies Visit)
}

/** The host said he is gone (or kept silent too long): now he walks into his own forge. */
async function comeHome($: EngineInterface) {
  const trip = anim.trip
  if (!trip || trip.phase !== 'leaving') return
  trip.phase = 'back'
  anim.x = theme.stage.offstage
  await $.store.delete(VISIT_PREFIX + anim.sessionId)
  await note($, fill(words().notes.tripBack, { who: trip.name }))
  if (trip.idleAfter) await setAct($, 'idle')
  await heartbeat($)
}

/** While he is leaving: has the host walked him out yet? */
async function awaitHandoff($: EngineInterface) {
  const trip = anim.trip
  if (!trip || trip.phase !== 'leaving') return
  const v = (await $.store.get(VISIT_PREFIX + anim.sessionId)) as Visit | undefined
  if (!v || v.phase === 'gone' || anim.clock - (trip.leavingSince ?? anim.clock) > HANDOFF_WAIT) await comeHome($)
}

/** Whether another session's Scorpheus is visiting this forge: one key per visitor. */
async function pollVisits($: EngineInterface, voice: string) {
  // guests walked out since the last look: tell their sessions they are gone
  for (const id of anim.hostAcks.splice(0)) {
    const v = (await $.store.get(VISIT_PREFIX + id)) as Visit | undefined
    if (v && v.host === anim.sessionId) await $.store.set(VISIT_PREFIX + id, { ...v, phase: 'gone' } satisfies Visit)
  }
  const keys = await $.store.keys()
  const here = new Map<string, Visit>()
  for (const key of keys) {
    if (key.startsWith(VISIT_PREFIX) && key !== VISIT_PREFIX + anim.sessionId) {
      const v = (await $.store.get(key)) as Visit | undefined
      // the visitor decides when he leaves; this forge's clock only guards against a session that died
      const staying = v && v.host === anim.sessionId && (v.phase ?? 'there') === 'there' && v.until + 60_000 > anim.nowMs
      if (staying) here.set(key.slice(VISIT_PREFIX.length), v)
      const id = key.slice(VISIT_PREFIX.length)
      if (v && v.host === anim.sessionId && v.phase === 'leaving' && !anim.guests.has(id)) anim.hostAcks.push(id) // never came in here
    }
    // news by raven from the other forges, newer than this session and not seen yet
    if (key.startsWith(NEWS_PREFIX) && key !== NEWS_PREFIX + anim.sessionId) {
      const n = (await $.store.get(key)) as News | undefined
      const sender = key.slice(NEWS_PREFIX.length)
      if (n && n.at > Math.max(anim.startedMs, anim.newsSeen.get(sender) ?? 0)) {
        anim.newsSeen.set(sender, n.at)
        anim.messenger = { kind: n.kind, since: anim.clock }
        // the title first: it is what tells the person where to go
        const who = n.title ? `« ${n.title.length > 48 ? `${n.title.slice(0, 47)}…` : n.title} »` : forgeLabel(n)
        await note($, fill(n.kind === 'done' ? words().notes.newsDone : words().notes.newsWaiting, { who }))
      }
    }
  }
  for (const [id, v] of here) {
    if (anim.guests.has(id) || anim.hostAcks.includes(id)) continue
    anim.guests.set(id, { project: v.project, name: v.name, title: v.title, cape: v.cape, accessory: v.accessory, arrived: anim.clock })
    await note($, fill(words().notes.guestIn, { who: forgeLabel(v) }))
    if (anim.current === 'idle' && !anim.trip) {
      await setAct($, 'waving', '', 2000)
      if (voice !== 'aucune') await say($, pick('welcome', forgeLabel(v, false)), true)
    }
  }
  for (const [id, g] of anim.guests) {
    if (here.has(id) || g.leaving !== undefined) continue
    g.leaving = anim.clock
    // a guest leaves something behind
    const kinds = ['coin', 'cookie', 'ore'] as const
    const kind = kinds[hashOf(id + g.arrived) % kinds.length] ?? 'coin'
    anim.gift = { kind, since: anim.clock }
    await note($, fill(words().notes.guestOut, { who: forgeLabel(g, false), gift: words().notes.gifts[kind] }))
  }
}

/** Sends the other forges news by raven, under this session's own key. */
async function sendNews($: EngineInterface, kind: News['kind']) {
  // the conversation's title, else the task Haiku named: the project alone tells four conversations apart from none
  const title = anim.title || anim.goal
  const n: News = { kind, project: anim.project, name: anim.name, ...(title ? { title } : {}), at: anim.nowMs }
  await $.store.set(NEWS_PREFIX + anim.sessionId, n)
}

/** The conversation's title, when Claude Code gives one: kept, for the ravens and a reload. */
async function learnTitle($: EngineInterface, title: string | undefined) {
  const t = title?.trim()
  if (!t || t === anim.title) return
  anim.title = t
  if (anim.sessionId) await $.store.set(TITLE_PREFIX + anim.sessionId, t).catch(() => undefined)
}

/** Three forges idle at once, none feasted today: the lowest id holds the banquet; the others go. */
async function maybeBanquet($: EngineInterface) {
  const today = dateKey(anim.nowMs)
  const b = (await $.store.get(BANQUET_KEY)) as Banquet | undefined
  if (b && b.date === today && b.until > anim.nowMs) {
    if (b.host === anim.sessionId) {
      anim.banquetUntil = b.until
    } else if (anim.banquetGone !== `${b.date}:${b.host}` && !anim.trip && anim.current === 'idle' && !anim.isWorking) {
      const host = anim.neighbours.find(n => n.id === b.host)
      if (host) {
        anim.banquetGone = `${b.date}:${b.host}`
        anim.trip = { phase: 'out', host: host.id, project: host.project, name: host.name ?? host.project, until: 0 }
        anim.tripUntil = b.until
        await note($, fill(words().notes.feastOut, { who: forgeLabel(host) }))
      }
    }
    return
  }
  if (b && b.date === today) return // one feast a day
  const ready = (p: { idleSince?: number; away?: boolean }) => p.idleSince !== undefined && !p.away && anim.nowMs - p.idleSince >= BANQUET_IDLE_MS
  const idle = [
    ...(ready({ idleSince: anim.idleSince }) ? [anim.sessionId] : []),
    ...anim.neighbours.filter(ready).map(n => n.id),
  ].sort()
  if (idle.length >= 3 && idle[0] === anim.sessionId) {
    const feast: Banquet = { date: today, host: anim.sessionId, until: anim.nowMs + BANQUET_MS }
    await $.store.set(BANQUET_KEY, feast)
    anim.banquetUntil = feast.until
    await note($, words().notes.feast)
  }
}

/** The day's habits, when he is idle: coffee early on, a bite at half past twelve (the person's break), the lantern at dusk. */
async function routine($: EngineInterface) {
  const now = new Date(anim.nowMs)
  const hour = now.getHours() + now.getMinutes() / 60
  const today = dateKey(anim.nowMs)
  if (hour >= 7 && hour < 19) anim.lanternLit = false
  if (hour >= 19.5 || hour < 6) anim.lanternLit = anim.lanternLit || anim.current !== 'idle' // busy at dusk: lit anyway
  if (anim.current !== 'idle' || anim.trip || (anim.routine && anim.routine.until > anim.clock)) return
  if (!anim.coffeeDone && hour < 11 && anim.clock < 20 * 60 * 1000 / TICK_MS) {
    anim.coffeeDone = true
    anim.routine = { kind: 'coffee', until: anim.clock + 400 }
    await note($, words().notes.coffee)
  } else if (hour >= 12.5 && hour < 14 && anim.lunchDone !== today) {
    anim.lunchDone = today
    anim.routine = { kind: 'lunch', until: anim.clock + 800 }
    await note($, words().notes.lunch)
  } else if ((hour >= 19 || hour < 6) && !anim.lanternLit) {
    anim.routine = { kind: 'lantern', until: anim.clock + 140 }
  }
}

// --- the task's status: what the person asked, where it stands -------------------

const GOAL_SYSTEM =
  "Tu donnes le titre de la tâche qu'un développeur confie à son assistant de code : 10 mots au plus, " +
  "commençant par un verbe à l'infinitif, sans guillemets ni point final. Si la nouvelle demande prolonge la tâche " +
  'en cours (« vas-y », « ok », « continue », une précision), garde ce titre ou ajuste-le. Réponds par le titre seul.'
const PROGRESS_SYSTEM =
  "Tu suis le travail d'un assistant de code sur une tâche. D'après ses dernières actions, dis où il en est " +
  "en 2 phrases courtes, 30 mots au plus, au présent : ce qui est déjà fait dans ce tour, puis l'étape en cours, " +
  'concrète (fichiers, tests, recherche). Sans guillemets ni liste. Réponds par les phrases seules.'
const DONE_SYSTEM =
  "Tu résumes la fin d'un tour de travail d'un assistant de code, en 2 ou 3 phrases courtes, 40 mots au plus : " +
  "ce qui est fait, puis ce qui reste ou ce qu'il attend de la personne, s'il y a lieu. Sans guillemets ni liste. " +
  'Réponds par les phrases seules.'

/** A model's or a person's text as plain words on one run, or undefined when nothing is left. */
const oneLine = (text: string | undefined, max: number) =>
  text?.replace(/[*`#]/g, '').replace(/^\s*[-•]\s+/gm, '').replace(/\s*\n+\s*/g, ' ').trim()
    .replace(/^[«"'\s]+|[»"'\s]+$/g, '').replace(/\.$/, '').slice(0, max) || undefined

/** Words laid out on `max` lines of `width` columns at most; a cut last line ends in an ellipsis. */
function wrapWords(text: string, width: number, max: number): string[] {
  if (max <= 0) return []
  const lines: string[] = []
  let current = ''
  for (const word of text.split(/\s+/).filter(Boolean)) {
    const next = current ? `${current} ${word}` : word
    if (next.length <= width) { current = next; continue }
    if (current) lines.push(current)
    current = word.length > width ? word.slice(0, width) : word
    if (lines.length === max) break
  }
  if (lines.length < max && current) lines.push(current)
  if (lines.length > max || (lines.length === max && lines.join(' ').length < text.trim().length - 1)) {
    lines.length = max
    const last = lines[max - 1] ?? ''
    lines[max - 1] = `${last.length >= width ? last.slice(0, width - 1) : last}…`
  }
  return lines
}

/** A tool call as the status reads it: a command by its description, the plan by its current step. */
function actionOf(tool: string, input: Record<string, unknown>): string {
  if (tool === 'TodoWrite' && Array.isArray(input.todos)) {
    const todos = input.todos as Array<{ content?: string; status?: string; activeForm?: string }>
    const current = todos.find(t => t.status === 'in_progress')
    const done = todos.filter(t => t.status === 'completed').length
    return `plan ${done}/${todos.length}${current ? ` : ${current.activeForm ?? current.content ?? ''}` : ''}`
  }
  if (typeof input.command === 'string') {
    return `${tool} : ${typeof input.description === 'string' ? input.description : input.command.slice(0, 60)}`
  }
  return describe(tool, input)
}

async function setStatus($: EngineInterface, change: Partial<Status>) {
  await update($, status, s => ({ goal: anim.goal, line: '', at: anim.nowMs, done: false, ...(s ?? {}), ...change }))
}

/** A request from the person: the task it sets, in a few words. A bare "go on" keeps the task. */
async function newGoal($: EngineInterface, text: string, mode: string) {
  if (mode === 'aucun') return
  const seq = ++anim.goalSeq
  anim.lineSeq += 1 // a line still coming about the last turn is stale
  anim.doings = []
  anim.sinceStatus = 0
  anim.statusAt = anim.nowMs - (STATUS_EVERY_MS - STATUS_FIRST_MS)
  const plain = oneLine(text, 120) ?? ''
  const fallback = plain.length >= 12 || !anim.goal ? plain : anim.goal
  await setStatus($, { goal: fallback, line: '', at: anim.nowMs, done: false })
  const named = mode === 'haiku'
    ? oneLine(await askHaiku($, GOAL_SYSTEM, `Tâche en cours : ${anim.goal || 'aucune'}\nNouvelle demande : ${text.slice(0, 1500)}`, 40), 120)
    : undefined
  if (seq !== anim.goalSeq) return
  anim.goal = named ?? fallback
  await setStatus($, { goal: anim.goal })
}

/** A tool call on the main thread: where the task stands, from Haiku every two minutes at most. */
async function noteAction($: EngineInterface, action: string, mode: string) {
  anim.doings.push(action)
  if (anim.doings.length > 40) anim.doings.splice(0, anim.doings.length - 40)
  anim.sinceStatus += 1
  if (mode === 'aucun') return
  if (mode === 'simple') return setStatus($, { line: action, at: anim.nowMs, done: false })
  if (anim.nowMs - anim.statusAt < STATUS_EVERY_MS || anim.sinceStatus < 3) return
  anim.statusAt = anim.nowMs
  anim.sinceStatus = 0
  const seq = ++anim.lineSeq
  const line = oneLine(await askHaiku($, PROGRESS_SYSTEM,
    `Tâche : ${anim.goal || '(inconnue)'}\nDernières actions, la plus récente en bas :\n${anim.doings.slice(-20).join('\n')}`, 110), 300)
  if (seq !== anim.lineSeq || !line) return
  await setStatus($, { line, at: anim.nowMs, done: false })
}

/** The turn's answer: what is done, what is left. */
async function finishStatus($: EngineInterface, answer: string, mode: string) {
  if (mode === 'aucun') return
  const seq = ++anim.lineSeq
  // the head says what was done, the tail what is left: both, when the answer is long
  const excerpt = answer.length > 2000 ? `${answer.slice(0, 1200)}\n[…]\n${answer.slice(-800)}` : answer
  const summary = mode === 'haiku'
    ? oneLine(await askHaiku($, DONE_SYSTEM, `Tâche : ${anim.goal || '(inconnue)'}\nRéponse finale de l'assistant :\n${excerpt}`, 140), 300)
    : undefined
  if (seq !== anim.lineSeq) return
  await setStatus($, { line: summary ?? oneLine(answer, 300) ?? 'tour terminé', at: anim.nowMs, done: true })
}

/** What he dreams of, once a conversation, from how it began. */
async function chooseDream($: EngineInterface) {
  if (anim.dream) return
  const motifs = Object.keys(DREAMS) as Dream[]
  anim.dream = motifs[hashOf(anim.sessionId) % motifs.length] // while Haiku thinks
  const answer = await askHaiku($,
    // "a smith's dream" drew Haiku to the gear whatever the subject: it is told to look at the subject alone
    'Choisis le motif qui évoque le mieux le SUJET de cette conversation (pas le métier de celui qui rêve). ' +
    `Motifs et ce qu'ils évoquent :\n${motifs.map(m => `${m} : ${DREAM_HINTS[m]}`).join('\n')}\nRéponds par le mot seul.`,
    `Projet : ${anim.project}\n${anim.opening.slice(0, 400)}`, 8)
  const word = answer?.toLowerCase().replace(/[^a-z]/g, '') as Dream | undefined
  if (word && motifs.includes(word)) anim.dream = word
}

/** Idle a while, nobody working beside him, another forge at work: he may go and see it. */
async function maybeTrip($: EngineInterface, voice: string) {
  if (anim.trip || anim.guests.size > 0 || anim.isWorking || anim.current !== 'idle') return
  if (anim.apprentices > 0 || anim.jobs > 0 || theme.anchorOf('idle', worldNow()) !== undefined) return
  if (anim.clock < (anim.lastTrip < 0 ? TRIP_FIRST_TICKS : anim.lastTrip + TRIP_AGAIN_TICKS)) return
  const hosts = anim.neighbours.filter(n => !n.away)
  if (hosts.length === 0 || chance((anim.clock + anim.phase) * 3) >= 0.08) return
  const host = hosts[Math.floor(chance(anim.clock + anim.phase * 5) * hosts.length)]
  if (host) await startTrip($, host, voice)
}

/** A run of failures brings the storm, a run of successes a rainbow for two minutes. */
function streak(outcome: 'success' | 'failure') {
  if (outcome === 'success') {
    anim.successStreak += 1
    anim.failStreak = 0
    if (anim.successStreak === 3) anim.rainbowUntil = anim.clock + RAINBOW_TICKS
  } else {
    anim.failStreak += 1
    anim.successStreak = 0
    anim.rainbowUntil = -1
  }
}

async function syncClock($: EngineInterface) {
  anim.nowMs = await $.clock.now()
}

/** A request to the model on the main thread: he thinks, with that model's hammer. */
async function noteStep($: EngineInterface, model: string, effort: string | number | undefined) {
  anim.hammer = hammerOf(model)
  anim.effort = effortOf(effort)
  await setAct($, 'thinking', 'réfléchit')
}

const fallback = (): Pending =>
  anim.isWorking
    ? { activity: 'thinking', detail: 'réfléchit', flavor: 'forge' }
    : { activity: 'idle', detail: '', flavor: 'forge' }

/**
 * Puts Scorpheus in a moment. A one-shot (`holdMs`) plays to its end: plain work
 * asked meanwhile waits and follows it. Waiting on the person never waits.
 */
async function setAct($: EngineInterface, activity: Activity, detail = '', holdMs = 0, flavor: Flavor = 'forge') {
  if (anim.trip && activity !== 'idle' && activity !== 'away' && activity !== 'waving') await endTrip($, false) // work calls him home
  const playing = anim.revert !== undefined && ONE_SHOTS.has(anim.current)
  if (playing && !ONE_SHOTS.has(activity) && activity !== 'waiting' && activity !== 'sweep') {
    anim.pending = { activity, detail, flavor }
    return
  }
  anim.revert?.cancel()
  anim.revert = undefined
  if (activity !== anim.current) anim.tick = 0
  anim.current = activity
  anim.flavor = flavor
  if (activity !== 'idle' && activity !== 'sleep') anim.lastBusy = anim.clock
  await update($, act, () => ({ activity, detail, flavor }))
  if (holdMs > 0) {
    anim.revert = $.clock.after(holdMs, () => {
      anim.revert = undefined
      const next = anim.pending ?? fallback()
      anim.pending = undefined
      void setAct($, next.activity, next.detail, 0, next.flavor).catch(() => undefined)
    })
  }
}

/** The task ids and tool_use_ids a `<task-notification>` names. */
function taskIdsIn(text: string): string[] {
  const ids: string[] = []
  for (const m of text.matchAll(/<(task-id|tool-use-id)>\s*([^<\s]+)\s*</g)) if (m[2]) ids.push(m[2])
  return ids
}

/**
 * A task is over: the apprentice or the command it names leaves, whichever id
 * names it (tool_use_id, agent id, task id). A background one rings the bell.
 */
async function finishTasks($: EngineInterface, ids: readonly string[]) {
  if (ids.length === 0) return
  const named = new Set(ids)
  let rang = false
  await setCrew($, c => {
    const apprentices = c.apprentices.filter(a => {
      const done = named.has(a.id) || (a.agentId !== undefined && named.has(a.agentId))
      if (done && a.background) rang = true
      return !done
    })
    const jobs = c.jobs.filter(j => {
      const done = named.has(j.id) || (j.toolUseId !== undefined && named.has(j.toolUseId))
      if (done) rang = true
      return !done
    })
    return { apprentices, jobs }
  })
  if (rang) anim.bellAt = anim.clock
}

/** What the engine says of the subagents: the ones it no longer runs leave. */
async function reconcileApprentices($: EngineInterface) {
  if (anim.apprentices === 0) return
  const over = new Set(['completed', 'failed', 'killed'])
  const ended = (await $.agent.list()).filter(a => over.has(a.status)).map(a => a.id)
  await finishTasks($, ended)
}

/** Writes who works beside him, for the words, and counts them for the scene. */
async function setCrew($: EngineInterface, change: (c: Crew) => Crew) {
  const c = await update($, crew, change)
  const ids = new Set(c.apprentices.map(a => a.id))
  for (const [id, cub] of anim.cubs) {
    if (!ids.has(id)) {
      anim.cubs.delete(id)
      anim.leaving.push({ ...cub, left: anim.clock })
      const gone = anim.names.get(id)
      if (gone) void note($, fill(words().notes.helperOut, { name: gone })).catch(() => undefined)
    }
  }
  for (const a of c.apprentices) if (a.name) anim.names.set(a.id, a.name)
  for (const a of c.apprentices) {
    if (!anim.cubs.has(a.id)) anim.cubs.set(a.id, { seed: hashOf(a.id) % 9973, colour: colourOf(a.type), born: anim.clock })
  }
  anim.apprentices = c.apprentices.length
  anim.jobs = c.jobs.length
}

/**
 * Loads the theme the `theme` option names: the forge, a theme shipped in
 * themes/, or a JSON file by its path. A theme that does not load leaves the
 * forge in place, and says why in the log.
 */
async function useTheme($: EngineInterface, wanted: string) {
  anim.themeWanted = wanted
  const file = themeFile($.plugin.root, wanted)
  let errors: string[] = []
  if (file === undefined) {
    theme = forge
    anim.themeStamp = undefined
  } else {
    try {
      anim.themeStamp = (await $.fs.stat(file)).mtimeMs
      const made = themeFrom(await $.fs.read(file), wanted)
      theme = made.theme
      errors = made.errors
    } catch {
      theme = forge
      errors = [`fichier introuvable : ${file}`]
    }
  }
  if (errors.length > 0) await note($, `thème « ${wanted} » refusé : ${errors[0]}${errors.length > 1 ? ` (+${errors.length - 1})` : ''}`)
}

/** A theme file changed on disk: drawn again at once, so its author sees it. */
async function watchTheme($: EngineInterface) {
  const file = themeFile($.plugin.root, anim.themeWanted)
  if (file === undefined) return
  const stamp = await $.fs.stat(file).then(s => s.mtimeMs, () => undefined)
  if (stamp === undefined || stamp === anim.themeStamp) return
  await useTheme($, anim.themeWanted)
  $.ui.invalidate('ui.render')
}

export const register: Register = (on, options) => {
  const allowed: Crop | undefined = DECOR_OF[String(options.decor)]
  const themeWanted = String(options.theme ?? 'forge').trim() || 'forge'
  const voice = String(options.repliques ?? 'local')
  const statusMode = String(options.statut ?? 'haiku')

  on('session.start', async ($, e, next) => {
    const r = await next(e)
    if (!e.isInteractive) return r

    for (const t of anim.timers) t.cancel()
    await useTheme($, themeWanted)
    const now = await $.session.usage()
    const u = toUsage(now, now.startedAt)
    learn(u)
    await update($, usage, () => u)
    // a reload starts the module over: the apprentices still at work come from the engine's list
    const live = await $.agent.list().catch(() => [])
    await setCrew($, () => ({
      apprentices: live.filter(a => a.status === 'running')
        .map(a => ({ id: a.id, agentId: a.id, type: a.type, description: a.description, background: true })),
      jobs: [],
    }))
    await syncClock($)
    anim.hammer = hammerOf(await $.session.model())
    anim.project = short(await $.session.cwd())
    anim.banner = bannerOf(anim.project)
    // a host without session ids must not cost him his start: the project and the hour stand in
    anim.sessionId = await $.session.id().catch(() => `${anim.project}-${anim.nowMs}`)
    anim.phase = hashOf(anim.sessionId) % 9973
    anim.title = String((await $.store.get(TITLE_PREFIX + anim.sessionId)) ?? anim.title)
    {
      // a session resumed keeps the look it had; a new one takes its hash's until Haiku chooses
      // (a look this theme does not have is not kept: a theme changed since)
      const stored = ((await $.store.get(PRESENCE_KEY)) as Record<string, Presence> | undefined)?.[anim.sessionId]
      const { variants, accessories } = theme.looks
      const known = stored && variants[stored.cape] && accessories.includes(stored.accessory) ? stored : undefined
      const keys = Object.keys(variants)
      anim.cape = known?.cape ?? keys[hashOf(anim.sessionId) % keys.length] ?? 'foret'
      anim.accessory = known?.accessory ?? accessories[hashOf(anim.sessionId + '!') % accessories.length] ?? 'aucun'
      anim.styled = known !== undefined
      anim.personality = known?.personality ?? PERSONALITIES[hashOf(anim.sessionId + '?') % PERSONALITIES.length] ?? 'calme'
      const names = theme.names
      anim.name = (known?.name && names.includes(known.name) ? known.name : undefined) ?? names[hashOf(anim.sessionId + '#') % names.length] ?? anim.name
    }
    anim.startedMs = anim.nowMs
    anim.masterpiece = (await $.store.get(`master:${anim.sessionId}`)) !== undefined
    await heartbeat($)
    await flushWeek($)
    const hour = new Date(anim.nowMs).getHours()
    if (voice !== 'aucune') await say($, pick(hour >= 5 && hour < 12 ? 'morning' : hour >= 18 ? 'evening' : 'afternoon'), true)
    await setAct($, 'waving', '', 3000)

    // every task started and not awaited catches its own failure: a store write cut short by a
    // reload, or a session ending, must not surface as an error nobody handled
    anim.timers = [
      $.clock.every(TICK_MS, () => {
        anim.tick += 1
        anim.clock += 1
        anim.nowMs += TICK_MS
        if (anim.current !== 'idle' && anim.current !== 'sleep' && anim.current !== 'waving' && anim.current !== 'away') anim.today.ticks += 1
        if (anim.clock === TEA_TICKS && !anim.saidTea) {
          anim.saidTea = true
          $.ui.toast(`${anim.name} : ${words().teaToast}`)
          if (voice !== 'aucune') void say($, pick('tea'), true).catch(() => undefined)
        }
        // he walks toward where the moment wants him, or where he fancies
        steer()
        if (anim.trip?.phase === 'out' && anim.x >= theme.stage.offstage) void arriveAway($).catch(() => undefined)
        if (anim.trip?.phase === 'away' && anim.nowMs >= anim.trip.until) void endTrip($, true).catch(() => undefined)
        if ((anim.clock + anim.phase) % HANDOFF_TICKS === 0) {
          void awaitHandoff($).catch(() => undefined)
          void catPoll($).catch(() => undefined)
          void pollVisits($, voice).catch(() => undefined)
          void watchTheme($).catch(() => undefined)
        }
        if ((anim.clock + anim.phase) % POLL_TICKS === 0) {
          // idle since when: three forges idle at once make a banquet
          const free = anim.current === 'idle' && !anim.isWorking && !anim.trip && anim.apprentices === 0 && anim.jobs === 0
          anim.idleSince = free ? anim.idleSince ?? anim.nowMs : undefined
          void maybeBanquet($).catch(() => undefined)
          void maybeTrip($, voice).catch(() => undefined)
          void routine($).catch(() => undefined)
          // the cat sometimes comes down to follow him about: when he strolls, not when work calls him
          if (anim.cat.mode === 'roof' && anim.current === 'idle' && anim.x !== anim.target && chance(anim.clock + anim.phase * 13) < 0.15) {
            anim.cat.mode = 'follow'
            anim.cat.since = anim.clock
          }
          // a rare sight in the window, about once an hour: a dragon by day, a shooting star in the
          // evening; none once the shutters are closed (23h to 6h), nobody would see it
          const hour = new Date(anim.nowMs).getHours()
          const window = anim.crop !== undefined && theme.sky(anim.crop) // a scene without the sky shows no rare sight
          if (!anim.rare && window && hour >= 6 && hour < 23 && chance((anim.clock + anim.phase) * 11) < RARE_ODDS) {
            anim.rare = { kind: hour >= 21 ? 'star' : 'dragon', since: anim.clock }
            // he points it out: a dragon a few pixels wide is missed unless someone says so
            if (voice !== 'aucune') void say($, pick(anim.rare.kind), true).catch(() => undefined)
          }
        }
        if (anim.rare && anim.clock - anim.rare.since > theme.rareFor) anim.rare = undefined
        if (anim.routine?.kind === 'lantern' && anim.x === theme.stage.roam.min && !anim.lanternLit) {
          anim.lanternLit = true
          anim.routine = undefined
          void note($, words().notes.lantern).catch(() => undefined)
        }
        // ten quiet minutes with nobody working beside him: a nap
        if (anim.current === 'idle' && anim.apprentices === 0 && anim.jobs === 0 && anim.clock - anim.lastBusy >= SLEEP_TICKS) {
          void setAct($, 'sleep').catch(() => undefined)
          void chooseDream($).catch(() => undefined)
        }
        if (!anim.bandId || !anim.crop) return
        // a band not mounted (a survey, a narrow terminal) answers deny or rejects: nothing to do
        $.ui.blit({ requestId: anim.bandId, key: KEY, cells: encode(theme.frame(anim.crop, anim.current, poseTick(), worldNow())) })
          .catch(() => undefined)
      }),
      // the countdowns move by the minute; the local time is set again
      $.clock.every(30_000, () => {
        void syncClock($).catch(() => undefined)
        void flushWeek($).catch(() => undefined)
        void heartbeat($).catch(() => undefined)
        void reconcileApprentices($).catch(() => undefined)
        $.ui.invalidate('ui.render')
      }),
    ]
    return r
  })

  on('prompt.submit', async ($, e, next) => {
    // a task left in the background came back: the one its ids name leaves
    if (e.origin.kind === 'task-notification') await finishTasks($, taskIdsIn(e.text))
    if (!anim.isWorking) {
      anim.turnStart = anim.clock
      anim.turnPassed = false
    }
    if (e.origin.kind === 'composer' && !anim.opening) anim.opening = e.text
    if (e.origin.kind === 'composer') void newGoal($, e.text, statusMode).catch(() => undefined)
    void $.store.delete(NEWS_PREFIX + anim.sessionId).catch(() => undefined) // whatever we waited for, the person is here
    anim.isWorking = true
    if (!anim.styled && e.origin.kind === 'composer') void chooseLook($, e.text).catch(() => undefined)
    if (anim.current === 'sleep') {
      await setAct($, 'waking', '', 1000)
      if (voice !== 'aucune') await say($, pick('wake'), true)
    }
    const hour = new Date(anim.nowMs).getHours()
    if ((hour >= 23 || hour < 5) && !anim.saidNight && voice !== 'aucune') {
      anim.saidNight = true
      await say($, pick('night'), true)
    }
    await setAct($, 'thinking', 'réfléchit')
    return next(e)
  })

  // A notification queued while a turn runs is delivered into it as an
  // attachment, never as a prompt: most background agents end that way.
  on('prompt.attachment', async ($, e, next) => {
    if (e.text.includes('<task-notification>')) await finishTasks($, taskIdsIn(e.text))
    return next(e)
  })

  // Each request to the model: on the main thread, he thinks until a tool runs.
  on('turn.step', async function* ($, e, next) {
    if (e.agentId === undefined) await noteStep($, e.model, e.effort)
    return yield* next(e)
  })

  on('agent.spawn', async ($, e, next) => {
    const r = await next(e)
    if (r.deny === undefined) {
      const apprentice = { id: e.tool_use_id, agentId: r.agentId, type: e.subagentType, description: e.description, background: e.background }
      await setCrew($, c => ({ ...c, apprentices: [...c.apprentices.filter(a => a.id !== apprentice.id), apprentice] }))
      void nameCub($, apprentice.id, apprentice.type, apprentice.description).catch(() => undefined)
    }
    return r
  })

  on('session.compact', async ($, e, next) => {
    if (e.trigger === 'precompute') return next(e)
    await setAct($, 'sweep', words().details.compact)
    await note($, words().notes.compact)
    const r = await next(e)
    const after = fallback()
    await setAct($, after.activity, after.detail)
    return r
  })

  // The conversation's title rides on these two classic events only
  on('classic.SessionStart', async ($, e, next) => {
    await learnTitle($, e.session_title)
    return next(e)
  })
  on('classic.UserPromptSubmit', async ($, e, next) => {
    await learnTitle($, e.session_title)
    return next(e)
  })

  // The person is needed only when Claude Code tells them so. A `tool.check`
  // answering `ask` is not that: in auto mode the classifier takes the ask
  // and approves it silently, and the "!" stayed up over a plain command.
  on('classic.Notification', async ($, e, next) => {
    const why = NEEDS_YOU[e.notification_type]
    if (why !== undefined) {
      await setAct($, 'waiting', why)
      await sendNews($, 'waiting').catch(() => undefined) // a raven tells the other forges
    }
    return next(e)
  })

  on('tool.call', async ($, e, next) => {
    const input = e as unknown as Record<string, unknown>
    const flavor = flavorOf(e.tool, input)
    const command = typeof input.command === 'string' ? input.command : ''
    if (typeof input.agentId === 'string') {
      // an apprentice's own tool: the day counts it; Scorpheus follows the main thread alone
      const r = await next(e)
      if (r.deny === undefined && r.isError !== true) {
        if (flavor === 'test') anim.today.tests += 1
        else if (flavor === 'build') anim.today.builds += 1
        else if (flavor === 'shell') anim.today.shells += 1
        else if (EDITS.has(e.tool)) anim.today.edits += 1
      }
      return r
    }
    const activity: Activity = ASKS.has(e.tool) ? 'waiting' : READS.has(e.tool) ? 'review' : 'running'
    await setAct($, activity, describe(e.tool, input), 0, flavor)
    void noteAction($, actionOf(e.tool, input), statusMode).catch(() => undefined)

    const r = await next(e)
    const ran = r.deny === undefined
    if (ran && r.isError !== true) {
      if (flavor === 'test') anim.today.tests += 1
      else if (flavor === 'build') anim.today.builds += 1
      else if (flavor === 'shell') anim.today.shells += 1
      else if (EDITS.has(e.tool)) anim.today.edits += 1
      await gain($, 1 + (EDITS.has(e.tool) ? 1 : 0) + (flavor === 'build' ? 3 : 0))
    }

    if (e.tool === 'TaskStop' && ran && typeof input.task_id === 'string') await finishTasks($, [input.task_id])
    // an apprentice working in the foreground is back with its answer
    if (e.tool === 'Agent' && e.tool_use_id) {
      const id = e.tool_use_id
      await setCrew($, c => ({ ...c, apprentices: c.apprentices.filter(a => a.id !== id || a.background) }))
    }
    // a command left running: an hourglass on the anvil until its notification
    if (SHELLS.has(e.tool) && input.run_in_background === true && ran && r.isError !== true) {
      const said = typeof r.text === 'string' ? r.text : JSON.stringify(r.result ?? '')
      const job: Job = { id: /with ID:\s*([A-Za-z0-9_-]+)/.exec(said)?.[1] ?? e.tool_use_id ?? command, toolUseId: e.tool_use_id, label: firstWord(command) }
      await setCrew($, c => ({ ...c, jobs: [...c.jobs, job] }))
      return r
    }

    if (ran && flavor === 'test') {
      anim.outcome = r.isError === true ? 'fail' : 'pass'
      streak(anim.outcome === 'pass' ? 'success' : 'failure')
      if (anim.outcome === 'pass') {
        await gain($, 3)
        anim.turnPassed = true
      }
      await note($, fill(anim.outcome === 'pass' ? words().notes.testsPass : words().notes.testsFail, { cmd: firstWord(command) }))
      if (voice !== 'aucune') await say($, pick(anim.outcome === 'pass' ? 'pass' : 'crack'))
      if (anim.failStreak === 3 && voice !== 'aucune') await say($, pick('duck'), true)
      await setAct($, 'quench', anim.outcome === 'pass' ? words().details.pass : words().details.crack, 2600, 'test')
    } else if (ran && r.isError !== true && (COMMIT.test(command) || PUSH.test(command))) {
      anim.ship = PUSH.test(command) ? 'push' : 'commit'
      streak('success')
      await gain($, 3)
      await note($, anim.ship === 'push' ? words().notes.push : words().notes.commit)
      if (voice !== 'aucune') await say($, pick(anim.ship))
      await setAct($, 'ship', anim.ship, anim.ship === 'push' ? 3400 : 2400, 'git')
    } else if (ran && r.isError === true) {
      streak('failure')
      if (voice !== 'aucune') await say($, pick(anim.failStreak === 3 ? 'duck' : 'failed'), anim.failStreak === 3)
      await setAct($, 'failed', `${e.tool} a échoué`, 2500)
    } else if (anim.current === 'waiting' || !ran) {
      const after = fallback()
      await setAct($, after.activity, after.detail)
    }
    return r
  })

  on('turn.complete', async ($, e, next) => {
    // a subagent's own turn ending is not the work being done, but its apprentice is
    if (e.agentId !== undefined) {
      await finishTasks($, [e.agentId])
      return next(e)
    }
    anim.isWorking = false
    anim.lastBusy = anim.clock
    const turnTicks = anim.turnStart >= 0 ? anim.clock - anim.turnStart : 0
    if (e.reason === 'answer') {
      streak('success')
      await gain($, 2)
      void finishStatus($, e.answer, statusMode).catch(() => undefined)
      if (turnTicks >= LONG_TURN_TICKS) await sendNews($, 'done').catch(() => undefined)
      if (turnTicks >= MASTER_TICKS && anim.turnPassed && !anim.masterpiece) {
        // a long turn that ends with the blade proven: the conversation's masterpiece
        anim.masterpiece = true
        await $.store.set(`master:${anim.sessionId}`, { date: dateKey(anim.nowMs) })
        await note($, words().notes.masterpiece)
      }
      await setAct($, 'jumping', '', 1800)
      if (voice !== 'aucune') void sayAnswer($, e.answer, voice).catch(() => undefined)
      anim.turnStart = -1
    } else if (e.reason === 'aborted') {
      anim.turnStart = -1
      await setAct($, 'idle')
    } else {
      anim.turnStart = -1
      streak('failure')
      await setAct($, 'failed', e.reason === 'refusal' ? 'refus' : 'erreur API', 3000)
    }
    return next(e)
  })

  on('session.measure', async ($, e, next) => {
    const before = await read($, usage)
    const u = toUsage(e, before?.startedAt)
    const wasCold = anim.exhausted
    learn(u)
    if (anim.exhausted && !wasCold) await note($, words().notes.cold)
    if (anim.exhausted && !wasCold && voice !== 'aucune') await say($, pick('cold'), true)
    await update($, usage, () => u)

    for (const l of u.limits) {
      const was = before?.limits.find(b => b.kind === l.kind)?.percent
      const name = l.kind === 'five_hour' ? '5h' : l.kind === 'seven_day' ? '7j' : l.kind
      if (was !== undefined && was - l.percent >= 20) {
        anim.warned.delete(l.kind)
        $.ui.toast(`${anim.name} : fenêtre ${name} réinitialisée`)
        if (!anim.isWorking) await setAct($, 'jumping', `${name} neuve`, 2400)
      } else if (l.percent >= 90 && !anim.warned.has(l.kind)) {
        anim.warned.add(l.kind)
        $.ui.toast(`${anim.name} : ${name} à ${Math.round(l.percent)} %`)
      }
    }
    return next(e)
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const columns = e.props.bodyColumns
    if (e.props.hasSurvey || columns < 24) {
      anim.bandId = undefined
      return next(e)
    }

    const u = await read($, usage)
    const a: Act = await read($, act)
    const team: Crew = await read($, crew)
    const line = await read($, speech)
    const xp: Experience = await read($, experience)
    const now = await $.clock.now()
    const model = await $.session.model()
    // a reached limit outranks the pressure it implies
    const { moods, atWork, labels } = words()
    const label = a.activity === 'idle' && anim.exhausted ? moods.cold
      : a.activity === 'idle' && anim.pressure >= 95 ? moods.breathless
      : a.activity === 'idle' && anim.pressure >= 85 ? moods.gauges
      : a.activity === 'idle' && team.jobs.length > 0 ? moods.hourglass
      : a.activity === 'idle' && anim.failStreak >= 3 ? moods.duck
      : a.activity === 'idle' && anim.clock >= TEA_TICKS ? moods.tea
      : a.activity === 'idle' && new Date(anim.nowMs).getHours() < 5 ? moods.yawn
      : a.activity === 'running' ? atWork[a.flavor ?? 'forge']
      : a.activity === 'review' && a.flavor === 'web' ? atWork.web
      : labels[a.activity]

    const { Box, Text } = $.ui.resolve(e)
    // The sprite needs the terminal's Raster and room beside the words; else the words alone.
    const ROWS = theme.rows
    const crop = e.surface === 'terminal' && e.props.maxRows >= ROWS ? fitCrop(allowed, columns) : undefined
    const sceneColumns = crop ? cropWidth(crop) + 2 : 0
    const room = columns - sceneColumns
    const barWidth = clamp(room - 34, 8, 40)
    const roomy = room >= 40 // the info line under the gauges

    const gauge = (name: string, p: number | undefined, tail: string, key: string, colour?: string) => {
      if (p === undefined) return null
      const filled = Math.round((clamp(p, 0, 100) / 100) * barWidth)
      const c = colour ?? tone(p)
      return (
        <Text key={key} wrap="truncate">
          <Text dimColor>{name.padEnd(4)}</Text>
          <Text color={c}>{'━'.repeat(filled)}</Text>
          <Text color="#3a3e4c">{'━'.repeat(barWidth - filled)}</Text>
          <Text color={c} bold>{` ${String(Math.round(p)).padStart(3)}%`}</Text>
          <Text dimColor>{tail}</Text>
        </Text>
      )
    }

    const limit = (kind: string): Gauge | undefined => u?.limits.find(l => l.kind === kind)
    const tailOf = (g: Gauge | undefined, withDay: boolean) => {
      if (!g?.resetsAt) return ''
      const left = countdown(Date.parse(g.resetsAt) - now)
      return `  ↻ ${left} · ${resetAt(g.resetsAt, now, withDay)}`
    }
    const five = limit('five_hour'), seven = limit('seven_day')
    const ctxTail = u?.tokens !== undefined && u.window
      ? `  ${Math.round(u.tokens / 1000)}k/${Math.round(u.window / 1000)}k` : ''

    const log = await read($, journal)
    const st: Status | null = statusMode === 'aucun' ? null : await read($, status)
    const today = todayNow()
    const minutes = Math.round((today.ticks * TICK_MS) / 60000)
    const tally = words().today
    const todayLine = [
      `aujourd'hui`,
      today.edits ? fill(tally.edits, { n: today.edits }) : '',
      today.tests ? fill(tally.tests, { n: today.tests }) : '',
      today.builds ? fill(tally.builds, { n: today.builds }) : '',
      today.shells ? fill(tally.shells, { n: today.shells }) : '',
      minutes >= 1 ? fill(tally.time, { t: minutes >= 60 ? `${Math.floor(minutes / 60)}h${String(minutes % 60).padStart(2, '0')}` : `${minutes} min` }) : '',
    ].filter(Boolean).join(' · ')
    const hex = (letter: string) => `#${(theme.palette[letter] ?? 0xb07cff).toString(16).padStart(6, '0')}`
    const shown = team.apprentices.slice(0, 3)
    // what fits under the rest: sixteen rows, the log takes what is left
    const more = team.apprentices.length - shown.length
    const used = 7 + 1 + (team.apprentices.length > 0 ? 1 + shown.length + (more > 0 ? 1 : 0) : 0) + (team.jobs.length > 0 ? 1 : 0)
    // the task on two lines at most, where it stands on three; the forge's last two events in what is left
    const free = Math.max(0, ROWS - used)
    const textWidth = Math.max(20, room - 4)
    const goalLines = st ? wrapWords(st.goal || '…', textWidth, Math.min(2, free)) : []
    const stateLines = st?.line ? wrapWords(st.line, textWidth, Math.min(3, free - goalLines.length)) : []
    const logShown = log.slice(-Math.min(2, Math.max(0, free - goalLines.length - stateLines.length)))
    const ago = st && st.line && now - st.at >= 2 * 60_000 ? ` · il y a ${Math.round((now - st.at) / 60_000)} min` : ''
    const lastState = stateLines.length - 1
    const jobs = team.jobs.length === 1
      ? `1 commande · ${team.jobs[0]?.label ?? ''}`
      : `${team.jobs.length} commandes · ${team.jobs.map(j => j.label).join(', ')}`

    const panel = (
      <Box flexDirection="column" justifyContent="center">
        <Text wrap="truncate">
          <Text bold color="#3f80dc">{anim.name}</Text>
          <Text dimColor>{` · ${label}`}</Text>
          {a.detail ? <Text color="#e3a63c">{` · ${a.detail}`}</Text> : null}
        </Text>
        {line ? <Text key="speech" italic color="#e2c8ff" wrap="truncate">{`« ${line} »`}</Text> : <Text> </Text>}
        {gauge('ctx', u?.ctx, ctxTail, 'ctx')}
        {gauge('5h', five?.percent, tailOf(five, false), '5h')}
        {gauge('7j', seven?.percent, tailOf(seven, true), '7j')}
        {gauge('xp', ((xp.xp - xpFor(xp.level)) / (xpFor(xp.level + 1) - xpFor(xp.level))) * 100, `  niv. ${xp.level} · ${xp.xp} xp`, 'xp', '#b07cff')}
        {roomy ? <Text dimColor wrap="truncate">{`    ${[
          anim.project,
          model,
          u?.startedAt !== undefined ? `session ${countdown(now - u.startedAt)}` : '',
          u?.usd !== undefined ? `$${u.usd.toFixed(2)}` : '',
        ].filter(Boolean).join(' · ')}`}</Text> : null}
        {roomy ? <Text key="today" dimColor wrap="truncate">{`    ${todayLine}`}</Text> : null}
        {team.apprentices.length > 0 ? <Text key="apprentices" color="#b07cff">{fill(words().helpers, { n: team.apprentices.length })}</Text> : null}
        {shown.map(p => {
          const cub = anim.cubs.get(p.id)
          const age = cub ? Math.floor(((anim.clock - cub.born) * TICK_MS) / 60000) : 0
          return (
            <Text key={`cub-${p.id}`} wrap="truncate">
              <Text color={hex(colourOf(p.type))}>{'  ● '}</Text>
              <Text bold>{p.name ?? '…'}</Text>
              <Text dimColor>{` · ${p.type} · ${age} min — ${p.description}${p.background ? ' (fond)' : ''}`}</Text>
            </Text>
          )
        })}
        {more > 0 ? <Text key="cubs-more" dimColor>{`  … et ${more} autre${more > 1 ? 's' : ''}`}</Text> : null}
        {team.jobs.length > 0 ? (
          <Text key="jobs" wrap="truncate">
            <Text color="#ffd75e">{'en fond     '}</Text>
            <Text dimColor>{jobs}</Text>
          </Text>
        ) : null}
        {goalLines.map((text, i) => (
          <Text key={`goal-${i}`} wrap="truncate">
            <Text color="#e3a63c">{i === 0 ? '▸ ' : '  '}</Text>
            <Text bold>{text}</Text>
          </Text>
        ))}
        {stateLines.map((text, i) => (
          <Text key={`status-${i}`} wrap="truncate">
            <Text color={st?.done ? '#74e07a' : '#7fd4ff'}>{i > 0 ? '    ' : st?.done ? '  ✓ ' : '  … '}</Text>
            <Text>{text}</Text>
            {i === lastState && text.length + ago.length <= textWidth ? <Text dimColor>{ago}</Text> : null}
          </Text>
        ))}
        {logShown.map((entry, i) => <Text key={`log-${i}`} dimColor italic wrap="truncate">{`  ${entry}`}</Text>)}
      </Box>
    )

    if (e.surface !== 'terminal' || crop === undefined) {
      anim.bandId = undefined
      return panel
    }

    anim.bandId = e.requestId
    anim.crop = crop
    const { Raster } = $.ui.resolve(e)
    return (
      <Box flexDirection="row" gap={2}>
        <Raster key={KEY} columns={cropWidth(crop)} rows={ROWS} cells={encode(theme.frame(crop, anim.current, poseTick(), worldNow()))} />
        {panel}
      </Box>
    )
  })
}
