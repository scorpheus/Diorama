// Drawing on a grid of letters, one per pixel ('.' transparent), and turning it
// into terminal cells: half blocks, two stacked pixels per cell.

export type Grid = string[][]

export const blank = (width: number, height: number): Grid =>
  Array.from({ length: height }, () => Array<string>(width).fill('.'))

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

/** Raster cells (RasterProps.cells) for columns x0..x1 of the grid: two pixels per cell. */
export function toCells(g: Grid, palette: Readonly<Record<string, number>>, x0: number, x1: number): Uint32Array {
  const w = x1 - x0
  const rows = Math.floor(g.length / 2)
  const words = new Uint32Array(w * rows * 3)
  for (let r = 0; r < rows; r++) {
    for (let x = 0; x < w; x++) {
      const top = palette[g[2 * r]?.[x0 + x] ?? '.']
      const bot = palette[g[2 * r + 1]?.[x0 + x] ?? '.']
      const i = (r * w + x) * 3
      if (top === undefined && bot === undefined) words.set([0x20, DEFAULT, DEFAULT], i)
      else if (top === undefined) words.set([LOWER, bot ?? DEFAULT, DEFAULT], i)
      else words.set([UPPER, top, bot ?? DEFAULT], i)
    }
  }
  return words
}
