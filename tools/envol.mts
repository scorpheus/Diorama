// Un vol d'essai du passeur (passeur/passeur.ps1) : le personnage d'un thème
// s'envole de la fenêtre d'une session Claude à celle d'une autre, sans attendre
// une vraie visite. Windows seulement.
//
//   node tools/envol.mts <theme.json | forge> --de <session | pid> --vers <session | pid>
//       [--cape ardoise] [--accessoire lunettes] [--colonnes 144] [--lignes 16]
//
// Sans --de ni --vers : la liste des sessions ouvertes, à choisir. --colonnes est
// la largeur de la scène dans le bandeau (la forge s'élargit avec la fenêtre) ;
// par défaut, celle du thème sans élargissement. Node 22.6 ou plus.

import { execFileSync } from 'node:child_process'
import { readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { spriteOf } from '../hooks/flight.ts'
import { CALM } from '../hooks/world.ts'
import type { Theme } from '../hooks/world.ts'
import { loadTheme } from './charge.mts'

const args = process.argv.slice(2)
const flag = (name: string, fallback: string) => {
  const i = args.indexOf(name)
  return i >= 0 ? args.splice(i, 2)[1] ?? fallback : fallback
}
const from = flag('--de', '')
const to = flag('--vers', '')
const capeName = flag('--cape', '')
const accessory = flag('--accessoire', 'aucun')
const columns = flag('--colonnes', '')
const rows = flag('--lignes', '')
const [source] = args

if (!source || !from || !to) {
  console.error('usage : node tools/envol.mts <theme.json | forge> --de <session | pid> --vers <session | pid> [--cape nom] [--accessoire nom] [--colonnes N] [--lignes N]')
  const dir = join(homedir(), '.claude', 'sessions')
  const sessions = readdirSync(dir).filter(f => f.endsWith('.json')).flatMap(f => {
    try {
      return [JSON.parse(readFileSync(join(dir, f), 'utf8')) as { pid: number; sessionId: string; name?: string; cwd?: string; kind?: string }]
    } catch {
      return []
    }
  })
  console.error('\nsessions ouvertes (pid, nom, dossier) :')
  for (const s of sessions) console.error(`  ${String(s.pid).padStart(6)}  ${(s.name ?? '').padEnd(24)} ${s.cwd ?? ''}${s.kind && s.kind !== 'interactive' ? `  (${s.kind})` : ''}`)
  process.exit(2)
}

const loaded = loadTheme(source)
if (Array.isArray(loaded)) {
  console.error(`${loaded.length} erreur(s) dans ${source} :`)
  for (const e of loaded) console.error(`  - ${e}`)
  process.exit(1)
}
const theme: Theme = loaded
const variants = theme.looks.variants
const cape = variants[capeName] ?? Object.values(variants)[0]
if (capeName && !variants[capeName]) console.error(`« ${capeName} » n'est pas une cape du thème : ${Object.keys(variants).join(', ')}`)

const sprite = spriteOf(theme, { ...CALM, cape: cape?.colors ?? CALM.cape, accessory })
if (!sprite) {
  console.error(`${theme.title} : le personnage ne se découpe pas de sa scène (son absence change autre chose), pas de vol possible.`)
  process.exit(1)
}
const [x0, x1] = theme.crops.full
const file = join(tmpdir(), 'diorama-envol.json')
writeFileSync(file, JSON.stringify(sprite))
console.log(`${theme.title} : ${sprite.w} x ${sprite.h} pixels, ${sprite.frames.length} images`)

const script = join(dirname(fileURLToPath(import.meta.url)), '..', 'passeur', 'passeur.ps1')
const out = execFileSync('powershell.exe', [
  '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', script, '-Essai', file, '-De', from, '-Vers', to,
  '-Colonnes', columns || String(x1 - x0), '-Lignes', rows || String(theme.rows),
], { encoding: 'utf8' })
console.log(out.trim())
