// Ce que route-du-rhum-suivre.mts lit dans la page du tracker GeoVoile. La fonction est envoyée telle
// quelle à la page (par son texte) : elle ne doit rien utiliser d'autre que ses arguments et `window`.
// Elle vit à part pour que le banc (route-du-rhum-lire.banc.mts) la fasse tourner sur un vrai pointage.

/**
 * Ce qu'on lit dans la page, une fois le tracker chargé : chaque bateau, sa classe, sa position
 * au pointage, et la ligne du pointage (rang, distance à l'arrivée, vitesse, statut). `quand` :
 * l'instant à rejouer, en secondes, ou null pour le dernier pointage. `tracker` : celui de la page,
 * ou celui d'un pointage enregistré (le banc).
 */
export const LIRE = (quand: number | null, tracker?: any) => {
  const t = tracker ?? (globalThis as any).window?.tracker
  if (!t || !Array.isArray(t._boats) || t._boats.length === 0) return { pas: 'pas de tracker dans la page' }
  const classe: Record<string, string> = {}
  for (const c of Object.values<any>(t._boatClassesByIds ?? {})) for (const b of c.boats ?? []) classe[b.id] = c.name
  // les pointages : `reports` sur la carte 2026 (relevé du 3 novembre : `_reports` n'existe plus, et sans lui le
  // relevé n'avait ni rang ni distance depuis le départ), `_reports` sur les trackers des éditions passées ; du
  // plus ancien au plus récent, quel que soit l'ordre du tracker
  const pointages: any[] = [...(Array.isArray(t.reports) ? t.reports : Array.isArray(t._reports) ? t._reports : [])]
    .filter(p => typeof p?.timecode === 'number')
    .sort((a, b) => a.timecode - b.timecode)
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
