// Which theme the `theme` option names: the forge (the default), a theme shipped
// in the plugin's themes/ folder by its name, or a JSON file by its absolute path.
// The engine reads the file; what is here only names it and makes a theme of it.

import type { Theme } from '../world.ts'
import { checkTheme, dataTheme } from './data.ts'
import type { ThemeSpec } from './data.ts'
import { forge } from './forge.ts'

/** Where a theme's file is, from the plugin's folder, or undefined for the forge (drawn in code). */
export function themeFile(root: string, wanted: string): string | undefined {
  if (wanted === '' || wanted === 'forge') return undefined
  if (/[\\/]/.test(wanted) || wanted.endsWith('.json')) return wanted
  // the plugin's folder, whether the root given is it or its .claude-plugin
  const base = root.replace(/[\\/]\.claude-plugin[\\/]?$/, '')
  return `${base}/themes/${wanted}/theme.json`
}

/** The theme a file's text makes, checked; the forge, with the reasons, when it does not. */
export function themeFrom(text: string, wanted: string): { theme: Theme; errors: string[] } {
  let raw: unknown
  try {
    raw = JSON.parse(text)
  } catch (e) {
    return { theme: forge, errors: [`JSON illisible : ${e instanceof Error ? e.message : String(e)}`] }
  }
  const errors = checkTheme(raw)
  if (errors.length > 0) return { theme: forge, errors }
  return { theme: dataTheme(raw as ThemeSpec, wanted), errors: [] }
}
