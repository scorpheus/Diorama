// The character as the passeur (passeur/passeur.ps1) flies him from one window
// to another on a visit: the theme's own walk, cut out of the scene.
//
// No theme says what its character looks like on its own, so he is found the way
// one would by eye: the scene drawn with him, then the same instant without him;
// the pixels that differ are his.

import type { Activity } from '../types'
import type { Theme, World } from './world.ts'

/**
 * The character walking right, a frame a tick: `w` x `h` pixels each, read row
 * by row, one letter per pixel (`.` clear) whose colour is `colours[letter]`
 * (0xRRGGBB); `top` is the row of the scene his head is at, `x` the column of the
 * scene his left edge is at in the first frame.
 */
export type Sprite = { w: number; h: number; top: number; x: number; colours: Record<string, number>; frames: string[] }

const DEFAULT = 0x01000000
const UPPER = 0x2580 // ▀
const LOWER = 0x2584 // ▄
const LETTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
const FRAMES = 8 // a walk's cycle and some
const MAX_WIDTH = 48 // pixels: a cut-out wider than this caught something else than him

/** Raster cells back into pixels, two to a cell: a colour, or undefined where clear. */
function pixelsOf(cells: Uint32Array, columns: number): (number | undefined)[][] {
  const rows = cells.length / 3 / columns
  const px: (number | undefined)[][] = Array.from({ length: rows * 2 }, () => Array<number | undefined>(columns))
  const colour = (c: number | undefined) => (c === undefined || c === DEFAULT ? undefined : c)
  for (let r = 0; r < rows; r++) {
    for (let x = 0; x < columns; x++) {
      const i = (r * columns + x) * 3
      const ch = cells[i], fg = colour(cells[i + 1]), bg = colour(cells[i + 2])
      const top = px[2 * r], bot = px[2 * r + 1]
      if (!top || !bot) continue
      if (ch === UPPER) [top[x], bot[x]] = [fg, bg]
      else if (ch === LOWER) [top[x], bot[x]] = [undefined, fg]
      else if (ch !== 0x20) [top[x], bot[x]] = [bg ?? fg, bg ?? fg] // a finer glyph: its ground stands for it
    }
  }
  return px
}

/**
 * The character of `world` walking right, cut out of the scene, in the palette's
 * own colours; undefined when the cut-out caught more than him (a theme where his
 * absence changes something else in the scene).
 */
export function spriteOf(theme: Theme, world: World, activity: Activity = 'idle'): Sprite | undefined {
  const [x0, x1] = theme.crops.full
  const columns = x1 - x0
  const alone: World = {
    ...world, x: 0, walking: true, facing: 1, absent: false, guests: [], apprentices: [], cat: undefined, messenger: undefined,
    speaking: false, banner: undefined, gift: undefined, rare: undefined, routine: undefined, dream: undefined,
  }
  const cut: { x: number; y: number; c: number }[][] = []
  for (let t = 0; t < FRAMES; t++) {
    const seen = pixelsOf(theme.frame('full', activity, t, { ...alone, x: t }, false), columns)
    const empty = pixelsOf(theme.frame('full', activity, t, { ...alone, x: t, absent: true }, false), columns)
    const mine: { x: number; y: number; c: number }[] = []
    seen.forEach((row, y) => row.forEach((c, x) => {
      if (c !== undefined && c !== empty[y]?.[x]) mine.push({ x: x - t, y, c }) // his own frame of reference: the steps cancel out
    }))
    if (mine.length === 0) return undefined
    cut.push(mine)
  }
  const all = cut.flat()
  const left = Math.min(...all.map(p => p.x)), right = Math.max(...all.map(p => p.x))
  const top = Math.min(...all.map(p => p.y)), bottom = Math.max(...all.map(p => p.y))
  const w = right - left + 1, h = bottom - top + 1
  if (w > MAX_WIDTH || h > theme.rows * 2) return undefined
  const letters = new Map<number, string>()
  const colours: Record<string, number> = {}
  const frames = cut.map(mine => {
    const grid = Array.from({ length: h }, () => Array<string>(w).fill('.'))
    for (const p of mine) {
      let l = letters.get(p.c)
      if (l === undefined) {
        l = LETTERS[letters.size] ?? '.'
        letters.set(p.c, l)
        colours[l] = p.c
      }
      const row = grid[p.y - top]
      if (row) row[p.x - left] = l
    }
    return grid.map(r => r.join('')).join('')
  })
  return { w, h, top, x: left + x0, colours, frames }
}
