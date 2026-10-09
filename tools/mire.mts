// Mire d'un thème Diorama, en direct dans un terminal : chaque moment joue à
// son vrai rythme (un tic de 150 ms), en vraies couleurs, comme dans le bandeau.
// Plusieurs thèmes côte à côte, nommés A, B, C, D : pour choisir entre des variantes.
//
//   node tools/mire.mts <theme.json | forge> [autre.json ...] [--recadrage full]
//
// Touches : ← → le moment · m la marche (comme en vrai, à l'arrêt, vers la droite,
// vers la gauche) · h l'heure · espace pause · . un tic en pause · q quitter.
// « Comme en vrai », le personnage se déplace exactement comme le moteur le fait
// pendant ce moment. Node 22.6 ou plus. --test : quelques images, puis sortie.

import { CALM, pace, wander } from '../hooks/world.ts'
import type { Crop, Theme, World } from '../hooks/world.ts'
import { loadTheme } from './charge.mts'
import { MOMENTS, roams } from './moments.mts'

const args = process.argv.slice(2)
const flag = (name: string, fallback: string) => {
  const i = args.indexOf(name)
  return i >= 0 ? args.splice(i, 2)[1] ?? fallback : fallback
}
const testing = args.includes('--test')
const crop = flag('--recadrage', 'full') as Crop
const sources = args.filter(a => a !== '--test')
if (sources.length === 0 || sources.length > 4) {
  console.error('usage : node tools/mire.mts <theme.json | forge> [autre.json ...] (quatre au plus) [--recadrage full]')
  process.exit(2)
}

const LETTERS = 'ABCD'
const themes: Theme[] = []
for (const source of sources) {
  const loaded = loadTheme(source)
  if (Array.isArray(loaded)) {
    console.error(`${loaded.length} erreur(s) dans ${source} :`)
    for (const e of loaded) console.error(`  - ${e}`)
    process.exit(1)
  }
  themes.push(loaded)
}

const MODES = ['comme en vrai', "à l'arrêt", 'en marche vers la droite', 'en marche vers la gauche'] as const
const HOURS: ReadonlyArray<[string, Partial<World>]> = [
  ['de jour', { hour: 12 }],
  ['le soir', { hour: 19, lanternLit: true }],
  ['de nuit', { hour: 23, lanternLit: true }],
]
const state = { moment: 0, mode: 0, hour: 0, paused: false, tick: 0, clock: 1 }
// where each theme's character stands and heads, as the engine moves it
const walkers = themes.map(() => ({ x: 0, target: 0, restUntil: 0 }))

function restart() {
  state.tick = 0
  const m = MOMENTS[state.moment]!
  themes.forEach((t, i) => {
    const home = t.anchorOf(m.activity, { ...CALM, ...m.world }) ?? 0
    const start = MODES[state.mode] === 'en marche vers la droite' ? t.stage.roam.min : MODES[state.mode] === 'en marche vers la gauche' ? t.stage.roam.max : home
    walkers[i] = { x: start, target: start, restUntil: 0 }
  })
}

/** The world a theme draws this tick, its character moved a step the way the mode says. */
function worldFor(t: Theme, i: number): World {
  const m = MOMENTS[state.moment]!
  const w = walkers[i]!
  const world: World = { ...CALM, ...HOURS[state.hour]![1], ...m.world, clock: state.clock }
  const mode = MODES[state.mode]
  const { min, max } = t.stage.roam
  if (mode === 'comme en vrai') {
    const anchor = t.anchorOf(m.activity, world)
    if (anchor !== undefined) w.target = anchor
    else if (w.x === w.target && state.clock >= w.restUntil) Object.assign(w, wander(m.activity, w.x, state.clock, state.clock, t.stage.idleSpots))
    w.target = Math.max(min, Math.min(max, w.target))
    w.x = pace(m.activity, w.x, w.target, state.clock)
  } else if (mode !== "à l'arrêt") {
    // back and forth across the whole floor, a pixel a tick
    const dir = mode === 'en marche vers la droite' ? 1 : -1
    w.target = dir > 0 ? max : min
    if (w.x === w.target) w.x = dir > 0 ? min : max
    else w.x += dir
  }
  return { ...world, x: w.x, walking: w.x !== w.target, facing: w.target < w.x ? -1 : 1 }
}

const DEFAULT = 0x01000000
const sgr = (fg: number, bg: number) =>
  `\x1b[${fg === DEFAULT ? '39' : `38;2;${fg >> 16};${(fg >> 8) & 255};${fg & 255}`};${bg === DEFAULT ? '49' : `48;2;${bg >> 16};${(bg >> 8) & 255};${bg & 255}`}m`

function panel(t: Theme, i: number): string[] {
  const m = MOMENTS[state.moment]!
  const [x0, x1] = t.crops[crop] ?? t.crops.full
  const w = x1 - x0
  const tick = m.once ? state.tick % m.once : state.tick // a one-shot plays once in the band: here, over and over
  const cells = t.frame(crop, m.activity, tick, worldFor(t, i))
  const rows: string[] = []
  for (let r = 0; r < t.rows; r++) {
    let line = ''
    for (let x = 0; x < w; x++) {
      const k = (r * w + x) * 3
      line += sgr(cells[k + 1]!, cells[k + 2]!) + String.fromCodePoint(cells[k]!)
    }
    rows.push(line + '\x1b[0m')
  }
  const title = `${themes.length > 1 ? `${LETTERS[i]}  ` : ''}${t.title}`
  return [`\x1b[1m${title.slice(0, w)}\x1b[0m${' '.repeat(Math.max(0, w - title.length))}`, ...rows]
}

function screen(): string {
  const m = MOMENTS[state.moment]!
  const cols = process.stdout.columns || 120
  const widths = themes.map(t => { const [a, b] = t.crops[crop] ?? t.crops.full; return b - a })
  const side = widths.reduce((s, w) => s + w + 3, 0) <= cols
  const out: string[] = ['\x1b[H']
  const walk = roams(m) ? '  · le moteur fait marcher le personnage dans ce moment' : ''
  const once = m.once ? `  · joué une fois (${(m.once * 0.15).toFixed(1)} s), ici en boucle` : ''
  out.push(`\x1b[1m${state.moment + 1}/${MOMENTS.length}  ${m.label}\x1b[0m  · ${MODES[state.mode]}  · ${HOURS[state.hour]![0]}${state.paused ? '  · \x1b[7m en pause \x1b[0m' : ''}\x1b[K`)
  out.push(`\x1b[2m${walk}${once}\x1b[0m\x1b[K`)
  const panels = themes.map(panel)
  if (side) {
    const h = Math.max(...panels.map(p => p.length))
    for (let r = 0; r < h; r++) out.push(panels.map((p, i) => p[r] ?? ' '.repeat(widths[i]!)).join('   ') + '\x1b[K')
  } else {
    for (const p of panels) { out.push(...p.map(l => l + '\x1b[K')); out.push('\x1b[K') }
  }
  out.push('\x1b[K')
  // the moments, the one playing lit
  let line = ''
  for (const [i, mm] of MOMENTS.entries()) {
    const item = i === state.moment ? `\x1b[7m ${mm.label} \x1b[0m` : ` ${mm.label} `
    if (line.length > 0 && (line.replace(/\x1b\[[0-9;]*m/g, '') + mm.label).length + 3 > cols) { out.push(line + '\x1b[K'); line = '' }
    line += item
  }
  out.push(line + '\x1b[K')
  out.push('\x1b[2m← → le moment · m la marche · h l\'heure · espace pause · . un tic · q quitter\x1b[0m\x1b[K')
  return out.join('\n') + '\x1b[J'
}

function advance() {
  state.tick++
  state.clock++
}

if (testing) {
  // every moment in every mode, a few ticks each: nothing must throw
  for (state.moment = 0; state.moment < MOMENTS.length; state.moment++) {
    for (state.mode = 0; state.mode < MODES.length; state.mode++) {
      restart()
      for (let k = 0; k < 40; k++) { screen(); advance() }
    }
  }
  state.moment = MOMENTS.findIndex(m => m.activity === 'sweep'); state.mode = 0; restart()
  for (let k = 0; k < 6; k++) { screen(); advance() }
  console.log(screen().replace(/\x1b\[[0-9;?]*[A-Za-z]/g, '').split('\n').slice(1, 3).join('\n'))
  process.exit(0)
}

restart()
process.stdout.write('\x1b[2J\x1b[?25l')
const timer = setInterval(() => {
  process.stdout.write(screen())
  if (!state.paused) advance()
}, 150)
function quit() {
  clearInterval(timer)
  process.stdout.write('\x1b[0m\x1b[?25h\n')
  process.exit(0)
}
process.on('SIGINT', quit)
if (process.stdin.isTTY) process.stdin.setRawMode(true)
process.stdin.resume()
process.stdin.on('data', (data: Buffer) => {
  const key = data.toString()
  if (key === 'q' || key === '\u0003' || key === '\u001b') quit()
  else if (key === '\u001b[C' || key === 'l') { state.moment = (state.moment + 1) % MOMENTS.length; restart() }
  else if (key === '\u001b[D' || key === 'j') { state.moment = (state.moment + MOMENTS.length - 1) % MOMENTS.length; restart() }
  else if (key === 'm') { state.mode = (state.mode + 1) % MODES.length; restart() }
  else if (key === 'h') state.hour = (state.hour + 1) % HOURS.length
  else if (key === ' ') state.paused = !state.paused
  else if (key === '.' && state.paused) advance()
})
