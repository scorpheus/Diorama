// Le pointage de la Route du Rhum : lit la flotte sur le tracker GeoVoile de la course et
// l'écrit dans diorama-route-du-rhum.json, dans le dossier de Claude Code (~/.claude, ou
// CLAUDE_CONFIG_DIR), au format que lit la session désignée (README, « La Route du Rhum »).
//
//   node tools/route-du-rhum-suivre.mts                 toutes les 15 minutes, jusqu'à ce qu'on l'arrête
//   node tools/route-du-rhum-suivre.mts --une-fois      un pointage, puis s'arrête (pour une tâche planifiée)
//   node tools/route-du-rhum-suivre.mts --minutes 30    un autre rythme (10 au moins)
//   node tools/route-du-rhum-suivre.mts --annee 2022 --instant 2022-11-12T14:00Z --une-fois
//                                                       rejoue un instant d'une édition passée
//   node tools/route-du-rhum-suivre.mts --sortie <fichier>   écrit ailleurs (--sortie - : à l'écran)
//   node tools/route-du-rhum-suivre.mts --url <page>    une autre page du tracker
//
// En 2026, la carte officielle n'est plus sur routedurhum.geovoile.com (« Mise en ligne à venir »
// jusqu'au départ) mais sur carto.routedurhum.com, le même tracker GeoVoile sous le nom de la course.
//
// Les positions arrivent chiffrées du serveur de GeoVoile ; seul leur JavaScript les décode. On
// charge donc la carte dans Edge (ou Chrome) sans fenêtre, par le protocole de ses outils de
// développement, et on lit l'objet décodé en mémoire : `tracker._boats`, ses classes, ses
// pointages. Rien à installer. DIORAMA_NAVIGATEUR donne un autre navigateur Chromium.
// Le fichier s'écrit à côté puis se renomme : la session ne lit jamais un fichier à moitié écrit.
// Node 22.6 ou plus.

import { spawn } from 'node:child_process'
import { existsSync, mkdtempSync, renameSync, rmSync, writeFileSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { join } from 'node:path'

const args = process.argv.slice(2)
const option = (nom: string) => {
  const i = args.indexOf(nom)
  return i >= 0 ? args[i + 1] : undefined
}
const annee = option('--annee') ?? '2026'
const instant = option('--instant')
const quand = instant === undefined ? undefined : Date.parse(instant)
if (quand !== undefined && Number.isNaN(quand)) throw new Error(`--instant : « ${instant} » n'est pas une date`)
const minutes = Math.max(10, Number(option('--minutes') ?? 15))
const uneFois = args.includes('--une-fois')
const config = process.env.CLAUDE_CONFIG_DIR ?? join(homedir(), '.claude')
const sortie = option('--sortie') ?? join(config, 'diorama-route-du-rhum.json')
const url = option('--url') ?? (annee === '2026' ? 'https://carto.routedurhum.com/fr/index.html' : `https://routedurhum.geovoile.com/${annee}/tracker/`)
const SAINT_MALO = { lat: 48.65, lon: -2.02 }

const navigateur = [
  process.env.DIORAMA_NAVIGATEUR,
  'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Google/Chrome/Application/chrome.exe',
  '/usr/bin/chromium',
  '/usr/bin/google-chrome',
].find(p => p !== undefined && existsSync(p))

const dormir = (ms: number) => new Promise(r => setTimeout(r, ms))
const heure = () => new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })

/**
 * Ce qu'on lit dans la page, une fois le tracker chargé : chaque bateau, sa classe, sa position
 * au pointage, et la ligne du pointage (rang, distance à l'arrivée, vitesse, statut). `quand` :
 * l'instant à rejouer, en secondes, ou null pour le dernier pointage.
 */
const LIRE = (quand: number | null) => {
  const t = (window as any).tracker
  if (!t || !Array.isArray(t._boats) || t._boats.length === 0) return { pas: 'pas de tracker dans la page' }
  const classe: Record<string, string> = {}
  for (const c of Object.values<any>(t._boatClassesByIds ?? {})) for (const b of c.boats ?? []) classe[b.id] = c.name
  const pointages: any[] = t._reports ?? []
  const pointage = quand === null ? pointages[pointages.length - 1] : [...pointages].reverse().find(p => p.timecode <= quand)
  const lignes: Record<string, any> = {}
  for (const l of pointage?.lines ?? []) lignes[l.boat] = l
  // la route d'un premier de classe : au plus une quarantaine de points, jusqu'au pointage
  const trace = (b: any) => {
    const locs: any[] = (b.track?.locations ?? []).filter((l: any) => quand === null || l.timecode <= quand)
    const pas = Math.max(1, Math.ceil(locs.length / 40))
    const gardes = locs.filter((_, i) => i % pas === 0 || i === locs.length - 1)
    return gardes.map(l => [Math.round(l.lat * 100) / 100, Math.round(l.lng * 100) / 100])
  }
  const position = (b: any) => {
    const locs: any[] = b.track?.locations ?? []
    const loc = quand === null ? b.track?.lastLocation : [...locs].reverse().find(l => l.timecode <= quand)
    return loc && typeof loc.lat === 'number' ? { lat: loc.lat, lon: loc.lng } : null
  }
  return {
    etat: t._eventState,
    depart: t.timecodeOfficialStart,
    pointage: pointage?.timecode ?? null,
    bateaux: t._boats.map((b: any) => {
      const l = lignes[b.id]
      const marin = b.sailors?.[0]
      return {
        nom: String(b.name ?? '').trim(),
        skipper: marin ? `${marin.fname ?? ''} ${marin.lname ?? ''}`.trim() : null,
        classe: classe[b.id] ?? null,
        coque: b.hullColors ?? [],
        couleur: b.mainColor ?? b.track?.color ?? null,
        position: position(b),
        rang: l?.rank ?? null,
        dtf: l?.dtf ?? null,
        vitesse: l?.speed ?? null,
        statut: l?.racestatus ?? null,
        trace: l?.rank === 1 && /^(RAC|ARV)$/.test(l?.racestatus ?? '') ? trace(b) : null,
      }
    }),
  }
}

/** La page du tracker chargée dans un navigateur sans fenêtre, et ce que LIRE en tire. */
async function lireLeTracker(): Promise<ReturnType<typeof LIRE>> {
  if (!navigateur) throw new Error('ni Edge ni Chrome trouvé (DIORAMA_NAVIGATEUR pour en donner un)')
  const profil = mkdtempSync(join(tmpdir(), 'diorama-rhum-'))
  const port = 9400 + Math.floor(Math.random() * 500)
  const p = spawn(navigateur, ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${profil}`, '--no-first-run', '--no-default-browser-check', '--window-size=1440,900', 'about:blank'], { stdio: 'ignore' })
  let ws: WebSocket | undefined
  try {
    let page: { webSocketDebuggerUrl: string } | undefined
    for (let i = 0; i < 75 && !page; i++) {
      await dormir(200)
      page = await fetch(`http://127.0.0.1:${port}/json`).then(r => r.json()).then((l: any[]) => l.find(t => t.type === 'page')).catch(() => undefined)
    }
    if (!page) throw new Error('le navigateur ne répond pas')
    ws = new WebSocket(page.webSocketDebuggerUrl)
    await new Promise((ok, ko) => { ws!.addEventListener('open', ok); ws!.addEventListener('error', ko) })
    let n = 0
    const attente = new Map<number, (m: any) => void>()
    ws.addEventListener('message', e => {
      const m = JSON.parse(String(e.data))
      attente.get(m.id)?.(m)
    })
    const cdp = (method: string, params: object = {}) =>
      new Promise<any>(r => {
        attente.set(++n, r)
        ws!.send(JSON.stringify({ id: n, method, params }))
      })
    await cdp('Page.navigate', { url })
    // le tracker charge ses traces et ses pointages après la page : on attend qu'il les ait
    const expression = `(${LIRE.toString()})(${quand === undefined ? 'null' : Math.floor(quand / 1000)})`
    let lu: any = { pas: 'la page ne charge pas' }
    for (let i = 0; i < 30; i++) {
      await dormir(2000)
      const r = await cdp('Runtime.evaluate', { expression, returnByValue: true })
      lu = r.result?.result?.value ?? { pas: r.result?.exceptionDetails?.text ?? 'rien' }
      if (!lu.pas && (lu.pointage !== null || lu.etat === 'STANDBY' || lu.etat === 'PRERACE')) break
    }
    return lu
  } finally {
    ws?.close()
    // Edge fermé pour de bon avant d'ôter son profil : sous Windows, il le tient encore un moment,
    // et un profil qui résiste ne doit pas faire perdre un pointage déjà lu
    const ferme = new Promise(r => p.once('exit', r))
    p.kill()
    await Promise.race([ferme, dormir(5000)])
    try {
      rmSync(profil, { recursive: true, force: true, maxRetries: 10, retryDelay: 300 })
    } catch (e) {
      console.log(`${heure()}  profil ${profil} laissé : ${e instanceof Error ? e.message : e}`)
    }
  }
}

/** Les classes du tracker sous les noms du plugin (« Ultim 32/23 » : Ultim). */
function classeDe(nom: string | null): string | undefined {
  const k = (nom ?? '').toLowerCase().replace(/[^a-z0-9]/g, '')
  if (k.startsWith('ultim')) return 'Ultim'
  if (k.includes('fifty') || k.startsWith('ocean50')) return 'Ocean Fifty'
  if (k.startsWith('imoca')) return 'IMOCA'
  if (k.startsWith('class40')) return 'Class40'
  if (k.startsWith('rhummulti') || k.startsWith('vintagemulti')) return 'Vintage Multi'
  if (k.startsWith('rhummono') || k.startsWith('vintagemono')) return 'Vintage Mono'
  return undefined
}

/** La coque, puis la bande : la première couleur de la coque, et une autre qui s'en distingue. */
function couleursDe(coque: string[], trace: string | null): [string, string] | undefined {
  const c = coque.filter(x => /^[0-9a-f]{6}$/i.test(x))
  const fond = c[0] ?? trace
  // une bande qui se voit sur la coque : #FEFEFE sur #FFFFFF n'en est pas une
  const ecart = (a: string, b: string) => [0, 2, 4].reduce((s, i) => s + Math.abs(parseInt(a.slice(i, i + 2), 16) - parseInt(b.slice(i, i + 2), 16)), 0)
  const bande = [trace, c[2], c[1], c[3]].find(x => x && /^[0-9a-f]{6}$/i.test(x) && (!fond || ecart(x, fond) > 60))
  return fond && bande ? [`#${fond}`, `#${bande}`] : undefined
}

const arrondi = (v: number, n: number) => Math.round(v * 10 ** n) / 10 ** n

async function pointer(): Promise<void> {
  const lu: any = await lireLeTracker()
  if (lu.pas) {
    console.log(`${heure()}  ${url} : ${lu.pas} ; rien d'écrit`)
    return
  }
  const inconnues = new Map<string, number>()
  const bateaux = lu.bateaux.flatMap((b: any) => {
    const classe = classeDe(b.classe)
    if (!classe) inconnues.set(String(b.classe), (inconnues.get(String(b.classe)) ?? 0) + 1)
    if (!classe || !b.nom) return []
    // avant le départ, les inscrits sans position sont au port
    const pos = b.position ?? (lu.pointage === null ? SAINT_MALO : null)
    if (!pos) return []
    const couleurs = couleursDe(b.coque, b.couleur)
    return [{
      nom: b.nom,
      ...(b.skipper ? { skipper: b.skipper } : {}),
      classe,
      ...(couleurs ? { couleurs } : {}),
      lat: arrondi(pos.lat, 3),
      lon: arrondi(pos.lon, 3),
      ...(typeof b.rang === 'number' && b.rang > 0 ? { rang: b.rang } : {}),
      ...(typeof b.dtf === 'number' ? { dtf: arrondi(b.dtf, 1) } : {}),
      ...(typeof b.vitesse === 'number' ? { vitesse: arrondi(b.vitesse, 1) } : {}),
      ...(b.statut ? { statut: b.statut } : {}),
      ...(Array.isArray(b.trace) && b.trace.length > 1 ? { trace: b.trace } : {}),
    }]
  })
  // une classe que le plugin ne connaît pas ne s'écrit pas : on le dit, pour qu'elle ne manque pas en silence
  if (inconnues.size > 0) console.log(`${heure()}  classes laissées de côté : ${[...inconnues].map(([c, n]) => `${c} (${n})`).join(', ')}`)
  if (bateaux.length === 0) {
    console.log(`${heure()}  ${url} : aucun bateau lisible (état ${lu.etat}) ; rien d'écrit`)
    return
  }
  const at = new Date((lu.pointage ?? Math.floor(Date.now() / 1000)) * 1000).toISOString().replace(/\.\d+Z$/, 'Z')
  const texte = `${JSON.stringify({ at, source: url, bateaux }, null, 1)}\n`
  const parClasse = Object.entries(Object.groupBy(bateaux, (b: any) => b.classe)).map(([c, l]) => `${c} ${l!.length}`).join(', ')
  if (sortie === '-') {
    process.stdout.write(texte)
    return
  }
  const cote = `${sortie}.${process.pid}.tmp`
  writeFileSync(cote, texte)
  renameSync(cote, sortie)
  // le départ que donne le tracker, à comparer à celui du plugin (RHUM_START, 1er novembre 12h02 UTC)
  const depart = typeof lu.depart === 'number' ? `, départ ${new Date(lu.depart * 1000).toISOString().replace(/:00\.000Z$/, 'Z')}` : ''
  console.log(`${heure()}  pointage de ${at} (état ${lu.etat}${depart}) : ${bateaux.length} bateaux (${parClasse}) → ${sortie}`)
}

for (;;) {
  try {
    await pointer()
  } catch (e) {
    console.log(`${heure()}  ${url} : ${e instanceof Error ? e.message : e}`)
  }
  if (uneFois || quand !== undefined) break
  await dormir(minutes * 60_000)
}
