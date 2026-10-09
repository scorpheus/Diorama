// Aperçu d'un thème Diorama : vérifie le fichier, puis dessine une planche PNG
// des moments principaux, exactement comme le bandeau les dessinerait.
//
//   node tools/apercu.mts <theme.json | forge> <sortie.png> [--echelle 4] [--recadrage full]
//
// Node 22.6 ou plus (il lit le TypeScript du mod tel quel). La planche se lit
// de gauche à droite, ligne par ligne, dans l'ordre que la commande affiche.

import { readFileSync, writeFileSync } from 'node:fs'
import { crc32, deflateSync } from 'node:zlib'

import { forge } from '../hooks/themes/forge.ts'
import { checkTheme, dataTheme } from '../hooks/themes/data.ts'
import type { ThemeSpec } from '../hooks/themes/data.ts'
import { CALM } from '../hooks/world.ts'
import type { Crop, Theme, World } from '../hooks/world.ts'

const args = process.argv.slice(2)
const flag = (name: string, fallback: string) => {
  const i = args.indexOf(name)
  return i >= 0 ? args.splice(i, 2)[1] ?? fallback : fallback
}
const scale = Number(flag('--echelle', '4'))
const crop = flag('--recadrage', 'full') as Crop
const [source, out] = args
if (!source || !out) {
  console.error('usage : node tools/apercu.mts <theme.json | forge> <sortie.png> [--echelle 4] [--recadrage full]')
  process.exit(2)
}

let theme: Theme
if (source === 'forge') {
  theme = forge
} else {
  const raw = JSON.parse(readFileSync(source, 'utf8')) as unknown
  const errors = checkTheme(raw)
  if (errors.length > 0) {
    console.error(`${errors.length} erreur(s) dans ${source} :`)
    for (const e of errors) console.error(`  - ${e}`)
    process.exit(1)
  }
  theme = dataTheme(raw as ThemeSpec, source)
}

const cub = (seed: number, age: number, colour: string) => ({ seed, colour, age })
const guest = { x: theme.stage.guestSpots[0] ?? 20, walking: false, facing: -1, cape: ['N', 'L'] as const, accessory: theme.looks.accessories[1] ?? 'aucun', tick: 3, waving: true }
// The moments shown, in order: [label, activity, tick, world]
const SCENES: Array<[string, Parameters<Theme['frame']>[1], number, Partial<World>]> = [
  ['repos', 'idle', 0, {}],
  ['repos (plus tard)', 'idle', 13, { hour: 9 }],
  ['marche', 'idle', 1, { x: -5, walking: true, facing: -1 }],
  ['marche (autre pas)', 'idle', 2, { x: 3, walking: true, facing: 1 }],
  ['réfléchit', 'thinking', 4, {}],
  ['travaille', 'running', 3, { x: theme.anchorOf('running', CALM) ?? 0 }],
  ['travaille (compile)', 'running', 5, { flavor: 'build', effort: 2, x: theme.anchorOf('running', { ...CALM, flavor: 'build' }) ?? 0 }],
  ['examine', 'review', 2, {}],
  ['attend la personne', 'waiting', 1, {}],
  ['échec', 'failed', 2, {}],
  ['réussite', 'jumping', 3, {}],
  ['salue', 'waving', 2, {}],
  ['dort (rêve)', 'sleep', 4, { dream: 'fusee', hour: 23 }],
  ['parle', 'idle', 5, { speaking: true }],
  ['aides au travail', 'idle', 0, { apprentices: [cub(3, 20, 'P'), cub(7, 33, 'b'), cub(11, 90, 'G')] }],
  ['visite + animal', 'idle', 0, { guests: [guest], cat: { mode: 'roof', x: 0, age: 3 } }],
  ['animal en marche', 'idle', 0, { cat: { mode: 'walk', x: 30, age: 3, dir: -1, step: 1 } }],
  ['soir, trophées, jauges', 'idle', 0, { hour: 20, lanternLit: true, rack: ['sword', 'axe', 'dagger', 'mace'], masterpiece: true, ctx: 80, fuel5h: 40, fuel7d: 70, usd: 6 }],
  ['orage, sous pression', 'idle', 0, { weather: 'storm', pressure: 92 }],
  ['fête, messager', 'idle', 0, { banquet: true, messenger: { kind: 'waiting', age: 12 }, gift: { kind: 'coin', age: 1 } }],
]

const [x0, x1] = theme.crops[crop] ?? theme.crops.full
const fw = x1 - x0, fh = theme.rows * 2
const cols = 2
const gap = 2
const W = (fw + gap) * cols + gap, H = (fh + gap) * Math.ceil(SCENES.length / cols) + gap
const pixels = new Uint8Array(W * H * 3).fill(0x16) // the terminal's dark background
SCENES.forEach(([, activity, tick, world], n) => {
  const ox = gap + (n % cols) * (fw + gap), oy = gap + Math.floor(n / cols) * (fh + gap)
  const words = theme.frame(crop, activity, tick, { ...CALM, clock: tick * 7 + 1, ...world })
  for (let r = 0; r < theme.rows; r++) {
    for (let x = 0; x < fw; x++) {
      const i = (r * fw + x) * 3
      const chr = words[i], fg = words[i + 1] ?? 0, bg = words[i + 2] ?? 0
      for (let half = 0; half < 2; half++) {
        // finer glyphs, at a pixel's scale: a half shows the ink when it holds any of it
        const braille = chr >= 0x2800 && chr <= 0x28ff ? chr - 0x2800 : -1
        const quads = ' ▘▝▀▖▌▞▛▗▚▐▜▄▙▟█'.indexOf(String.fromCodePoint(chr ?? 0x20))
        const inked = braille >= 0 ? (braille & (half === 0 ? 0x1b : 0xe4)) !== 0
          : quads > 0 ? (quads & (half === 0 ? 3 : 12)) !== 0
          : chr !== 0x20 && chr !== 0x2580 && chr !== 0x2584
        const c = chr === 0x2580 ? (half === 0 ? fg : bg) : chr === 0x2584 ? (half === 1 ? fg : bg) : inked ? fg : bg
        if (c === 0x01000000) continue
        const p = ((oy + 2 * r + half) * W + ox + x) * 3
        pixels[p] = (c >> 16) & 255; pixels[p + 1] = (c >> 8) & 255; pixels[p + 2] = c & 255
      }
    }
  }
})

// PNG, scaled up: one square per pixel
const SW = W * scale, SH = H * scale
const raw = Buffer.alloc((SW * 3 + 1) * SH)
for (let y = 0; y < SH; y++) {
  raw[y * (SW * 3 + 1)] = 0
  for (let x = 0; x < SW; x++) {
    const p = ((Math.floor(y / scale) * W) + Math.floor(x / scale)) * 3
    raw.set(pixels.subarray(p, p + 3), y * (SW * 3 + 1) + 1 + x * 3)
  }
}
const chunk = (type: string, data: Buffer) => {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length)
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data])
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(body) >>> 0)
  return Buffer.concat([len, body, crc])
}
const ihdr = Buffer.alloc(13)
ihdr.writeUInt32BE(SW, 0); ihdr.writeUInt32BE(SH, 4); ihdr[8] = 8; ihdr[9] = 2
writeFileSync(out, Buffer.concat([Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]))
console.log(`${theme.title} : ${SCENES.length} moments, ${cols} par ligne, dans cet ordre :`)
SCENES.forEach(([label], n) => console.log(`  ${n + 1}. ${label}`))
console.log(`→ ${out}`)
