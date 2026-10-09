// Charger un thème pour les outils : « forge », ou un fichier de thème, vérifié.

import { readFileSync } from 'node:fs'

import { forge } from '../hooks/themes/forge.ts'
import { checkTheme, dataTheme } from '../hooks/themes/data.ts'
import type { ThemeSpec } from '../hooks/themes/data.ts'
import type { Theme } from '../hooks/world.ts'

/** A theme by what the person gives: « forge », or a theme file, checked; the errors when it is refused. */
export function loadTheme(source: string): Theme | string[] {
  if (source === 'forge') return forge
  let raw: unknown
  try {
    raw = JSON.parse(readFileSync(source, 'utf8'))
  } catch (e) {
    return [`${source} ne se lit pas : ${e instanceof Error ? e.message : String(e)}`]
  }
  const errors = checkTheme(raw)
  return errors.length > 0 ? errors : dataTheme(raw as ThemeSpec, source)
}
