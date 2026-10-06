/** What Scorpheus is doing; each is a row of the Codex pet sheet. */
export type Activity =
  | 'idle' | 'thinking' | 'running' | 'review' | 'waiting' | 'failed' | 'jumping' | 'waving'
  | 'quench' | 'ship' | 'sweep' | 'sleep' | 'waking' | 'away'

/** What kind of work the hammer is doing: it colours the sparks and names the moment. */
export type Flavor = 'forge' | 'build' | 'test' | 'shell' | 'git' | 'web' | 'agent' | 'mcp'

/** One rate-limit window as the band draws it. */
export type Gauge = { kind: string; percent: number; resetsAt?: string }

/** The figures the gauges draw, from `session.measure`. */
export type Usage = {
  startedAt?: number
  ctx?: number
  tokens?: number
  window?: number
  limits: Gauge[]
  usd?: number
}

/**
 * A subagent at work: a violet wisp in the scene, a line in the words. `id` is
 * the Agent call's tool_use_id, `agentId` the subagent's own (its loop's events
 * and its task notification carry it).
 */
export type Apprentice = { id: string; agentId?: string; type: string; description: string; background: boolean; name?: string }

/** A command left running in the background: its task id, the call that started it, its first word. */
export type Job = { id: string; toolUseId?: string; label: string }

/** Who works beside Scorpheus: apprentices, and commands left running in the background. */
export type Crew = { apprentices: Apprentice[]; jobs: Job[] }

/** A day at the forge, kept across sessions for the week's rack. */
export type Day = { date: string; tests: number; builds: number; edits: number; shells: number; ticks: number }

/** The session's experience: points, and the level they make. */
export type Experience = { xp: number; level: number }

/**
 * Where the conversation's task stands, for the words beside the scene: `goal` the
 * task the person asked for, `line` what is being done (or, once `done`, what was
 * done and what is left), `at` when that line was written.
 */
export type Status = { goal: string; line: string; at: number; done: boolean }

/** The pose in play and the words under the name. */
export type Act = { activity: Activity; detail: string; flavor?: Flavor }

declare module 'claude-code' {
  interface PluginState {
    'diorama': { usage: Usage | null; act: Act; crew: Crew; speech: string | null; xp: Experience; log: string[]; status: Status | null }
  }
}
