// Drawing on a grid of letters, one per pixel ('.' transparent), and turning it
// into terminal cells: half blocks, two stacked pixels per cell.

export type Grid = string[][] & { over?: Map<number, Over> }

/**
 * A glyph finer than the half block, laid over one cell: braille dots, quarter
 * blocks, or a glyph of its own. It shows only while the cell keeps the two
 * pixels it was laid on (`under`): whatever is drawn over it later hides it.
 */
export type Over = { kind: 'dots' | 'quads' | 'glyph'; bits: number; c: string; bg: string; under: string }

export const blank = (width: number, height: number): Grid =>
  Array.from({ length: height }, () => Array<string>(width).fill('.'))

const cellOf = (cx: number, cy: number) => cy * 4096 + cx
const underOf = (g: Grid, cx: number, cy: number) => (g[2 * cy]?.[cx] ?? '.') + (g[2 * cy + 1]?.[cx] ?? '.')

function lay(g: Grid, cx: number, cy: number, kind: Over['kind'], bits: number, c: string, on?: string) {
  if (cx < 0 || cy < 0 || 2 * cy + 1 >= g.length || cx >= (g[0]?.length ?? 0)) return
  const under = underOf(g, cx, cy)
  const plain = under[0] === under[1]
  if (kind !== 'glyph' && (!plain || (on !== undefined && !on.includes(under[0] ?? '.')))) return // dots need one ground
  const over = (g.over ??= new Map())
  const was = over.get(cellOf(cx, cy))
  const merge = kind !== 'glyph' && was?.kind === kind && was.under === under
  over.set(cellOf(cx, cy), { kind, bits: merge ? (was.bits | bits) : bits, c, bg: plain ? under[0] ?? '.' : '.', under })
}

const BRAILLE = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]]
/** A braille dot at (fx, fy): two per pixel across, two per pixel down; only on a cell of one colour, in `on` if given. */
export function dot(g: Grid, fx: number, fy: number, c: string, on?: string) {
  const x = Math.round(fx), y = Math.round(fy)
  if (x < 0 || y < 0) return
  lay(g, x >> 1, y >> 2, 'dots', BRAILLE[x & 1]?.[y & 3] ?? 0, c, on)
}
/** A quarter block at (qx, qy): two per pixel across, one per pixel down; only on a cell of one colour, in `on` if given. */
export function quad(g: Grid, qx: number, qy: number, c: string, on?: string) {
  const x = Math.round(qx), y = Math.round(qy)
  if (x < 0 || y < 0) return
  lay(g, x >> 1, y >> 1, 'quads', 1 << ((y & 1) * 2 + (x & 1)), c, on)
}
/** A glyph over the cell (cx, cy), in place of the pixels it holds now, on the terminal's own background. */
export function glyph(g: Grid, cx: number, cy: number, ch: string, c: string) {
  lay(g, cx, cy, 'glyph', ch.codePointAt(0) ?? 0x20, c)
}

export const put = (g: Grid, x: number, y: number, ch: string) => {
  const row = g[y]
  if (row && x >= 0 && x < row.length && ch !== '.') row[x] = ch
}

export const erase = (g: Grid, x: number, y: number) => {
  const row = g[y]
  if (row && x >= 0 && x < row.length) row[x] = '.'
}

export function stamp(g: Grid, x: number, y: number, rows: readonly string[]) {
  rows.forEach((row, j) => {
    for (let i = 0; i < row.length; i++) put(g, x + i, y + j, row[i] ?? '.')
  })
}

export function line(g: Grid, x0: number, y0: number, x1: number, y1: number, ch: string | ((i: number) => string)) {
  const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0)
  const sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1
  let err = dx + dy, x = x0, y = y0
  for (let i = 0; ; i++) {
    put(g, x, y, typeof ch === 'string' ? ch : ch(i))
    if (x === x1 && y === y1) return
    const e2 = 2 * err
    if (e2 >= dy) { err += dy; x += sx }
    if (e2 <= dx) { err += dx; y += sy }
  }
}

/** A row read right to left: a sprite turned the other way. */
export const mirror = (rows: readonly string[]) => rows.map(r => [...r].reverse().join(''))

/** A fixed pseudo-random in [0, 1) for an integer seed. */
export const rand = (n: number) => ((Math.imul(n ^ 0x5bd1e995, 0x27d4eb2d) >>> 0) % 10007) / 10007

const DEFAULT = 0x01000000
const UPPER = 0x2580 // ▀
const LOWER = 0x2584 // ▄
const QUADS = [...' ▘▝▀▖▌▞▛▗▚▐▜▄▙▟█'].map(ch => ch.codePointAt(0) ?? 0x20)

/**
 * A light on the scene: the colour a pixel of `letter` at (x, y) takes, from its
 * palette colour; for a clear pixel (no colour), what the light shows behind the
 * scene, or nothing.
 */
export type Light = (x: number, y: number, letter: string, colour: number | undefined) => number | undefined

/** Raster cells (RasterProps.cells) for columns x0..x1 of the grid: two pixels per cell, then what lies over them, under `light` if given. */
export function toCells(g: Grid, palette: Readonly<Record<string, number>>, x0: number, x1: number, light?: Light): Uint32Array {
  const w = x1 - x0
  const rows = Math.floor(g.length / 2)
  const words = new Uint32Array(w * rows * 3)
  const colour = (x: number, y: number, letter: string) => {
    const c = palette[letter]
    return light ? light(x, y, letter, c) : c
  }
  for (let r = 0; r < rows; r++) {
    for (let x = 0; x < w; x++) {
      const top = colour(x0 + x, 2 * r, g[2 * r]?.[x0 + x] ?? '.')
      const bot = colour(x0 + x, 2 * r + 1, g[2 * r + 1]?.[x0 + x] ?? '.')
      const i = (r * w + x) * 3
      if (top === undefined && bot === undefined) words.set([0x20, DEFAULT, DEFAULT], i)
      else if (top === undefined) words.set([LOWER, bot ?? DEFAULT, DEFAULT], i)
      else words.set([UPPER, top, bot ?? DEFAULT], i)
      const o = g.over?.get(cellOf(x0 + x, r))
      if (o && o.under === underOf(g, x0 + x, r)) {
        const ch = o.kind === 'dots' ? 0x2800 + o.bits : o.kind === 'quads' ? QUADS[o.bits] ?? 0x20 : o.bits
        words.set([ch, colour(x0 + x, 2 * r, o.c) ?? DEFAULT, colour(x0 + x, 2 * r, o.bg) ?? DEFAULT], i)
      }
    }
  }
  return words
}
