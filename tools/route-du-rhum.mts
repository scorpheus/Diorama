// La Route du Rhum ne se montre que dans une session de la machine : celle que nomme le
// fichier diorama-route-du-rhum.txt, dans le dossier de Claude Code (~/.claude, ou
// CLAUDE_CONFIG_DIR). Cet outil l'écrit ; un orchestrateur peut aussi l'écrire lui-même.
//
//   node tools/route-du-rhum.mts                          les sessions ouvertes, et celle qui la montre
//   node tools/route-du-rhum.mts <pid | session | nom>    celle-ci la montre, et plus aucune autre
//   node tools/route-du-rhum.mts --aucune                 aucune ne la montre
//
// Les sessions la voient dans les deux secondes. Une session lancée avec
// DIORAMA_ROUTE_DU_RHUM=1 dans son environnement la montre aussi, quoi que dise le fichier.
// Node 22.6 ou plus.

import { readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'

const config = process.env.CLAUDE_CONFIG_DIR ?? join(homedir(), '.claude')
const file = join(config, 'diorama-route-du-rhum.txt')
type Session = { pid: number; sessionId: string; name?: string; cwd?: string; kind?: string }

const sessions: Session[] = (() => {
  try {
    return readdirSync(join(config, 'sessions')).filter(f => f.endsWith('.json')).flatMap(f => {
      try {
        return [JSON.parse(readFileSync(join(config, 'sessions', f), 'utf8')) as Session]
      } catch {
        return []
      }
    })
  } catch {
    return []
  }
})()
const named = (() => {
  try {
    return readFileSync(file, 'utf8').replace(/^﻿/, '').trim().split(/\s+/)[0] ?? ''
  } catch {
    return ''
  }
})()
/** The session a word names, as the plugin reads it: its id, its pid, or its name. */
const find = (word: string) =>
  sessions.find(s => s.sessionId === word || String(s.pid) === word || (s.name !== undefined && s.name.toLowerCase() === word.toLowerCase()))

const [word] = process.argv.slice(2)
if (word === undefined) {
  const holder = named ? find(named) : undefined
  console.log(named ? `la Route du Rhum se montre dans : ${holder ? `${holder.pid} ${holder.name ?? ''}`.trim() : `« ${named} » (aucune session ouverte de ce nom)`}` : 'la Route du Rhum ne se montre dans aucune session')
  console.log('\nsessions ouvertes (pid, nom, dossier) :')
  for (const s of sessions) console.log(`${s === holder ? '⛵' : '  '}${String(s.pid).padStart(6)}  ${(s.name ?? '').padEnd(24)} ${s.cwd ?? ''}${s.kind && s.kind !== 'interactive' ? `  (${s.kind})` : ''}`)
  console.log('\nnode tools/route-du-rhum.mts <pid | session | nom>   pour la montrer ailleurs ; --aucune pour personne')
} else if (word === '--aucune') {
  rmSync(file, { force: true })
  console.log('la Route du Rhum ne se montre plus dans aucune session')
} else {
  const s = find(word)
  // the session's id when the word names one now: it outlives a pid, and a name changes
  writeFileSync(file, `${s?.sessionId ?? word}\n`)
  console.log(s ? `la Route du Rhum se montre dans la session ${s.pid}${s.name ? ` (${s.name})` : ''}` : `aucune session ouverte ne s'appelle « ${word} » : celle qui portera ce nom, cet identifiant ou ce pid la montrera`)
}
