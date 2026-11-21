# 17 — How the Work Gets Done: the Delivery System

**Added 2026-11-19.** The PM: *"Creating an efficient system of creating the workload, assigning
Assistants to write campaigns, running campaigns and testing results, tracking progress and
updating documents atomically. … Failed work isn't dropped, it's passed to the next capable
agent."*

## 17.1 Who does what

| Who | Role | Does | Never does |
|---|---|---|---|
| **Kris** | PM | Decides direction; approves on the Review Desk; physical tests | — |
| **Mookie, Buddy** | Human testers and makers | Work through test passes; build the golden projects; report | — |
| **Claude** (architect) | Plans and unifies | Contracts, reference implementations, hidden tests, integration code, reviews, STATUS rows | Lets crew output into the repo unchecked |
| **The Crew** (Gordon, Deepak, Qwen, Hoss, Nemotron, Barry, Larry, Haiku) | Hands | One module, one test file, one icon, one research digest at a time, under gates they cannot read | Run anything; touch the repo directly |
| **The keeper** (scheduled, no model) | Keeps work moving | Resumes stopped campaigns; relays failures; files hand-offs; never spends money without a budget check | Make decisions that belong to a person |

## 17.2 The flow of one piece of work

```
idea / bug / project need
  → a TASK in docs/plan (one line: id, goal, owner, check)          [Claude or Kris]
  → a CONTRACT + REFERENCE + HIDDEN TESTS, reference passes them     [Claude]
  → a CAMPAIGN (crew/campaigns/mk_*.json), registered with the keeper [Claude]
  → crew members write, testers cross-test, gates judge              [the Crew]
       ├─ fail → rework → relay to the next holder of the post       [campaign]
       ├─ ladder used up → senior directives → one more attempt      [campaign]
       ├─ campaign stops (budget, crash, reboot) → keeper resumes it  [keeper]
       └─ still failing → hand-off to the next capable agent:         [keeper]
            other crew posts → OpenCode rescue → Claude queue → Review Desk
  → accepted module lands in the crew workspace                      [the Crew]
  → Claude vendors it into makercad on a branch, runs xtask ci       [Claude]
  → Review Desk: APPROVE COMMIT                                      [Kris]
  → commit, STATUS row with command + number                         [Claude]
```

**Nothing is dropped.** A task leaves the pipeline only as *accepted*, or as a hand-off item on the
Review Desk that names who has it now. A failure is information for the next agent, carried with
the best submission so far ("rework, not restart").

## 17.3 The keeper ("cron for the Crew")

A Windows scheduled task, **The Crew keeper**, runs every 15 minutes with no model calls:

1. **Watchdog**: marks lost, stalled and overdue runs (exists: `runners/watchdog.py`).
2. **Schedule tick**: fires due reminders and timed jobs (exists: `runners/schedule.py tick`).
3. **Keep campaigns going**: for every campaign registered with the keeper that is not finished
   and not running, it checks the budget, then resumes it through the heavy queue
   (`python -m crew campaign run SPEC --resume`). A resumed failed task goes to the **next member
   who has not had it**, carrying the best submission.
4. **Hand off what the crew can't do**: when every holder of a task's post has tried, the task
   becomes a hand-off: a brief with the contract, the best submission and every failed check,
   queued on the Review Desk and in `runs/crew/keeper/handoffs/` for the next capable agent
   (another post, an OpenCode rescue, or a Claude session).
5. **Report**: one line per decision in `runs/crew/keeper/keeper.log`, the current picture in
   `report.json` for the status overlay, and the night shift's 03:00 report counts it.

Safety: it only touches campaigns registered with it (`keeper add`), never more than one heavy run
at a time (the heavy queue), never when the weekly credit headroom is under the reserve, and at most
`max_resumes` times a day per campaign, so a broken campaign can't loop.

## 17.4 Documents updated atomically

- **One writer per document.** STATUS rows are appended by a script (`xtask status add`), which
  takes the claim, the command and the result, runs nothing itself, and writes through an atomic
  rename. Two agents can't interleave half-lines.
- **Docs change in the same commit as the code** they describe. A PR that changes a command's
  schema and not its doc fails `xtask docs`.
- **Plans are data.** `docs/plan/tasks.toml` holds every open task with its id, owner, state and
  check. The Board in The Crew's control room and the status overlay read it; nobody edits a
  progress table by hand.

## 17.5 Measuring the crew, honestly

Every campaign ends with the crew's own report (`REPORT.md`, `summary.json`), and
`runners/member_stats.py` keeps the record across campaigns. The MakerCAD build is one of the
"about ten real projects" over which the crew's usefulness is being judged, so each campaign's
cost, first-try rate and relay count go into STATUS alongside the code it produced.
