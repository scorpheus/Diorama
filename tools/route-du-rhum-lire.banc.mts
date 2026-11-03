// Le banc de la lecture du tracker, sur un vrai pointage : node --test tools/route-du-rhum-lire.banc.mts
//
// bancs/route-du-rhum-2026-11-03.json est un instantané de carto.routedurhum.com pris le 3 novembre 2026 à
// 8 h 17 (pointages de 7 h et 8 h UTC, et celui du départ), réduit aux champs que LIRE consulte. Ce jour-là
// le tracker range ses pointages sous `reports` ; `_reports` n'existe plus. Le relevé ne trouvait donc
// aucune ligne de pointage depuis le départ : ni rang ni distance, et la session retombait sur le temps.

import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

import { LIRE } from './route-du-rhum-lire.mts'

const tracker = JSON.parse(readFileSync(new URL('./bancs/route-du-rhum-2026-11-03.json', import.meta.url), 'utf8'))

test('le dernier pointage réel donne à chaque bateau en course son rang et sa distance', () => {
  assert.equal('_reports' in tracker, false, 'le banc serait creux : l\'instantané a l\'ancien champ')
  const lu: any = LIRE(null, tracker)
  assert.equal(lu.pas, undefined)
  assert.equal(lu.pointage, 1793689200, 'le pointage de 8 h (07:00 UTC), le dernier')
  const enCourse = lu.bateaux.filter((b: any) => b.statut === 'RAC')
  const avecRang = lu.bateaux.filter((b: any) => typeof b.rang === 'number' && b.rang > 0)
  const avecDtf = lu.bateaux.filter((b: any) => typeof b.dtf === 'number')
  console.log(`${lu.bateaux.length} bateaux, ${enCourse.length} en course, ${avecRang.length} avec un rang, ${avecDtf.length} avec une distance`)
  assert.ok(enCourse.length >= 100, `en course : ${enCourse.length}`)
  assert.ok(avecRang.length >= enCourse.length, `rangs : ${avecRang.length} pour ${enCourse.length} en course`)
  assert.ok(avecDtf.length >= enCourse.length, `distances : ${avecDtf.length} pour ${enCourse.length} en course`)
  // un premier par classe, avec sa route
  const premiers = lu.bateaux.filter((b: any) => b.rang === 1)
  const classes = new Set(lu.bateaux.map((b: any) => b.classe))
  assert.equal(premiers.length, classes.size, `premiers : ${premiers.map((b: any) => `${b.classe} ${b.nom}`).join(', ')}`)
  for (const p of premiers) assert.ok(Array.isArray(p.trace) && p.trace.length > 1, `la route de ${p.nom}`)
  // le premier Ultim est loin de Saint-Malo : moins de 3 542 milles à faire
  const ultim = premiers.find((b: any) => /^ultim/i.test(b.classe))
  assert.ok(ultim && ultim.dtf < 3542, `premier Ultim : ${ultim?.nom} à ${ultim?.dtf} milles`)
})

test('un instant rejoué prend le pointage d\'avant, pas le dernier', () => {
  const lu: any = LIRE(1793689200 - 60, tracker)
  assert.equal(lu.pointage, 1793685600, 'le pointage de 7 h (06:00 UTC)')
  assert.ok(lu.bateaux.some((b: any) => b.rang === 1))
})
