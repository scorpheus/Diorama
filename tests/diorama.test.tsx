import { expect, mock, test } from 'claude-code/testing'
import type { Engine } from 'claude-code/testing'
import type { AgentStatus, On, ToolCallInput, ToolCallResult } from 'claude-code'

import { CALM, SMITHS, frame } from '../hooks/themes/forge.ts'
import { checkTheme, dataTheme } from '../hooks/themes/data.ts'
import type { ThemeSpec } from '../hooks/themes/data.ts'

const BAND = {
  hasSurvey: false,
  isWorking: false,
  maxRows: 20,
  bodyColumns: 130,
  scroll: { offset: 0, bodyRows: 20 },
  view: {},
}

const typed = (text: string) => ({ text, wait: false, origin: { kind: 'composer' as const } })
const ended = (reason: 'answer' | 'aborted', agentId?: string) =>
  ({ reason, answer: '', durationMs: 1000, isAborted: reason === 'aborted', turnId: 't1', agentId })
const notified = (text: string) => ({ text, wait: false, origin: { kind: 'task-notification' as const } })
const spawned = (id: string, subagentType: string, description: string, background: boolean) => ({
  tool_use_id: id, prompt: 'p', description, subagentType, background, fork: false,
  provider: { plugin: 'engine', tier: 'core' as const }, parentModel: 'claude-opus-5-5',
})

/** The smith's name in the line under the scene: one of the forge's, never the person's. */
const SMITH = new RegExp(`^(${SMITHS.join('|')})$`)

const IN_TWO_HOURS = new Date(Date.UTC(2026, 9, 2, 14, 0)).toISOString()

/** The engine's own answers, beneath the plugin; blits are recorded. */
const blits: string[] = []
let duringCompact: (() => Promise<void>) | undefined
const shelf = new Map<string, unknown>()
let listed: Array<{ id: string; description: string; type: string; status: AgentStatus }> = []
let sessionId = 'sess-1'
function engine(on: On, stored: Readonly<Record<string, unknown>> = {}) {
  blits.length = 0
  duringCompact = undefined
  on('session.compact', async () => {
    await duringCompact?.()
    return { skip: 'test' }
  })
  on('agent.spawn', (_$, e) => ({ model: 'claude-haiku-4-5', agentId: `ag-${e.tool_use_id}` }))
  on('prompt.attachment', (_$, e) => ({ text: e.text }))
  on('agent.list', () => ({ value: listed }))
  on('ui.blit', (_$, e) => {
    if ('cells' in e) blits.push(e.cells)
    return { value: {} }
  })
  on('session.start', (_$, e) => ({ cwd: e.cwd }))
  on('session.usage', () => ({ value: { startedAt: 0, context: { window: 200_000 }, rateLimits: [] } }))
  on('session.measure', (_$, e) => ({ changed: e.changed }))
  on('session.model', () => ({ value: 'claude-opus-5-5' }))
  on('session.id', () => ({ value: sessionId }))
  on('prompt.submit', (_$, e) => ({ text: e.text }))
  on('turn.complete', () => ({ text: '' }))
  on('classic.Notification', () => ({}))
  on('classic.UserPromptSubmit', () => ({}))
  on('classic.SessionStart', () => ({}))
  // the store, kept in this map: what the plugin writes, a test reads back
  shelf.clear()
  listed = []
  sessionId = 'sess-1'
  for (const [k, v] of Object.entries(stored)) shelf.set(k, v)
  on('store.get', (_$, e) => ({ value: shelf.get(e.key) }))
  on('store.set', (_$, e) => {
    shelf.set(e.key, e.value)
    return { value: undefined }
  })
  on('store.delete', (_$, e) => {
    shelf.delete(e.key)
    return { value: undefined }
  })
  on('store.keys', () => ({ value: [...shelf.keys()] }))
  on('ui.toast', () => ({ value: undefined }))
  on('session.cwd', () => ({ value: 'C:/projets/vehigraph' }))
  on('turn.step', async function* () {
    return { turnId: 't1', index: 0, answer: '', toolUses: [], stopReason: 'end_turn', usage: null } as never
  })
}

/** One request to the model, read to its end. */
async function step($: Engine, model: string, agentId?: string, effort?: 'low' | 'high' | 'max') {
  for await (const _ of $.turn.step({ turnId: 't1', index: 0, model, messageCount: 1, agentId, effort } as never)) {
    // the chunks themselves are of no interest here
  }
}

type ToolAnswer = (e: ToolCallInput) => ToolCallResult | Promise<ToolCallResult>
const answerTools: ToolAnswer = e =>
  e.tool === 'Bash' && /make all|ctest -R broken/.test(e.command) ? { result: 'boom', isError: true } : { result: 'ok' }

async function start($: Engine, on: On, tools: ToolAnswer = answerTools, stored: Readonly<Record<string, unknown>> = {}) {
  const clock = mock.clock(on, { now: Date.UTC(2026, 9, 2, 12, 0) })
  engine(on, stored)
  on('tool.call', (_$, e) => tools(e))
  await $.session.start({ cwd: 'C:/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'diorama', surface: 'terminal', component: 'AbovePrompt', props: BAND })
  return { ui, clock }
}

/** Glyphs the sprite paints (anything but a blank cell). */
function paintedCells(cells: string): number {
  const bin = Uint8Array.from(atob(cells), c => c.charCodeAt(0))
  const words = new Uint32Array(bin.buffer)
  let n = 0
  for (let i = 0; i < words.length; i += 3) if (words[i] !== 0x20) n++
  return n
}

test('draws the sprite and the gauges from session.measure', async ($, on) => {
  const { ui, clock } = await start($, on)
  await $.session.measure({
    context: { percent: 64, tokens: 128_000, window: 200_000 },
    rateLimits: [
      { kind: 'five_hour', percentUsed: 27, resetsAt: IN_TWO_HOURS },
      { kind: 'seven_day', percentUsed: 41, resetsAt: IN_TWO_HOURS },
    ],
    changed: ['context', 'rateLimits'],
  })

  const raster = await ui.find({ type: 'Raster', key: 'diorama' })
  // 130 columns: the whole forge and the apprentices' yard, 72 wide, 16 rows
  expect(raster?.props.columns).toBe(72)
  expect(raster?.props.rows).toBe(16)
  // output contract: a blank or near-blank frame is a broken scene
  expect(paintedCells(String(raster?.props.cells))).toBeGreaterThan(250)

  expect(await ui.find({ text: ' 64%' })).toBeDefined()
  expect(await ui.find({ text: ' 27%' })).toBeDefined()
  expect(await ui.find({ text: ' 41%' })).toBeDefined()
  expect(await ui.find({ text: '  128k/200k' })).toBeDefined()
  expect(await ui.find({ text: /claude-opus-5-5 · session/ })).toBeDefined()
  // the 5h line says when, not only how long
  const at = new Date(IN_TWO_HOURS)
  const hhmm = `${String(at.getHours()).padStart(2, '0')}h${String(at.getMinutes()).padStart(2, '0')}`
  expect(await ui.find({ text: `  ↻ 2h00 · ${hhmm}` })).toBeDefined()

  // end to end: the timer repaints the mounted Raster, and the frames change
  await clock.advance(1500)
  expect(blits.length).toBeGreaterThanOrEqual(8)
  expect(new Set(blits).size).toBeGreaterThanOrEqual(2)
})

// Coverage contract: every Activity is reached through the events that should raise it.
test('every activity is reachable from real events', { timeoutMs: 60_000 }, async ($, on) => {
  const { ui, clock } = await start($, on)
  const label = async () => (await ui.find({ text: /^ · / }))?.text
  const seen: string[] = []
  const saw = async () => { seen.push(String(await label())) }

  await saw() // waving, from session.start
  await clock.advance(3100)
  await saw() // idle: the greeting's hold ran out, so session.start ran

  await $.prompt.submit(typed('go'))
  await saw() // thinking
  await $.tool.call({ tool: 'Edit', file_path: 'a.ts', old_string: 'a', new_string: 'b' })
  await saw() // running
  await step($, 'claude-opus-5-5')
  await saw() // thinking again: the model is asked once more
  await $.tool.call({ tool: 'Read', file_path: 'C:/x/notes.md' })
  await saw() // review
  expect(await ui.find({ text: ' · Read notes.md' })).toBeDefined()

  const broke = await $.tool.call({ tool: 'Bash', command: 'make all' })
  expect(broke.isError).toBe(true)
  await saw() // failed
  await clock.advance(2600)

  await $.tool.call({ tool: 'Bash', command: 'ctest --output-on-failure' })
  await saw() // quench
  await clock.advance(2700)

  await $.tool.call({ tool: 'Bash', command: 'git commit -m "forge"' })
  await saw() // ship
  await clock.advance(2500)

  duringCompact = saw
  await $.session.compact({ trigger: 'manual', messages: [] } as never) // sweep, while it runs

  await $.turn.complete(ended('answer'))
  await saw() // jumping
  await clock.advance(1900)

  await clock.advance(10 * 60 * 1000)
  await saw() // sleep: ten quiet minutes
  await $.prompt.submit(typed('debout'))
  await saw() // waking

  expect(seen).toEqual([
    ' · salut !', ' · au calme', ' · réfléchit', ' · à la forge', ' · réfléchit', ' · inspecte', ' · aïe…', ' · trempe la lame',
    " · expédie l'ouvrage", " · balaie l'atelier", ' · ouvrage fini !', ' · fait la sieste', ' · hein ?!',
  ])
  await clock.advance(1100)
  expect(await label()).toBe(' · réfléchit') // and back to thinking once awake
})

test('a one-shot plays to its end before the next work shows', async ($, on) => {
  const { ui, clock } = await start($, on)
  const label = async () => (await ui.find({ text: /^ · / }))?.text
  await $.prompt.submit(typed('go'))
  await $.tool.call({ tool: 'Bash', command: 'ctest -R broken' })
  expect(await ui.find({ text: ' · la lame a cassé' })).toBeDefined()
  await $.tool.call({ tool: 'Edit', file_path: 'a.ts', old_string: 'a', new_string: 'b' })
  expect(await label()).toBe(' · trempe la lame') // the edit waits behind the quench
  await clock.advance(2700)
  expect(await label()).toBe(' · à la forge')
})

test("a subagent's turn ending is not the work being done", async ($, on) => {
  const { ui } = await start($, on)
  await $.prompt.submit(typed('go'))
  await $.turn.complete(ended('answer', 'agent-1'))
  expect((await ui.find({ text: /^ · / }))?.text).toBe(' · réfléchit')
})

test("only the main thread's model steps make him think", async ($, on) => {
  const { ui } = await start($, on)
  const label = async () => (await ui.find({ text: /^ · / }))?.text
  await $.prompt.submit(typed('go'))
  await $.tool.call({ tool: 'Edit', file_path: 'a.ts', old_string: 'a', new_string: 'b' })
  await step($, 'claude-haiku-4-5', 'agent-1') // an apprentice's model: not his
  expect(await label()).toBe(' · à la forge')
  await step($, 'claude-opus-5-5')
  expect(await label()).toBe(' · réfléchit')
})

const notice = (taskId: string, toolUseId?: string) =>
  `<task-notification>\n<task-id>${taskId}</task-id>\n${toolUseId ? `<tool-use-id>${toolUseId}</tool-use-id>\n` : ''}<status>completed</status>\n</task-notification>`

// Session dc990cae: 13 of 21 background agents had no notification of their
// own; it came queued into a running turn. Every channel must let them go.
test('apprentices and background commands leave by whichever channel ends them', { timeoutMs: 30_000 }, async ($, on) => {
  const { ui, clock } = await start($, on, e =>
    e.tool === 'Bash' && e.run_in_background === true
      ? { result: 'Command running in background with ID: bz123. Output is being written to: x.output' }
      : { result: 'ok' })
  const count = async () => (await ui.find({ text: /^apprentis/ }))?.text.match(/^apprentis (\d+)/)?.[1] ?? '0'

  for (const id of ['a1', 'a2', 'a3', 'a4', 'a5', 'a6']) await $.agent.spawn(spawned(id, 'general-purpose', `tranche ${id}`, true))
  await $.agent.spawn(spawned('a7', 'Explore', 'revue', false))
  expect(await count()).toBe('7')
  expect(await ui.find({ text: /general-purpose · \d+ min — tranche a1 \(fond\)/ })).toBeDefined()
  expect(await ui.find({ text: '  … et 4 autres' })).toBeDefined() // three by name, the rest counted

  // the foreground one: its Agent call returns
  await $.tool.call({ tool: 'Agent', tool_use_id: 'a7', prompt: 'p', description: 'revue' } as never)
  expect(await count()).toBe('6')
  // a notification as a prompt of its own, by its tool-use-id
  await $.prompt.submit(notified(notice('ag-a1', 'a1')))
  expect(await count()).toBe('5')
  // a notification queued into a running turn: an attachment, by its task id
  await $.prompt.attachment({ type: 'queued_command', text: notice('ag-a2'), origin: { kind: 'engine' } } as never)
  expect(await count()).toBe('4')
  // the subagent's own turn ends
  await $.turn.complete(ended('answer', 'ag-a3'))
  expect(await count()).toBe('3')
  // stopped by hand
  await $.tool.call({ tool: 'TaskStop', task_id: 'ag-a4' } as never)
  expect(await count()).toBe('2')
  // the engine's own list says it is over: the half-minute check
  listed = [{ id: 'ag-a5', description: 'tranche a5', type: 'general-purpose', status: 'completed' },
    { id: 'ag-a6', description: 'tranche a6', type: 'general-purpose', status: 'running' }]
  await clock.advance(31_000)
  expect(await count()).toBe('1')
  expect(await ui.find({ text: /— tranche a6 \(fond\)/ })).toBeDefined() // the one still running stays

  // a background command: its hourglass leaves on its notification, by task id
  await $.tool.call({ tool: 'Bash', command: 'cmake --build build', run_in_background: true })
  expect((await ui.find({ text: /^1 commande/ }))?.text).toBe('1 commande · cmake')
  await $.prompt.attachment({ type: 'queued_command', text: notice('bz123'), origin: { kind: 'engine' } } as never)
  expect(await ui.find({ text: /^en fond/ })).toBeUndefined()
})

test("an apprentice's own tools do not move Scorpheus", async ($, on) => {
  const { ui } = await start($, on)
  await $.prompt.submit(typed('go'))
  await $.tool.call({ tool: 'Edit', file_path: 'a.ts', old_string: 'a', new_string: 'b', agentId: 'ag-x' } as never)
  expect((await ui.find({ text: /^ · / }))?.text).toBe(' · réfléchit')
})

// Coverage contract for the world: every field of World changes the picture.
test('every part of the world shows in the scene', () => {
  const paint = (activity: Parameters<typeof frame>[1], tick: number, world: Partial<typeof CALM>) =>
    Array.from(frame('full', activity, tick, { ...CALM, ...world })).join()
  const changes: Record<string, boolean> = {
    apprentices: paint('idle', 0, { apprentices: [{ seed: 3, colour: 'P', age: 20 }] }) !== paint('idle', 0, {}),
    leaving: paint('idle', 0, { apprentices: [{ seed: 3, colour: 'P', age: 20, leaving: 5 }] }) !== paint('idle', 0, { apprentices: [{ seed: 3, colour: 'P', age: 20 }] }),
    hourglasses: paint('idle', 0, { hourglasses: 1 }) !== paint('idle', 0, {}),
    fuel5h: paint('idle', 0, { fuel5h: 80 }) !== paint('idle', 0, { fuel5h: 0 }),
    fuel7d: paint('idle', 0, { fuel7d: 80 }) !== paint('idle', 0, { fuel7d: 0 }),
    ctx: paint('idle', 0, { ctx: 90 }) !== paint('idle', 0, { ctx: 10 }),
    bellAge: paint('idle', 0, { bellAge: 2 }) !== paint('idle', 0, {}),
    pressure: paint('idle', 0, { pressure: 90 }) !== paint('idle', 0, {}),
    flavor: paint('running', 4, { flavor: 'test' }) !== paint('running', 4, {}),
    build: paint('running', 1, { flavor: 'build' }) !== paint('running', 1, {}),
    turnTicks5: paint('running', 1, { turnTicks: 2400 }) !== paint('running', 1, { turnTicks: 2410 }),
    turnTicks15: paint('running', 1, { turnTicks: 6400 }) !== paint('running', 1, { turnTicks: 6410 }),
    outcome: paint('quench', 10, { outcome: 'fail' }) !== paint('quench', 10, { outcome: 'pass' }),
    ship: paint('ship', 8, { ship: 'push' }) !== paint('ship', 8, { ship: 'commit' }),
    cubAge: paint('idle', 0, { apprentices: [{ seed: 3, colour: 'P', age: 10 }] }) !== paint('idle', 0, { apprentices: [{ seed: 3, colour: 'P', age: 19 }] }),
    facing: paint('idle', 1, { x: -6, walking: true, facing: -1 }) !== paint('idle', 1, { x: -6, walking: true, facing: 1 }),
    hour: paint('idle', 0, { hour: 23 }) !== paint('idle', 0, { hour: 12 }),
    dusk: paint('idle', 0, { hour: 19 }) !== paint('idle', 0, { hour: 12 }),
    month: paint('idle', 0, { month: 1 }) !== paint('idle', 0, { month: 6 }),
    day: paint('idle', 0, { month: 12, day: 24 }) !== paint('idle', 0, { month: 12, day: 5 }),
    birthday: paint('idle', 0, { month: 10, day: 2 }) !== paint('idle', 0, { month: 10, day: 3 }),
    storm: paint('idle', 0, { weather: 'storm' }) !== paint('idle', 0, {}),
    rainbow: paint('idle', 0, { weather: 'rainbow' }) !== paint('idle', 0, {}),
    hammer: paint('running', 1, { hammer: 'sledge' }) !== paint('running', 1, { hammer: 'small' }),
    effort: paint('running', 1, { effort: 2 }) !== paint('running', 1, { effort: 0 }),
    usd: paint('idle', 0, { usd: 12 }) !== paint('idle', 0, {}),
    x: paint('idle', 0, { x: -6 }) !== paint('idle', 0, {}),
    walking: paint('idle', 1, { x: -6, walking: true }) !== paint('idle', 1, { x: -6 }),
    thinking: paint('thinking', 0, {}) !== paint('idle', 0, {}),
    guests: paint('idle', 0, { guests: [{ x: 26, walking: false, facing: -1, cape: ['N', 'L'], accessory: 'lunettes', tick: 3, waving: false }] }) !== paint('idle', 0, {}),
    banquet: paint('idle', 0, { banquet: true }) !== paint('idle', 0, {}),
    catRoof: paint('idle', 0, { cat: { mode: 'roof', x: 12, age: 0 } }) !== paint('idle', 0, {}),
    catWalk: paint('idle', 0, { cat: { mode: 'walk', x: 20, age: 0 } }) !== paint('idle', 0, { cat: { mode: 'roof', x: 12, age: 0 } }),
    gift: paint('idle', 0, { gift: { kind: 'ore', age: 1 } }) !== paint('idle', 0, {}),
    messenger: paint('idle', 0, { messenger: { kind: 'waiting', age: 10 } }) !== paint('idle', 0, {}),
    coffee: paint('idle', 0, { routine: 'coffee' }) !== paint('idle', 0, {}),
    lunch: paint('idle', 0, { routine: 'lunch' }) !== paint('idle', 0, {}),
    lantern: paint('idle', 0, { routine: 'lantern' }) !== paint('idle', 0, {}),
    lanternLit: paint('idle', 0, { lanternLit: true }) !== paint('idle', 0, {}),
    dream: paint('sleep', 0, { dream: 'voiture' }) !== paint('sleep', 0, {}),
    jovial: paint('idle', 0, { personality: 'jovial' }) !== paint('idle', 0, {}),
    grognon: paint('idle', 0, { personality: 'grognon' }) !== paint('idle', 0, {}),
    reveur: paint('idle', 0, { personality: 'reveur' }) !== paint('idle', 0, {}),
    applique: paint('idle', 0, { personality: 'applique' }) !== paint('idle', 0, {}),
    masterpiece: paint('idle', 0, { masterpiece: true }) !== paint('idle', 0, {}),
    star: paint('idle', 0, { hour: 22, rare: { kind: 'star', age: 3 } }) !== paint('idle', 0, { hour: 22 }),
    dragon: paint('idle', 0, { rare: { kind: 'dragon', age: 12 } }) !== paint('idle', 0, {}),
    shutters: paint('idle', 0, { hour: 23.5 }) !== paint('idle', 0, { hour: 22 }),
  }
  expect(Object.entries(changes).filter(([, changed]) => !changed).map(([field]) => field)).toEqual([])
})

test('a question to the person shows the waiting pose while it is open', async ($, on) => {
  let read: (() => Promise<string | undefined>) | undefined
  let seen: string | undefined
  const { ui } = await start($, on, async e => {
    if (e.tool === 'AskUserQuestion') seen = await read?.()
    return { result: 'ok' }
  })
  read = async () => (await ui.find({ text: /^ · / }))?.text
  await $.tool.call({ tool: 'AskUserQuestion', questions: [] })
  expect(seen).toBe(' · attend ton feu vert')
  expect(await read()).toBe(' · au calme') // and lets go once answered
})

test('a narrow band keeps the words and drops the sprite', async ($, on) => {
  mock.clock(on)
  engine(on)
  await $.session.start({ cwd: 'C:/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({
    plugin: 'diorama', surface: 'terminal', component: 'AbovePrompt',
    props: { ...BAND, bodyColumns: 36 },
  })
  expect(await ui.find({ type: 'Raster' })).toBeUndefined()
  expect(await ui.find({ text: SMITH })).toBeDefined()
})

test('the desktop gets the words alone', async ($, on) => {
  mock.clock(on)
  engine(on)
  await $.session.start({ cwd: 'C:/tmp', surface: 'desktop', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'diorama', surface: 'desktop', component: 'AbovePrompt', props: BAND })
  expect(await ui.find({ text: SMITH })).toBeDefined()
})

test('the decor option narrows the scene', { options: { decor: 'seul' } }, async ($, on) => {
  const { ui } = await start($, on)
  const raster = await ui.find({ type: 'Raster', key: 'diorama' })
  expect(raster?.props.columns).toBe(25)
  expect(raster?.props.rows).toBe(16)
})

test('a middling width drops the furnace first, then the anvil', async ($, on) => {
  mock.clock(on)
  engine(on)
  await $.session.start({ cwd: 'C:/tmp', surface: 'terminal', isInteractive: true })
  const widths: Array<number | undefined> = []
  for (const bodyColumns of [110, 80]) {
    const ui = await $.ui.mount({
      plugin: 'diorama', surface: 'terminal', component: 'AbovePrompt',
      props: { ...BAND, bodyColumns },
    })
    widths.push((await ui.find({ type: 'Raster', key: 'diorama' }))?.props.columns as number | undefined)
  }
  expect(widths).toEqual([58, 25])
})

test('a terminal too short for the forge keeps the words', async ($, on) => {
  mock.clock(on)
  engine(on)
  await $.session.start({ cwd: 'C:/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({
    plugin: 'diorama', surface: 'terminal', component: 'AbovePrompt',
    props: { ...BAND, maxRows: 13 },
  })
  expect(await ui.find({ type: 'Raster' })).toBeUndefined()
  expect(await ui.find({ text: SMITH })).toBeDefined()
})

test('the work names itself and colours the sparks', async ($, on) => {
  let read: (() => Promise<string | undefined>) | undefined
  let during: string | undefined
  const { ui } = await start($, on, async () => {
    during = await read?.()
    return { result: 'ok' }
  })
  read = async () => (await ui.find({ text: /^ · / }))?.text
  await $.prompt.submit(typed('go'))
  const seen: Record<string, string | undefined> = {}
  for (const [name, call] of [
    ['build', { tool: 'Bash', command: 'cmake --build build --config Release' }],
    ['git', { tool: 'Bash', command: 'git status' }],
    ['shell', { tool: 'Bash', command: 'ls -la' }],
    ['agent', { tool: 'Agent', prompt: 'p', description: 'd' }],
    ['forge', { tool: 'Edit', file_path: 'a.ts', old_string: 'a', new_string: 'b' }],
    ['test', { tool: 'Bash', command: './build/bin/Release/vehigraph_tests.exe --gtest_filter=X' }],
  ] as const) {
    await $.tool.call(call as never)
    seen[name] = during // the label while the tool runs
  }
  expect(seen).toEqual({
    build: ' · chauffe le fourneau',
    git: " · scelle l'ouvrage",
    shell: ' · manie ses outils',
    agent: ' · envoie un apprenti',
    forge: ' · à la forge',
    test: ' · éprouve la lame',
  })
  // the same blow, two kinds of work: the sparks differ
  const at = (f: 'forge' | 'test') => Array.from(frame('full', 'running', 4, { ...CALM, flavor: f })).join()
  expect(at('test')).not.toBe(at('forge'))
})

// The "!" means the person is needed: a permission dialog, not a command the
// auto-mode classifier approves on its own.
test('only a notification to the person raises the "!"', async ($, on) => {
  let read: (() => Promise<string | undefined>) | undefined
  const during: string[] = []
  const { ui } = await start($, on, async e => {
    if (e.tool === 'Bash' && e.command === 'ls') {
      during.push(String(await read?.()))
      await $.classic.Notification({ message: 'Claude is waiting for your input', notification_type: 'idle_prompt' })
      during.push(String(await read?.()))
      await $.classic.Notification({ message: 'Claude needs your permission to use Bash', notification_type: 'permission_prompt' })
      during.push(String(await read?.()))
    }
    return { result: 'ok' }
  })
  read = async () => (await ui.find({ text: /^ · / }))?.text
  await $.prompt.submit(typed('go'))
  await $.tool.call({ tool: 'Bash', command: 'ls' })
  expect(during).toEqual([' · manie ses outils', ' · manie ses outils', ' · attend ton feu vert'])
  expect(await ui.find({ text: ' · permission' })).toBeUndefined() // answered: back to work
  expect(await read()).toBe(' · réfléchit')
})

const day = (date: string, tests = 0, builds = 0) => ({ date, tests, builds, edits: 0, shells: 0, ticks: 20_000 })

test('the week is kept across sessions, seven days at most', { timeoutMs: 30_000 }, async ($, on) => {
  const old = ['2026-09-21', '2026-09-22', '2026-09-23', '2026-09-24', '2026-09-25', '2026-09-28', '2026-09-29', '2026-09-30']
  const { clock } = await start($, on, answerTools, { 'week:sess-1': old.map(d => day(d)) })
  await $.prompt.submit(typed('go'))
  for (let i = 0; i < 3; i++) await $.tool.call({ tool: 'Bash', command: 'ctest' })
  await clock.advance(31_000) // the half-minute flush
  const week = shelf.get('week:sess-1') as Array<{ date: string; tests: number }>
  expect(week.length).toBe(7)
  expect(week[0]?.date).toBe('2026-09-23') // eight old days and today: the two oldest fell off
  expect(week[6]?.date).toBe(new Date(Date.UTC(2026, 9, 2, 12, 0)).toISOString().slice(0, 10))
  expect(week[6]?.tests).toBe(3)
})

test('experience rises with the work, and a level is said aloud', async ($, on) => {
  const { ui } = await start($, on)
  await $.prompt.submit(typed('go'))
  expect((await ui.find({ text: /niv\. 1/ }))?.text).toMatch(/niv\. 1 · 0 xp/)
  for (let i = 0; i < 4; i++) await $.tool.call({ tool: 'Edit', file_path: 'a.ts', old_string: 'a', new_string: 'b' })
  expect((await ui.find({ text: /niv\. 2/ }))?.text).toMatch(/niv\. 2 · 8 xp/)
  expect((await ui.find({ text: /^« / }))?.text).toMatch(/niveau 2/i)
})

test("the project's name is on the forge's line", async ($, on) => {
  const { ui } = await start($, on)
  expect((await ui.find({ text: /vehigraph · claude-opus-5-5/ }))).toBeDefined()
})

test('a limit reached leaves the forge cold', async ($, on) => {
  const { ui, clock } = await start($, on)
  await clock.advance(3100)
  await $.session.measure({
    context: { percent: 40, tokens: 80_000, window: 200_000 },
    rateLimits: [{ kind: 'five_hour', percentUsed: 100, resetsAt: IN_TWO_HOURS }],
    changed: ['rateLimits'],
  })
  expect((await ui.find({ text: /^ · / }))?.text).toBe(' · à court de charbon')
})

test('three failures running bring out the duck', async ($, on) => {
  const { ui, clock } = await start($, on)
  await $.prompt.submit(typed('go'))
  for (let i = 0; i < 3; i++) {
    await $.tool.call({ tool: 'Bash', command: 'make all' })
    await clock.advance(2600)
  }
  await $.turn.complete(ended('aborted'))
  expect((await ui.find({ text: /^ · / }))?.text).toBe(' · parle au canard')
})

test('two hours in, tea', { timeoutMs: 120_000 }, async ($, on) => {
  const { ui, clock } = await start($, on)
  for (let i = 0; i < 5; i++) await clock.advance(24 * 60 * 1000 + 200) // in steps the mocked clock takes
  await $.prompt.submit(typed('go')) // wakes him from his nap
  await $.turn.complete(ended('aborted'))
  await clock.advance(1100)
  expect((await ui.find({ text: /^ · / }))?.text).toBe(' · prend le thé')
})

test('with the haiku voice, Haiku writes his line after a turn', { options: { repliques: 'haiku' } }, async ($, on) => {
  let asked = ''
  on('model.complete', (_$, e) => {
    asked = e.prompt
    return { value: { isAnswered: true, text: '« Joli travail, forgeron. »', usage: { input_tokens: 1, output_tokens: 1, cache_creation_input_tokens: 0, cache_read_input_tokens: 0 } } } as never
  })
  const { ui, clock } = await start($, on)
  await clock.advance(16_000) // past the greeting's quiet time
  await $.prompt.submit(typed('go'))
  await $.turn.complete({ ...ended('answer'), answer: 'Le module compile et les tests passent.' })
  await clock.advance(10)
  expect(asked).toContain('Le module compile')
  expect((await ui.find({ text: /^« / }))?.text).toBe('« Joli travail, forgeron. »')
})

test('with no voice, he keeps quiet', { options: { repliques: 'aucune' } }, async ($, on) => {
  const { ui } = await start($, on)
  await $.prompt.submit(typed('go'))
  await $.turn.complete(ended('answer'))
  expect(await ui.find({ text: /^« / })).toBeUndefined()
})


test('a reload keeps the apprentices still at work', async ($, on) => {
  mock.clock(on)
  engine(on)
  listed = [{ id: 'ag-live', description: 'tranche vivante', type: 'general-purpose', status: 'running' },
    { id: 'ag-done', description: 'tranche finie', type: 'general-purpose', status: 'completed' }]
  await $.session.start({ cwd: 'C:/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'diorama', surface: 'terminal', component: 'AbovePrompt', props: BAND })
  expect((await ui.find({ text: /^apprentis/ }))?.text).toBe('apprentis 1')
  expect(await ui.find({ text: /general-purpose · 0 min — tranche vivante \(fond\)/ })).toBeDefined()
})

/** The cells of one frame: [glyph, fg, bg] per cell, `cols` across. */
function cellsOf(b64: string): Uint32Array {
  const bin = Uint8Array.from(atob(b64), ch => ch.charCodeAt(0))
  return new Uint32Array(bin.buffer)
}

/** Columns in a band of rows where a colour shows, in a frame `cols` wide (drawn without light: `lumiere: 'non'`). */
function columnsWith(b64: string, cols: number, colour: number, rows: readonly number[], below = cols): number[] {
  const w = cellsOf(b64)
  const found: number[] = []
  for (const r of rows) {
    for (let x = 0; x < Math.min(cols, below); x++) {
      const i = (r * cols + x) * 3
      if (w[i + 1] === colour || w[i + 2] === colour) found.push(x)
    }
  }
  return found
}

const CREST = 0x1f4f9e // his blue crest: only Scorpheus wears it left of the banner
const VIOLET = 0xb07cff // a general-purpose cub's scarf

test('idle, Scorpheus roams the workshop', { options: { lumiere: 'non' }, timeoutMs: 60_000 }, async ($, on) => {
  const { clock } = await start($, on)
  await clock.advance(3100) // past the greeting
  blits.length = 0
  for (let i = 0; i < 9; i++) await clock.advance(10_000)
  const where = new Set(blits.map(b => Math.min(...columnsWith(b, 72, CREST, [3, 4, 5, 6, 7], 46))))
  expect(where.size).toBeGreaterThanOrEqual(3) // he went somewhere, and passed through on his way
})

test('a subagent comes in as a cub, and walks out when done', { options: { lumiere: 'non' }, timeoutMs: 30_000 }, async ($, on) => {
  const { clock } = await start($, on)
  const cubShows = () => columnsWith(blits[blits.length - 1] ?? '', 72, VIOLET, [12, 13, 14, 15]).length > 0
  await $.agent.spawn(spawned('c1', 'general-purpose', 'tranche', true))
  await clock.advance(1500)
  expect(cubShows()).toBe(true)
  await $.turn.complete(ended('answer', 'ag-c1'))
  await clock.advance(1500)
  expect(cubShows()).toBe(true) // on its way out, scroll held high
  await clock.advance(7000)
  expect(cubShows()).toBe(false) // gone
})

test('each cub gets a name, from the litter or from Haiku', async ($, on) => {
  on('model.complete', (_$, e) => ({ value: { isAnswered: true, text: e.prompt.includes('inventaire') ? 'Pistou' : '', usage: { input_tokens: 1, output_tokens: 1, cache_creation_input_tokens: 0, cache_read_input_tokens: 0 } } }) as never)
  const { ui } = await start($, on)
  await $.agent.spawn(spawned('n1', 'Explore', 'inventaire des sources', true))
  await $.agent.spawn(spawned('n2', 'Plan', 'découpage', true))
  expect(await ui.find({ text: /^Pistou$/ })).toBeDefined() // Haiku's
  const local = await ui.find({ text: /^(Akéla|Fenrir|Loupiot|Gris|Nuage|Flocon|Brume|Tison|Braise|Silex|Pépite|Rune|Grisou|Filou|Croc|Plume)$/ })
  expect(local).toBeDefined() // Haiku said nothing: one from the litter
  expect(await ui.find({ text: /arrivée de Pistou \(Explore\)/ })).toBeDefined() // and the log says so
})

test("Haiku dresses the session from the conversation's start, and the others see it", async ($, on) => {
  let asked = ''
  on('model.complete', (_$, e) => {
    if (e.prompt.includes('Début de la conversation')) asked = e.prompt
    return { value: { isAnswered: true, text: '{"cape": "braise", "accessoire": "tablier"}', usage: { input_tokens: 1, output_tokens: 1, cache_creation_input_tokens: 0, cache_read_input_tokens: 0 } } } as never
  })
  const { ui } = await start($, on, answerTools, {
    presence: { 'sess-other': { project: 'clawd-hq', cape: 'nuit', accessory: 'lunettes', seen: Date.UTC(2026, 9, 2, 12, 0) } },
  })
  await $.prompt.submit(typed('On refond le pilote automatique du tracé'))
  expect(asked).toContain('On refond le pilote')
  expect(await ui.find({ text: /tenue du jour : cape braise, tablier/ })).toBeDefined()
  const presence = shelf.get('presence') as Record<string, { cape: string; accessory: string }>
  expect(presence['sess-1']).toMatchObject({ cape: 'braise', accessory: 'tablier' })
  expect(presence['sess-other']).toBeDefined() // the neighbour is kept
})

test('a resumed session keeps its look', { options: { lumiere: 'non' } }, async ($, on) => {
  const { ui } = await start($, on, answerTools, {
    presence: { 'sess-1': { project: 'vehigraph', cape: 'prune', accessory: 'bandeau', seen: Date.UTC(2026, 9, 2, 11, 0) } },
  })
  await $.prompt.submit(typed('on reprend'))
  expect(await ui.find({ text: /tenue du jour/ })).toBeUndefined() // nothing chosen again
  expect((shelf.get('presence') as Record<string, { cape: string }>)['sess-1']?.cape).toBe('prune')
})


// Two sessions reloaded in the same instant (a save of the mod reaches them all)
// must not play the same frames: each has its own phase.
const idleFrames: Record<string, string[]> = {}
for (const id of ['sess-a', 'sess-b']) {
  test(`session ${id} idles to its own beat`, async ($, on) => {
    sessionId = id
    const clock = mock.clock(on, { now: Date.UTC(2026, 9, 2, 12, 0) })
    engine(on)
    sessionId = id
    on('tool.call', () => ({ result: 'ok' }))
    await $.session.start({ cwd: 'C:/tmp', surface: 'terminal', isInteractive: true })
    await $.ui.mount({ plugin: 'diorama', surface: 'terminal', component: 'AbovePrompt', props: BAND })
    await clock.advance(3100)
    blits.length = 0
    for (let i = 0; i < 6; i++) await clock.advance(10_000)
    // where he stands and how his head sits, frame by frame: the crest's columns and
    // rows, whatever colour his cape (the cape alone already differs between sessions)
    idleFrames[id] = blits.map(b => [3, 4, 5].map(r => columnsWith(b, 72, CREST, [r], 46).join(',')).join('|')) // rows no accessory covers
    expect(blits.length).toBeGreaterThan(300)
  })
}
test('two sessions started together are not in step', () => {
  const a = idleFrames['sess-a'] ?? [], b = idleFrames['sess-b'] ?? []
  const same = a.filter((pose, i) => pose === b[i]).length
  expect(same / a.length).toBeLessThan(0.8)
})

const START = Date.UTC(2026, 9, 2, 12, 0)
const crestColumns = (b: string) => columnsWith(b, 72, CREST, [3, 4, 5])

test('idle, he goes to visit another forge, and work calls him home', { options: { lumiere: 'non' }, timeoutMs: 120_000 }, async ($, on) => {
  const neighbour = { project: 'Dev', cape: 'nuit', accessory: 'lunettes', seen: START }
  const { ui, clock } = await start($, on, answerTools, { presence: { 'sess-dev': neighbour } })
  const label = async () => (await ui.find({ text: /^ · / }))?.text
  // the neighbour keeps beating; he waits for his chance, before his nap
  for (let s = 0; s < 57 && !shelf.has('visit:sess-1'); s++) {
    await clock.advance(10_000)
    const presence = shelf.get('presence') as Record<string, unknown>
    shelf.set('presence', { ...presence, 'sess-dev': { ...neighbour, seen: clock.now() } }) // what the plugin wrote is frozen
  }
  expect(shelf.get('visit:sess-1')).toMatchObject({ host: 'sess-dev', project: 'vehigraph' })
  expect(await ui.find({ text: /part rendre visite à la forge Dev/ })).toBeDefined()
  expect(await label()).toBe(' · en visite')
  blits.length = 0
  await clock.advance(1000)
  expect(blits.every(b => crestColumns(b).filter(x => x < 46).length === 0)).toBe(true) // his forge stands empty

  await $.prompt.submit(typed('reviens'))
  expect(shelf.get('visit:sess-1')).toMatchObject({ phase: 'leaving' }) // he says he is leaving
  expect(await label()).toBe(' · réfléchit')
  await clock.advance(3000)
  expect(crestColumns(blits[blits.length - 1] ?? '').filter(x => x < 46).length).toBe(0) // not home before the host lets him go
  // the host walks him out and answers
  shelf.set('visit:sess-1', { ...(shelf.get('visit:sess-1') as object), phase: 'gone' })
  await clock.advance(8000)
  expect(shelf.has('visit:sess-1')).toBe(false)
  expect(crestColumns(blits[blits.length - 1] ?? '').length).toBeGreaterThan(0) // home again
  expect(await ui.find({ text: /de retour de chez Dev/ })).toBeDefined()
})

test('a guest from another forge comes into the yard, is greeted, and is walked out', { options: { lumiere: 'non' }, timeoutMs: 60_000 }, async ($, on) => {
  const { ui, clock } = await start($, on, answerTools, {
    'visit:sess-dev': { host: 'sess-1', until: START + 60_000, project: 'Dev', cape: 'nuit', accessory: 'lunettes', phase: 'there' },
  })
  await clock.advance(9000)
  expect(await ui.find({ text: /visite de la forge Dev/ })).toBeDefined()
  expect(crestColumns(blits[blits.length - 1] ?? '').filter(x => x >= 54).length).toBeGreaterThan(0) // in the yard
  // his session says he leaves: he walks out, and only then is he told gone
  shelf.set('visit:sess-dev', { ...(shelf.get('visit:sess-dev') as object), phase: 'leaving' })
  await clock.advance(1500)
  expect(shelf.get('visit:sess-dev')).toMatchObject({ phase: 'leaving' }) // still walking out
  await clock.advance(6000)
  expect(await ui.find({ text: /la forge Dev repart/ })).toBeDefined()
  expect(crestColumns(blits[blits.length - 1] ?? '').filter(x => x >= 54).length).toBe(0) // gone
  expect(shelf.get('visit:sess-dev')).toMatchObject({ phase: 'gone' }) // and his session is told
})

test('called home, he comes back even if the host never answers', { timeoutMs: 120_000 }, async ($, on) => {
  const neighbour = { project: 'Dev', cape: 'nuit', accessory: 'lunettes', seen: START }
  const { ui, clock } = await start($, on, answerTools, { presence: { 'sess-dev': neighbour } })
  for (let s = 0; s < 57 && !shelf.has('visit:sess-1'); s++) {
    await clock.advance(10_000)
    shelf.set('presence', { ...(shelf.get('presence') as object), 'sess-dev': { ...neighbour, seen: clock.now() } })
  }
  expect(shelf.get('visit:sess-1')).toMatchObject({ phase: 'there' })
  await $.prompt.submit(typed('reviens'))
  await clock.advance(5000)
  expect(shelf.get('visit:sess-1')).toMatchObject({ phase: 'leaving' }) // still waiting for the host
  await clock.advance(12_000) // the host never answers: home after the wait
  expect(shelf.has('visit:sess-1')).toBe(false)
  expect(await ui.find({ text: /de retour de chez Dev/ })).toBeDefined()
})

const haikuSays = (answer: (prompt: string, system: string) => string) => (_$: unknown, e: { prompt: string; system?: string }) =>
  ({ value: { isAnswered: true, text: answer(e.prompt, e.system ?? ''), usage: { input_tokens: 1, output_tokens: 1, cache_creation_input_tokens: 0, cache_read_input_tokens: 0 } } }) as never
const localAt = (h: number, m = 0) => new Date(2026, 9, 2, h, m).getTime()
const dayOf = (ms: number) => {
  const d = new Date(ms)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

/** A session started at a given local time, the band mounted. */
async function startAt($: Engine, on: On, at: number, stored: Readonly<Record<string, unknown>> = {}) {
  const clock = mock.clock(on, { now: at })
  engine(on, stored)
  on('tool.call', () => ({ result: 'ok' }))
  await $.session.start({ cwd: 'C:/tmp', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ plugin: 'diorama', surface: 'terminal', component: 'AbovePrompt', props: BAND })
  return { ui, clock }
}

test('no two forges at work wear the same look', async ($, on) => {
  let asked = ''
  on('model.complete', haikuSays((_prompt, system) => {
    if (system.includes('tenue')) asked = system
    return '{"cape": "braise", "accessoire": "tablier", "caractere": "jovial"}'
  }))
  const { clock } = await start($, on, answerTools, { presence: { 'sess-dev': { project: 'Dev', cape: 'braise', accessory: 'tablier', seen: START } } })
  await $.prompt.submit(typed('Smoke test des 12 heures'))
  await clock.advance(1000) // the look is chosen in the background
  expect(asked).toContain('braise+tablier') // Haiku is told what is already worn
  const mine = (shelf.get('presence') as Record<string, { cape: string; accessory: string; personality: string }>)['sess-1']
  expect(`${mine?.cape}+${mine?.accessory}`).not.toBe('braise+tablier')
  expect(mine?.personality).toBe('jovial')
})

test('of two twins already dressed alike, the later one changes', async ($, on) => {
  const { ui } = await start($, on, answerTools, { presence: {
    'sess-0': { project: 'Dev', cape: 'braise', accessory: 'tablier', seen: START },
    'sess-1': { project: 'vehigraph', cape: 'braise', accessory: 'tablier', seen: START - 60_000 },
  } })
  const mine = (shelf.get('presence') as Record<string, { cape: string; accessory: string }>)['sess-1']
  expect(`${mine?.cape}+${mine?.accessory}`).not.toBe('braise+tablier')
  expect(await ui.find({ text: /change de tenue : la forge Dev portait la même/ })).toBeDefined()
})

test('ravens carry the news between forges', async ($, on) => {
  const { ui, clock } = await start($, on, answerTools, { 'news:sess-dev': { kind: 'waiting', project: 'Dev', at: START + 2000 } })
  await clock.advance(6000)
  expect(await ui.find({ text: /corbeau : la forge Dev attend ton feu vert/ })).toBeDefined()
  // this forge sends its own when the person is needed, and takes it back when they come
  await $.classic.Notification({ message: 'Claude needs your permission', notification_type: 'permission_prompt' })
  expect(shelf.get('news:sess-1')).toMatchObject({ kind: 'waiting', project: 'vehigraph' })
  await $.prompt.submit(typed('vas-y'))
  expect(shelf.has('news:sess-1')).toBe(false)
})

test("a raven names the conversation, not only its project", async ($, on) => {
  const { ui, clock } = await start($, on, answerTools, {
    'news:sess-pit': { kind: 'waiting', project: 'Dev', title: 'Gestion carburant pit AI', at: START + 2000 },
  })
  await clock.advance(6000)
  expect(await ui.find({ text: /corbeau : « Gestion carburant pit AI » attend ton feu vert/ })).toBeDefined()
  // this forge's own raven carries the title Claude Code gave the conversation
  await $.classic.UserPromptSubmit({ prompt: 'go', session_title: 'Refonte des jauges' })
  await $.classic.Notification({ message: 'Claude needs your permission', notification_type: 'permission_prompt' })
  expect(shelf.get('news:sess-1')).toMatchObject({ kind: 'waiting', title: 'Refonte des jauges' })
  expect(shelf.get('title:sess-1')).toBe('Refonte des jauges') // kept for a reload
})

test('each forge has a smith of its own name, and the words name the others by theirs', { timeoutMs: 60_000 }, async ($, on) => {
  // every name but one already worn by older forges: this one takes the one left
  const presence: Record<string, unknown> = {}
  SMITHS.slice(1).forEach((name, i) => {
    presence[`sess-0${String(i).padStart(2, '0')}`] = { project: 'Dev', cape: 'nuit', accessory: 'lunettes', seen: START, name }
  })
  const { ui, clock } = await start($, on, answerTools, {
    presence,
    'visit:sess-pit': { host: 'sess-1', until: START + 60_000, project: 'Dev', cape: 'braise', accessory: 'tablier', phase: 'there', name: 'Brasko', title: 'Gestion carburant pit AI' },
  })
  await clock.advance(9000)
  expect((await ui.find({ text: SMITH }))?.text).toBe(SMITHS[0])
  expect(await ui.find({ text: 'Scorpheus' })).toBeUndefined() // the person's name, not his
  expect(await ui.find({ text: /visite de Brasko \(« Gestion carburant pit AI »\)/ })).toBeDefined()
})

test('a guest leaves a gift', { timeoutMs: 60_000 }, async ($, on) => {
  const { ui, clock } = await start($, on, answerTools, {
    'visit:sess-dev': { host: 'sess-1', until: START + 20_000, project: 'Dev', cape: 'nuit', accessory: 'lunettes', phase: 'there' },
  })
  await clock.advance(9000)
  shelf.set('visit:sess-dev', { ...(shelf.get('visit:sess-dev') as object), phase: 'leaving' })
  await clock.advance(9000)
  expect(await ui.find({ text: /la forge Dev repart, et laisse (une pièce|un biscuit|une pépite)/ })).toBeDefined()
})

test('three forges idle at once: a banquet, held by the first', { timeoutMs: 120_000 }, async ($, on) => {
  const idleLong = { idleSince: START - 10 * 60_000 }
  const others = {
    'sess-2': { project: 'Dev', cape: 'nuit', accessory: 'lunettes', seen: START, ...idleLong },
    'sess-3': { project: 'captures', cape: 'ocre', accessory: 'aucun', seen: START, ...idleLong },
  }
  const { ui, clock } = await start($, on, answerTools, { presence: others })
  for (let i = 0; i < 18 && !shelf.has('banquet'); i++) {
    await clock.advance(10_000)
    const now = clock.now()
    shelf.set('presence', { ...(shelf.get('presence') as object), 'sess-2': { ...others['sess-2'], seen: now }, 'sess-3': { ...others['sess-3'], seen: now } })
  }
  expect(shelf.get('banquet')).toMatchObject({ host: 'sess-1' })
  expect(await ui.find({ text: /banquet à la forge/ })).toBeDefined()
})

test('invited to a banquet, he goes', async ($, on) => {
  const { ui, clock } = await start($, on, answerTools, {
    presence: { 'sess-0': { project: 'Dev', cape: 'nuit', accessory: 'lunettes', seen: START } },
    banquet: { date: dayOf(START), host: 'sess-0', until: START + 150_000 },
  })
  await clock.advance(9000)
  expect(await ui.find({ text: /part au banquet de la forge Dev/ })).toBeDefined()
})

test('the cat leaves the roof when the furnace roars', { options: { lumiere: 'non' } }, async ($, on) => {
  let during: string | undefined
  let tick: (() => Promise<void>) | undefined
  const { clock } = await start($, on, async e => {
    if (e.tool === 'Bash' && /cmake/.test(e.command)) {
      await tick?.()
      during = blits[blits.length - 1]
    }
    return { result: 'ok' }
  })
  tick = () => clock.advance(3000)
  await clock.advance(16_000) // the only forge takes the cat in; it walks to the sill
  const GOLD = 0xc99a3c
  const onRoof = (b: string) => columnsWith(b, 72, GOLD, [2, 3, 4]).some(x => x >= 15 && x <= 23)
  expect(onRoof(blits[blits.length - 1] ?? '')).toBe(true)
  await $.prompt.submit(typed('compile'))
  await $.tool.call({ tool: 'Bash', command: 'cmake --build build' })
  expect(during).toBeDefined()
  expect(onRoof(during ?? '')).toBe(false)
  // nor on the floor at his heels: fled, not following him to the bellows
  const catGrey = (b: string) => columnsWith(b, 72, GOLD, [13, 14, 15]).some(x => x < 21)
  expect(catGrey(during ?? '')).toBe(false)
})

for (const [habit, at, line] of [
  ['coffee in the morning', localAt(8, 0), 'un café pour commencer'],
  ['a bite at half past twelve', localAt(12, 40), 'casse-croûte de midi'],
  ['the lantern at dusk', localAt(19, 10), 'allume la lanterne'],
] as const) {
  test(`the day's habits: ${habit}`, { timeoutMs: 30_000 }, async ($, on) => {
    const { ui, clock } = await startAt($, on, at)
    await clock.advance(3100) // past the greeting
    for (let i = 0; i < 6 && !(await ui.find({ text: new RegExp(line) })); i++) await clock.advance(5000)
    expect(await ui.find({ text: new RegExp(line) })).toBeDefined()
  })
}

test('no bite before half past twelve', { timeoutMs: 30_000 }, async ($, on) => {
  const { ui, clock } = await startAt($, on, localAt(12, 10))
  for (let i = 0; i < 7; i++) await clock.advance(5000)
  expect(await ui.find({ text: /casse-croûte/ })).toBeUndefined()
})

test('Haiku names the task, says where it stands, then what is done and what is left', { timeoutMs: 120_000 }, async ($, on) => {
  const asked: Record<string, string[]> = { goal: [], progress: [], done: [] }
  on('model.complete', haikuSays((prompt, system) => {
    if (system.includes('titre de la tâche')) { asked.goal?.push(prompt); return 'Refondre le pilote du tracé' }
    if (system.includes('où il en est')) { asked.progress?.push(prompt); return 'Réécrit le régulateur dans pilot.ts' }
    if (system.includes("fin d'un tour")) { asked.done?.push(prompt); return 'Pilote refondu ; reste à valider sur piste' }
    return ''
  }))
  const { ui, clock } = await start($, on)
  await $.prompt.submit(typed('On refond le pilote automatique du tracé'))
  expect(await ui.find({ text: /^Refondre le pilote du tracé$/ })).toBeDefined()

  // three actions are not enough before half a minute: no call yet
  for (let i = 0; i < 3; i++) await $.tool.call({ tool: 'Edit', file_path: `pilot${i}.ts`, old_string: 'a', new_string: 'b' })
  expect(asked.progress).toHaveLength(0)
  await clock.advance(31_000)
  await $.tool.call({ tool: 'Bash', command: 'bun test', description: 'Lance les tests du pilote' })
  expect(asked.progress).toHaveLength(1)
  expect(asked.progress?.[0]).toContain('Bash : Lance les tests du pilote') // a command by its description
  expect(await ui.find({ text: /^Réécrit le régulateur dans pilot\.ts$/ })).toBeDefined()
  await $.tool.call({ tool: 'Edit', file_path: 'pilot.ts', old_string: 'a', new_string: 'b' })
  expect(asked.progress).toHaveLength(1) // two minutes at most between two

  await $.turn.complete({ ...ended('answer'), answer: 'Le pilote est refondu. Il reste à le valider sur piste.' })
  expect(await ui.find({ text: /^Pilote refondu ; reste à valider sur piste$/ })).toBeDefined()

  // "vas-y" carries on the same task: Haiku is told which
  await $.prompt.submit(typed('vas-y'))
  expect(asked.goal?.[1]).toContain('Tâche en cours : Refondre le pilote du tracé')
  expect(await ui.find({ text: /^Pilote refondu/ })).toBeUndefined() // the last turn's line is gone
})

test('the simple status needs no model: the request, then the last action', { options: { statut: 'simple' } }, async ($, on) => {
  let calls = 0
  on('model.complete', haikuSays((_prompt, system) => {
    if (/tâche|tour de travail/.test(system)) calls += 1
    return ''
  }))
  const { ui } = await start($, on)
  await $.prompt.submit(typed('Corriger le calcul des jauges'))
  expect(await ui.find({ text: /^Corriger le calcul des jauges$/ })).toBeDefined()
  await $.tool.call({ tool: 'Read', file_path: 'C:/src/gauges.ts' })
  expect(await ui.find({ text: /^Read gauges\.ts$/ })).toBeDefined()
  expect(calls).toBe(0)
})

test('with no status, the words keep to the forge', { options: { statut: 'aucun' } }, async ($, on) => {
  const { ui } = await start($, on)
  await $.prompt.submit(typed('Corriger le calcul des jauges'))
  expect(await ui.find({ text: /^Corriger le calcul des jauges$/ })).toBeUndefined()
})

test('the status takes two lines for the task and three for where it stands', async ($, on) => {
  const long = 'Analyser les passages des arrêts au stand entre une et quatre secondes et corriger les bugs du carburant'
  on('model.complete', haikuSays((_prompt, system) => system.includes('titre de la tâche') ? long
    : system.includes("fin d'un tour") ? `${long}. ${long}. ${long}. ${long}.` : ''))
  const { ui } = await start($, on)
  await $.prompt.submit(typed('Les arrêts au stand'))
  expect(await ui.find({ text: /^Analyser les passages/ })).toBeDefined()
  expect(await ui.find({ text: /^(?!Analyser).*carburant$/ })).toBeDefined() // the title's second line
  await $.turn.complete({ ...ended('answer'), answer: 'Fait.' })
  const lines = await ui.findAll({ text: /arrêts|carburant|Analyser|secondes|stand|bugs/ })
  expect(lines.some(l => l.text.endsWith('…'))).toBe(true) // a long summary is cut on its third line
})

test('a cat standing still does not tread the air', async () => {
  const at = (age: number, step?: number) =>
    Array.from(frame('full', 'idle', 0, { ...CALM, cat: { mode: 'walk', x: 56, age, dir: -1, step } })).join()
  expect(at(6)).toBe(at(5)) // still: one pose, whatever the time
  expect(at(5, 1)).not.toBe(at(5, 0)) // walking: its legs move with its steps
})

test('the forge is lit: the day through the window, a darker night, the palette untouched without light', () => {
  // the anvil's face (row 26, steel light), under the window, clear of the hearth
  const steel = (hour: number, lit = true) => frame('full', 'idle', 0, { ...CALM, hour }, lit)[(13 * 72 + 22) * 3 + 1] ?? 0
  const bright = (c: number) => (c >> 16) + ((c >> 8) & 255) + (c & 255)
  expect(steel(12, false)).toBe(0x8a90a0) // without light: the palette's own steel
  expect(steel(12)).not.toBe(0x8a90a0) // lit, it is the light's
  expect(bright(steel(23))).toBeLessThan(bright(steel(12))) // and darker by night
})

test('the light shows the wall behind the forge, never the yard', () => {
  const night = { ...CALM, hour: 22, lanternLit: true }
  const cells = frame('full', 'idle', 0, night)
  const clear = (x: number, r: number) => cells[(r * 72 + x) * 3] === 0x20 && cells[(r * 72 + x) * 3 + 2] === 0x01000000
  expect(clear(8, 1)).toBe(false) // beside the lantern, its light on the wall
  expect([...Array(72 - 54).keys()].every(i => clear(54 + i, 0))).toBe(true) // the sky over the yard stays the terminal's
  const flat = frame('full', 'idle', 0, night, false)
  expect(flat[(1 * 72 + 8) * 3]).toBe(0x20) // without light, no wall
})

test('the rack\'s rail keeps its colour while the fire wavers and the hammer flashes', () => {
  // the rail is under the rack's first cell row; above the hearth's hood, out of the blow's reach
  for (const x of [20, 24, 28]) {
    const tints = new Set<number>()
    for (let t = 0; t < 64; t++) {
      const cells = frame('full', 'running', t, { ...CALM, hour: 15, effort: 2, flavor: 'forge', rack: ['sword'], clock: 100 + t })
      const i = (5 * 72 + x) * 3 // the rail is the cell's lower pixel
      tints.add((cells[i] === 0x2584 ? cells[i + 1] : cells[i + 2]) ?? 0)
    }
    expect(tints.size).toBeLessThanOrEqual(2)
  }
})

test('his boots keep step with the ground as he walks', async () => {
  // the right boot, planted, under the same columns while he walks three pixels right over it
  const boots = (x: number) => {
    const cells = frame('full', 'idle', x, { ...CALM, walking: true, facing: 1, x })
    return Array.from(cells.slice((15 * 72 + 40) * 3, (15 * 72 + 45) * 3)).join()
  }
  expect(boots(1)).toBe(boots(0))
  expect(boots(2)).toBe(boots(0))
  expect(boots(3)).not.toBe(boots(0)) // then it lifts
})

test('he dreams, once a conversation, of what it is about', { timeoutMs: 120_000 }, async ($, on) => {
  let asked = 0
  on('model.complete', haikuSays((_prompt, system) => {
    if (system.includes('rêve')) asked += 1
    return 'voiture'
  }))
  const { clock } = await start($, on)
  for (let i = 0; i < 3; i++) await clock.advance(5 * 60 * 1000) // a nap after ten quiet minutes
  await $.prompt.submit(typed('debout'))
  await $.turn.complete(ended('aborted'))
  for (let i = 0; i < 3; i++) await clock.advance(5 * 60 * 1000) // a second nap
  expect(asked).toBe(1)
})

test('a long turn ending with green tests forges the masterpiece', { options: { lumiere: 'non' }, timeoutMs: 120_000 }, async ($, on) => {
  const { ui, clock } = await start($, on)
  await $.prompt.submit(typed('la grande refonte'))
  for (let i = 0; i < 4; i++) await clock.advance(4 * 60 * 1000) // sixteen minutes of work
  await $.tool.call({ tool: 'Bash', command: 'ctest --output-on-failure' })
  await $.turn.complete(ended('answer'))
  expect(await ui.find({ text: /chef-d'œuvre forgé/ })).toBeDefined()
  expect(shelf.has('master:sess-1')).toBe(true)
})

const GOLD_CAT = 0xc99a3c
const catShows = (b: string) => columnsWith(b, 72, GOLD_CAT, [2, 3, 4, 13, 14, 15]).length > 0

test('the machine has one cat: a forge without it never shows it', { options: { lumiere: 'non' }, timeoutMs: 60_000 }, async ($, on) => {
  const holder = { project: 'Dev', cape: 'nuit', accessory: 'lunettes', seen: START }
  const { clock } = await start($, on, answerTools, { presence: { 'sess-0': holder }, cat: { host: 'sess-0', since: START } })
  for (let i = 0; i < 3; i++) {
    await clock.advance(6000)
    shelf.set('presence', { ...(shelf.get('presence') as object), 'sess-0': { ...holder, seen: clock.now() } })
  }
  expect(blits.some(catShows)).toBe(false)
  expect(shelf.get('cat')).toMatchObject({ host: 'sess-0' })
})

test("nobody's cat, or its forge closed: the first forge takes it in", { options: { lumiere: 'non' }, timeoutMs: 60_000 }, async ($, on) => {
  const { ui, clock } = await start($, on, answerTools, { cat: { host: 'sess-closed', since: START - 3600_000 } })
  await clock.advance(16_000)
  expect(shelf.get('cat')).toMatchObject({ host: 'sess-1' })
  expect(await ui.find({ text: /un chat-panthère entre dans la forge/ })).toBeDefined()
  expect(catShows(blits[blits.length - 1] ?? '')).toBe(true)
})

test('after a while, the cat goes round to the next forge', { options: { lumiere: 'non' }, timeoutMs: 120_000 }, async ($, on) => {
  const next = { project: 'Dev', cape: 'nuit', accessory: 'lunettes', seen: START }
  const { ui, clock } = await start($, on, answerTools, {
    presence: { 'sess-2': next },
    cat: { host: 'sess-1', since: START - 20 * 60_000, from: 'captures' },
  })
  for (let i = 0; i < 12 && (shelf.get('cat') as { host: string }).host === 'sess-1'; i++) {
    await clock.advance(5000)
    shelf.set('presence', { ...(shelf.get('presence') as object), 'sess-2': { ...next, seen: clock.now() } })
  }
  expect(await ui.find({ text: /le chat-panthère arrive de la forge captures/ })).toBeDefined()
  expect(shelf.get('cat')).toMatchObject({ host: 'sess-2', from: 'vehigraph' })
  expect(await ui.find({ text: /le chat-panthère part chez la forge Dev/ })).toBeDefined()
  expect(catShows(blits[blits.length - 1] ?? '')).toBe(false) // gone from here
})

// --- themes of one's own ---------------------------------------------------------

const POSES = ['idle', 'thinking', 'running', 'review', 'waiting', 'failed', 'jumping', 'waving', 'quench', 'ship', 'sweep', 'sleep', 'waking'] as const

/** A small theme, 40 x 16: a character 6 x 8 whose every pose is a pixel of its own colour. */
function miniTheme(width = 40): ThemeSpec {
  const body = ['.kkkk.', 'kooook', 'kOOOOk', 'kOOOOk', '.kook.', '.kOOk.', '.k..k.', 'kk..kk']
  const marks = 'abcdefghijlmn'
  const poses = Object.fromEntries(POSES.map((p, i) => [p, [[`${marks[i]}.....`, ...body.slice(1)]]]))
  return {
    diorama: 1,
    name: 'Mini',
    width,
    height: 16,
    character: { home: [16, 8], recolor: ['o', 'O'], poses: { ...poses, walk: [body, body] } },
    sky: { at: [1, 1], size: [8, 5] },
    helpers: { walk: [['CCC', 'C.C']], area: [28, 39, 15] },
    words: { atWork: { forge: 'touille' }, notes: { level: 'niveau {n}, abracadabra' } },
  } as ThemeSpec
}

test('a theme file is checked, and what is wrong is said where it is', () => {
  expect(checkTheme(miniTheme())).toEqual([])
  const bad = miniTheme()
  ;(bad.character.poses as Record<string, unknown>).idle = [['.kk!k.']]
  expect(checkTheme(bad).join('\n')).toContain('character.poses.idle[0], ligne 1 : la lettre « ! » n\'est pas dans la palette')
  const none = miniTheme()
  delete (none.character.poses as Record<string, unknown>).idle
  expect(checkTheme(none).join('\n')).toContain('character.poses.idle manque')
  // output contract: a character drawn out of the frame is refused, not shown blank
  const away = miniTheme()
  away.character = { ...away.character, home: [200, 8] }
  expect(checkTheme(away).length).toBeGreaterThan(0)
})

test('every pose a theme draws is reached by its moment', () => {
  const theme = dataTheme(miniTheme(), 'mini')
  const at = (activity: (typeof POSES)[number]) => Array.from(theme.frame('full', activity, 0, { ...CALM, hour: 12 })).join()
  const shown = new Set(POSES.map(at))
  expect(shown.size).toBe(POSES.length)
  // a theme without a pose falls back: sleep without its own is the idle one
  const lean = miniTheme()
  delete (lean.character.poses as Record<string, unknown>).sleep
  const t2 = dataTheme(lean, 'lean')
  expect(Array.from(t2.frame('full', 'sleep', 0, CALM)).length).toBe(40 * 8 * 3)
})

/** The theme file on a fake disk: its text and when it last changed. */
const disk = { text: '', mtime: 1, asked: [] as string[] }
function fakeDisk(on: On, text: string) {
  disk.text = text
  disk.mtime = 1
  on('fs.stat', () => ({ value: { kind: 'file', size: disk.text.length, mtimeMs: disk.mtime, isLink: false } }))
  disk.asked = []
  on('fs.read', (_$, e) => {
    disk.asked.push(e.path.replace(/\\/g, '/'))
    return { value: disk.text }
  })
}

test('the theme option draws a theme of one\'s own, with its words', { options: { theme: 'C:/themes/mini.json' } }, async ($, on) => {
  fakeDisk(on, JSON.stringify(miniTheme()))
  const { ui } = await start($, on)
  const raster = await ui.find({ type: 'Raster', key: 'diorama' })
  expect(raster?.props.columns).toBe(40)
  expect(raster?.props.rows).toBe(8)
  await $.prompt.submit(typed('go'))
  await $.tool.call({ tool: 'Edit', file_path: 'a.ts', old_string: 'a', new_string: 'b' })
  expect(await ui.find({ text: ' · touille' })).toBeDefined()
  // an Edit is the forge's "à la forge"; here it is the theme's word, and nothing of the forge's shows
  expect(await ui.find({ text: / · à la forge/ })).toBeUndefined()
})

test('a broken theme leaves the forge, and the log says why', { options: { theme: 'C:/themes/broken.json' } }, async ($, on) => {
  fakeDisk(on, '{ "diorama": 1, "name": ')
  const { ui } = await start($, on)
  expect((await ui.find({ type: 'Raster', key: 'diorama' }))?.props.columns).toBe(72)
  expect(await ui.find({ text: /thème « C:\/themes\/broken\.json » refusé : JSON illisible/ })).toBeDefined()
})

test('a theme file changed on disk is drawn again at once', { options: { theme: 'C:/themes/mini.json' } }, async ($, on) => {
  fakeDisk(on, JSON.stringify(miniTheme(40)))
  const { ui, clock } = await start($, on)
  expect((await ui.find({ type: 'Raster', key: 'diorama' }))?.props.columns).toBe(40)
  disk.text = JSON.stringify(miniTheme(48))
  disk.mtime = 2
  await clock.advance(3000)
  expect((await ui.find({ type: 'Raster', key: 'diorama' }))?.props.columns).toBe(48)
})

test('a theme shipped with the plugin is named alone', { options: { theme: 'sorciere' } }, async ($, on) => {
  fakeDisk(on, JSON.stringify(miniTheme()))
  const { ui } = await start($, on)
  expect(disk.asked.some(p => p.endsWith('/themes/sorciere/theme.json') && !p.includes('.claude-plugin'))).toBe(true)
  expect((await ui.find({ type: 'Raster', key: 'diorama' }))?.props.columns).toBe(40)
})

