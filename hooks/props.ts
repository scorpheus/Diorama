// What every theme may use: the default palette, the dreams, the bubbles, the
// sparks, the messenger, the sky through a window. Drawn first for the forge;
// a theme of its own takes them as they are, or redraws them.

import type { Flavor } from '../types'
import type { Dream, World } from './world.ts'
import { line, put, rand, stamp } from './pixels.ts'
import type { Grid } from './pixels.ts'

/** The default palette: every theme has these letters, and may change or add to them. */
export const BASE_PALETTE: Record<string, number> = {
  k: 0x1b1a24, // outline
  d: 0x7c808c, // fur shadow
  g: 0xb4b7bf, // fur grey
  l: 0xdedbd3, // fur light
  w: 0xfaf9f4, // fur white
  n: 0x13294f, // crest navy
  B: 0x1f4f9e, // crest blue
  b: 0x3f80dc, // crest light
  e: 0x2b8ff2, // iris
  p: 0x060b18, // pupil
  h: 0xffffff, // eye glint
  y: 0xe3a63c, // gold
  t: 0x34b5ae, // teal scales
  o: 0x3f5a24, // cloak dark
  O: 0x62802f, // cloak
  r: 0x7a4a24, // leather, wood
  R: 0x4a2c16, // dark wood, logs
  m: 0x141018, // nose, mouth
  a: 0x4a4e5c, // steel
  A: 0x8a90a0, // steel light
  q: 0x4f4a55, // stone
  Q: 0x6e6876, // stone light
  z: 0x1f1a20, // mortar, furnace mouth
  E: 0xa3301b, // ember
  F: 0xff5a1f, // flame
  f: 0xffb02e, // flame bright
  Y: 0xfff3b0, // white heat
  H: 0xff8c2a, // hot metal
  s: 0xffd75e, // spark
  S: 0xff8a2a, // spark hot
  x: 0x5e5966, // smoke
  X: 0x8d8896, // smoke light
  c: 0x7fd4ff, // water, sweat, tears
  W: 0xefe2c0, // parchment
  V: 0xb8a57c, // parchment ink, log ends
  G: 0x74e07a, // test spark
  j: 0xc4f5a0, // test spark light
  P: 0xb07cff, // apprentice
  i: 0xe2c8ff, // apprentice light
  M: 0xff7ad9, // rune spark
  u: 0xbff3ff, // shell spark white, sleep z
  C: 0x6e2618, // cape: embers, dark
  D: 0xb5452a, // cape: embers
  N: 0x1e2a55, // cape: night, dark
  L: 0x3a5a9a, // cape: night
  U: 0x45285a, // cape: plum, dark
  I: 0x7d4f9e, // cape: plum
  T: 0x5e4212, // cape: ochre, dark
  J: 0xa8792a, // cape: ochre
  Z: 0x2e343a, // cape: slate, dark
  v: 0x5d6873, // cape: slate
  1: 0xc99a3c, // the panther's gold
  2: 0x2e2010, // its rosettes
  3: 0xead29a, // its muzzle
  4: 0xd6e64a, // its green eye
  5: 0x8a6424, // its far legs, in shadow
}

/** The default looks: [dark, light] letters, and what each means for Haiku. */
export const BASE_VARIANTS = {
  foret: { colors: ['o', 'O'], meaning: 'général, tranquille' },
  braise: { colors: ['C', 'D'], meaning: 'urgence, feu, correction pressée' },
  nuit: { colors: ['N', 'L'], meaning: 'réflexion, analyse, conception' },
  prune: { colors: ['U', 'I'], meaning: 'créatif, design, idées' },
  ocre: { colors: ['T', 'J'], meaning: 'données, recherche, mesures' },
  ardoise: { colors: ['Z', 'v'], meaning: 'technique, infrastructure, outillage' },
} as const

/** Spark colours by flavor of work: [flash, trail, spark, spark]. */
export const SPARKS: Record<Flavor, readonly [string, string, string, string]> = {
  forge: ['Y', 'S', 's', 'S'], // edits: the smith's gold
  build: ['Y', 'F', 'f', 'S'], // compiling: the furnace roars
  test: ['j', 'G', 'j', 'G'], // tests: the blade proved green
  shell: ['u', 'b', 'c', 'e'], // other commands: cold blue steel
  git: ['w', 'A', 'w', 'g'], // git: silver seal
  web: ['u', 't', 'c', 't'], // the web: teal far-sight
  agent: ['i', 'P', 'i', 'P'], // subagents: violet apprentices
  mcp: ['i', 'M', 'M', 'P'], // MCP tools: pink runes
}

// What it dreams of, 9 x 5 each, on the bubble's night: a 5 x 3 one read as a few dashes
export const DREAMS: Record<Dream, readonly string[]> = {
  voiture: ['..FFFF...', '.FwbbwF..', 'FFFFFFFFY', 'FkakFFkak', '.kak..kak'],
  fusee: ['..F......', '..wwwwF..', 'fFwwbwwwF', '..wwwwF..', '..F......'],
  bug: ['k..k.k..k', '.k.GGG.k.', '..GjGjG..', '.kGGGGGk.', 'k..G.G..k'],
  circuit: ['.A.A.A.A.', 'AkkkkkkkA', '.kkGjGkk.', 'AkkkkkkkA', '.A.A.A.A.'],
  livre: ['.WWWkWWW.', 'WVVWkWVVW', 'WWWWkWWWW', 'WVVWkWVVW', '.rrrkrrr.'],
  montagne: ['....w....', '...www...', '..gwgdg..', '.gggdddd.', 'ggggddddd'],
  poisson: ['...bbb..b', '.bbbbbbbb', 'bwbbbbbb.', '.bbbbbbbb', '...bbb..b'],
  etoile: ['....Y....', '...YYY...', 'YYYYYYYYY', '..YYYYY..', '.YY...YY.'],
  gateau: ['..F.F.F..', '..y.y.y..', '.MMMMMMM.', '.WWWWWWW.', '.MMMMMMM.'],
  dragon: ['......GG.', 'G....GjGG', '.GGGGGG..', '..GG.GG..', '..G...G..'],
  drapeau: ['kwkwkwk..', 'wkwkwkw..', 'kwkwkwk..', 'A........', 'A........'],
  carburant: ['..AA.....', '.FFFFF...', '.FyFyF...', '.FFyFF...', '.FyFyF...'],
  cle: ['.........', 'A.A...AAA', 'AAAAAAA.A', '.A....AAA', '.........'],
  ecran: ['AAAAAAAAA', 'AGG.yy..A', 'A.bbb.G.A', 'AAAAAAAAA', '...AAA...'],
  musique: ['...YYYYY.', '...Y...Y.', '...Y...Y.', '.YYY.YYY.', '.YYY.YYY.'],
  base: ['.cwwwwwc.', '.bcccccb.', '.bbbbbbb.', '.bcccccb.', '.bbbbbbb.'],
  reseau: ['cc.....cc', 'cc.w.w.cc', '...ccc...', 'cc.w.w.cc', 'cc.....cc'],
  manette: ['..aaaaa..', '.aAAAAAa.', 'aAwAAAFAa', 'aAAAAAGAa', '.aa...aa.'],
  graphique: ['.......G.', '....y..G.', '.F..y..G.', '.F..y..G.', 'AAAAAAAAA'],
  photo: ['..aaa....', 'AAAAAAAAA', 'AAAkkkAyA', 'AAAkckAAA', 'AAAAAAAAA'],
  carte: ['...FFF...', '..FFwFF..', '..FFFFF..', '...FFF...', '....F....'],
  maison: ['...FFF...', '..FFFFF..', '.FFFFFFF.', '..WbWrW..', '..WWWrW..'],
  horloge: ['..AAAAA..', '.AWWkWWA.', '.AWWkkkA.', '.AWWWWWA.', '..AAAAA..'],
  cadenas: ['...aaa...', '..a...a..', '.yyyyyyy.', '.yyykyyy.', '.yyyyyyy.'],
  nuage: ['...www...', '.wwwwww..', 'wwwwwwwww', '.wwwwwww.', '.c.c.c.c.'],
  ampoule: ['...YYY...', '..YYYYY..', '..YYwYY..', '...YYY...', '...aaa...'],
  engrenage: ['...A.A...', '..AAAAA..', '.AAA.AAA.', '..AAAAA..', '...A.A...'],
  robot: ['....F....', '.AAAAAAA.', '.AcAAAcA.', '.AAkkkAA.', '.AAAAAAA.'],
  plante: ['..G...G..', '.GGG.GGG.', '...GjG...', '..rrrrr..', '...rrr...'],
  cafe: ['...w.w...', '..w.w....', '.WWWWW.W.', '.WrrrWW..', '..WWW....'],
}

/** What each dream stands for, as Haiku is told when it picks one: every motif needs its line. */
export const DREAM_HINTS: Record<Dream, string> = {
  voiture: 'véhicule, conduite, automobile',
  drapeau: 'course, compétition, piste, classement',
  carburant: 'carburant, énergie, ravitaillement, consommation',
  fusee: 'vitesse, performance, lancement, déploiement',
  bug: 'débogage, correction de bugs',
  circuit: 'électronique, matériel, embarqué',
  ecran: 'interface, affichage, front-end, code',
  cle: 'accès, identifiants, clés d\'API, authentification',
  cadenas: 'sécurité, chiffrement, permissions',
  livre: 'documentation, écriture, lecture',
  montagne: 'gros chantier, refonte, défi',
  etoile: 'réussite, mise en valeur, favoris',
  gateau: 'fête, anniversaire, récompense',
  poisson: 'flux, streaming, pêche',
  musique: 'son, audio, musique, rythme',
  base: 'base de données, SQL, stockage',
  reseau: 'réseau, API, serveurs, communication',
  manette: 'jeu vidéo, gameplay, divertissement',
  graphique: 'statistiques, analyse de données, mesures, tableaux de bord',
  photo: 'image, photo, vidéo, rendu',
  carte: 'géographie, cartes, localisation, voyage',
  maison: 'domotique, maison, immobilier, famille',
  horloge: 'temps, planification, performances, chronométrage',
  nuage: 'cloud, météo, hébergement',
  ampoule: 'idée, conception, réflexion, brainstorming',
  engrenage: 'configuration, outillage, automatisation, mécanique',
  robot: 'IA, agents, modèles, automates',
  plante: 'croissance, nature, jardinage, écologie',
  cafe: 'pause, routine, petites tâches',
  dragon: 'tout le reste',
}

const DREAM_BUBBLE = [
  '.wwwwwwwwwww.',
  'ww.........ww',
  'w...........w',
  'w...........w',
  'w...........w',
  'w...........w',
  'ww.........ww',
  '.wwwwwwwwwww.',
]

/** The dream's bubble, 13 x 8, at (x, y). */
export function drawDream(g: Grid, x: number, y: number, dream: Dream) {
  stamp(g, x, y, DREAM_BUBBLE)
  // the dream's night inside the rim, then the motif in its middle
  DREAM_BUBBLE.forEach((row, j) => {
    const first = row.indexOf('w'), last = row.lastIndexOf('w')
    for (let i = first + 1; i < last; i++) if (row[i] === '.' && j > 0 && j < DREAM_BUBBLE.length - 1) put(g, x + i, y + j, 'n')
  })
  stamp(g, x + 2, y + 2, DREAMS[dream])
}

/** The '!' bubble: the person is needed. */
export const BUBBLE = ['.kkkkk.', 'kwwFwwk', 'kwwFwwk', 'kwwFwwk', 'kwwwwwk', 'kwwFwwk', '.kkkkk.', 'kk.....']
/** A line said aloud. */
export const SAYING = ['.kkkkk.', 'kwwwwwk', 'kwkwkwk', 'kwwwwwk', '.kkkkk.', 'kk.....']
/** A gear turning in a cloud: thinking. */
export const THOUGHT = [['.wwwww.', 'wwwwwww', 'ww.a.ww', 'wwaAaww', 'ww.a.ww', 'wwwwwww', '.wwwww.'],
  ['.wwwww.', 'wwwwwww', 'wwa.aww', 'ww.A.ww', 'wwa.aww', 'wwwwwww', '.wwwww.']]
export const Z_BIG = ['uuuu', '..u.', '.u..', 'uuuu']
export const Z_SMALL = ['uuu', '.u.', 'uuu']
/** A cloud of dust, in three ages: a failure, an arrival. */
export const PUFF = [
  ['..xx..', '.xXXx.', '..xx..'],
  ['.xXXx.', 'xXXXXx', 'xXxXXx', '.xXXx.'],
  ['x.XX.x', '.X..X.', 'X....X', '.x..x.'],
]
// Grey plumage: a black bird vanishes against a dark terminal. It faces left,
// the way it flies; four wingbeats, the body on rows 2..4, its claws under row 4.
export const RAVEN = [
  ['.....XX..', '......Xx.', '.xw....x.', 'yxxxxxxx.', '..xxxxxXX'], // wings up
  ['.........', '.........', '.xw.XXXX.', 'yxxxxxxxX', '..xxxxxX.'], // level
  ['.........', '.........', '.xw......', 'yxxxxxxx.', '..xxxXxXX', '....XX...', '...X.....'], // down
  ['.........', '.........', '.xw......', 'yxxxXXXxX', '..xxxxxX.'], // coming back up
]
/** Where a raven's load hangs from its claws, from its top-left. */
export const CLAWS = { x: 3, y: 5 }
export const GIFTS: Record<NonNullable<World['gift']>['kind'], readonly string[]> = {
  coin: ['.y.', 'yYy', '.y.'],
  cookie: ['rVr', 'VrV', 'rVr'],
  ore: ['.A.', 'AbA', 'aAa'],
}

/** Sparks flying from (ox, oy), `age` ticks after the blow. */
export function drawSparks(g: Grid, age: number, seed: number, count: number, ox: number, oy: number, pal: readonly [string, string, string, string]) {
  const at = (vx: number, vy: number, a: number) =>
    [Math.round(ox + vx * a), Math.round(oy - vy * a + 0.4 * a * a)] as const
  if (age === 0) {
    // the flash of the blow
    for (const [dx, dy] of [[0, 0], [-1, 0], [1, 0], [0, -1], [-2, -1], [2, -1]] as const) put(g, ox + dx, oy + dy, dx || dy ? pal[2] : pal[0])
  }
  for (let i = 0; i < count; i++) {
    const vx = (rand(seed * 17 + i) * 2 - 1) * 2.4
    const vy = 1 + rand(seed * 29 + i) * 1.8
    if (rand(seed * 41 + i) * 6 < age - 1) continue // the spark dies out
    const [x, y] = at(vx, vy, age)
    const [tx, ty] = at(vx, vy, Math.max(0, age - 0.5))
    put(g, tx, ty, pal[1])
    put(g, x, y, age < 2 ? pal[0] : i % 3 === 0 ? pal[3] : pal[2])
  }
}

export const season = (w: World) =>
  w.month === 12 || w.month <= 2 ? 'winter' : w.month <= 5 ? 'spring' : w.month <= 8 ? 'summer' : 'autumn'
export const feast = (w: World) =>
  w.month === 10 && w.day === 2 ? 'birthday' // the day the forge was lit, 2026
  : w.month === 10 && w.day >= 25 ? 'halloween'
  : w.month === 12 && w.day >= 15 && w.day <= 26 ? 'christmas'
  : (w.month === 12 && w.day === 31) || (w.month === 1 && w.day === 1) ? 'newyear'
  : undefined

// A dragon far off, flying right: tail left, head raised right, a wingbeat in four.
const DRAGON = [
  ['...B......', '..BBn...n.', '..BBnn..nn', 'n..nnnnnnB', '.nnn.n.n..'],
  ['..........', '........n.', '.BBBBn..nn', 'n..nnnnnnB', '.nnn.n.n..'],
  ['..........', '........n.', '........nn', 'n..nnnnnnB', '.nnnBBnn..', '...BBB....', '...B......'],
  ['..........', '........n.', '........nn', 'n..nnnnnnB', '.nnBBBBn..'],
]
const DRAGON_W = 10
/** Ticks a rare sight lasts in a sky `width` pixels wide (frame included): the dragon crosses a pixel every three ticks. */
export const rareFor = (width: number) => (DRAGON_W + width) * 3

/** Panes: the inside of a window, where the sky shows. */
export type Panes = { x: number; y: number; w: number; h: number }

/**
 * The sky through panes: by the hour, the season's weather, the run's weather,
 * the rare sights; with `shutters`, closed for the night from 23h to 6h.
 */
export function drawSky(g: Grid, w: World, tick: number, panes: Panes, shutters: boolean) {
  const { x: ix, y: iy, w: iw, h: ih } = panes
  const night = w.hour < 6 || w.hour >= 21
  const twilight = !night && (w.hour < 8 || w.hour >= 18)
  for (let j = 0; j < ih; j++) {
    for (let i = 0; i < iw; i++) {
      const sky = w.weather === 'storm' ? (j < 3 ? 'x' : 'X')
        : night ? 'n'
        : twilight ? (j < 2 ? 'B' : j < 4 ? 'b' : j < 6 ? 'f' : 'F')
        : 'c'
      put(g, ix + i, iy + j, sky)
    }
  }
  // the sun or the moon, along an arc through the day or the night
  const along = night ? ((w.hour + 3) % 24) / 9 : (w.hour - 6) / 15
  const bx = ix + Math.round(Math.max(0, Math.min(1, along)) * (iw - 2))
  const by = iy + Math.round(Math.abs(along - 0.5) * 2 * (ih - 3))
  if (w.weather !== 'storm') {
    if (night) {
      stamp(g, bx, by, ['WW', 'Wn'])
      for (let i = 0; i < 4; i++) if (rand(i * 31 + Math.floor(tick / 6)) > 0.35) put(g, ix + ((i * 5 + 2) % iw), iy + ((i * 3 + 1) % (ih - 2)), 'w')
    } else {
      stamp(g, bx, by, ['YY', 'Yy'])
    }
  }
  // each flake keeps its colour (`colours[i]`, cycling): a colour that changed
  // every frame read as a blinking warning light, not a leaf
  const fall = (colours: string, n: number, speed: number) => {
    for (let i = 0; i < n; i++) {
      const fx = ix + Math.floor(rand(i * 13) * iw) + Math.round(Math.sin((tick + i * 7) / 3))
      const fy = iy + Math.floor((tick * speed + i * 5) % ih)
      if (fx >= ix && fx < ix + iw) put(g, fx, fy, colours[i % colours.length] ?? 'w')
    }
  }
  if (w.weather === 'storm') {
    fall('c', 6, 1.5)
    if (tick % 23 < 2) {
      for (let j = 0; j < ih; j++) for (let i = 0; i < iw; i++) put(g, ix + i, iy + j, 'X')
      line(g, ix + 6, iy, ix + 4, iy + 3, 'Y')
      line(g, ix + 4, iy + 3, ix + 6, iy + 6, 'Y')
    }
  } else if (w.weather === 'rainbow') {
    const bands = ['F', 'f', 'y', 'G', 'b', 'P']
    bands.forEach((ch, k) => {
      for (let i = 0; i < iw; i++) {
        const t = (i - (iw - 1) / 2) / ((iw - 1) / 2)
        const ry = iy + 1 + k + Math.round(t * t * 3)
        if (ry < iy + ih) put(g, ix + i, ry, ch)
      }
    })
  }
  const sea = season(w)
  if (sea === 'winter') fall('w', 5, 0.5)
  if (sea === 'autumn') fall('Fy', 2, 0.35) // one russet leaf, one gold
  if (sea === 'spring') fall('M', 2, 0.3)
  if (feast(w) === 'newyear' && night) drawSparks(g, tick % 6, Math.floor(tick / 6), 6, ix + 3 + (Math.floor(tick / 6) % 5), iy + 3, ['Y', 'M', 's', 'b'])
  if (w.rare?.kind === 'star') {
    // a shooting star crosses the panes, its trail behind it
    const a = w.rare.age
    for (let k = 0; k < 4; k++) put(g, ix + iw - 1 - a + k, iy + Math.floor((a - k) / 2), k === 0 ? 'Y' : 'w')
  }
  if (w.rare?.kind === 'dragon') {
    // far off, a dragon beats its wings across the sky, seen through the panes only
    const dx = ix - DRAGON_W + Math.floor(w.rare.age / 3)
    const rows = DRAGON[Math.floor(w.rare.age / 2) % DRAGON.length] ?? []
    rows.forEach((row, j) => {
      for (let i = 0; i < row.length; i++) {
        const px = dx + i, py = iy - 1 + j
        if (px >= ix && px < ix + iw && py >= iy && py < iy + ih) put(g, px, py, row[i] ?? '.')
      }
    })
  }
  if (shutters && (w.hour >= 23 || w.hour < 6)) {
    // the shutters closed for the night, a thread of moonlight between them
    for (let j = 0; j < ih; j++) for (let i = 0; i < iw; i++) put(g, ix + i, iy + j, i === Math.floor(iw / 2) - 1 ? 'n' : i % 3 === 0 ? 'R' : 'r')
  }
}
