// Every moment a theme draws, as the engine plays it: what the previews show.
// The planche (apercu.mts), the live mire (mire.mts) and the coverage test all
// read this list, so a moment added to the engine and forgotten here fails the test.

import type { Activity } from '../types'
import { ROAMING } from '../hooks/world.ts'
import type { World } from '../hooks/world.ts'

export type Moment = {
  label: string
  activity: Activity
  world?: Partial<World>
  /** The tick the planche shows: the moment caught mid-gesture. */
  shot: number
  /** A one-shot: how many ticks it plays before the engine moves on (150 ms each). */
  once?: number
}

export const MOMENTS: readonly Moment[] = [
  { label: 'repos', shot: 0, activity: 'idle' },
  { label: 'réfléchit', shot: 4, activity: 'thinking' },
  { label: 'travaille', shot: 3, activity: 'running', world: { flavor: 'forge' } },
  { label: 'travaille (compile)', shot: 5, activity: 'running', world: { flavor: 'build', effort: 2 } },
  { label: 'examine', shot: 2, activity: 'review', world: { flavor: 'web' } },
  { label: 'attend la personne', shot: 1, activity: 'waiting' },
  { label: 'échec', shot: 2, activity: 'failed', once: 17 },
  { label: 'réussite', shot: 3, activity: 'jumping', once: 12 },
  { label: 'salue', shot: 2, activity: 'waving', once: 12 },
  { label: 'tests réussis', shot: 12, activity: 'quench', world: { outcome: 'pass', flavor: 'test' }, once: 17 },
  { label: 'tests ratés', shot: 12, activity: 'quench', world: { outcome: 'fail', flavor: 'test' }, once: 17 },
  { label: 'commit', shot: 5, activity: 'ship', world: { ship: 'commit', flavor: 'git' }, once: 16 },
  { label: 'push', shot: 10, activity: 'ship', world: { ship: 'push', flavor: 'git' }, once: 23 },
  { label: 'balaie (compactage)', shot: 3, activity: 'sweep', world: { ctx: 90 } },
  { label: 'dort', shot: 4, activity: 'sleep', world: { dream: 'fusee', hour: 23 } },
  { label: 'se réveille', shot: 2, activity: 'waking', once: 7 },
]

/** The moments not shown, each with why: an exemption is a debt, named. */
export const NOT_SHOWN: Partial<Record<Activity, string>> = {
  away: 'le personnage est parti en visite : sa place est vide, il n\'y a rien à dessiner',
}

/** Whether the engine walks the character during a moment (then a theme shows its walk). */
export const roams = (m: Moment) => ROAMING.includes(m.activity)

/** A moment found by its label or its activity (`sweep`, `balaie`...). */
export const momentOf = (name: string) =>
  MOMENTS.find(m => m.label === name) ?? MOMENTS.find(m => m.activity === name) ?? MOMENTS.find(m => m.label.startsWith(name))
