"""Builds the pause-point test pass: a wide look at what exists, how it was made, and what to decide.

Uses the test-pass skill's template. Images are the screens captured by tools/ui3_screens.ps1, embedded as small JPEGs
(tools/ui3_prepare.py), so the file stays one offline file.
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]  # the repo
TEMPLATE = Path.home() / ".claude" / "skills" / "test-pass" / "assets" / "template.html"
OUT = ROOT / "docs" / "test-passes" / "Test pass - MakerCAD pause point - 2026-11-23.html"
B64 = json.loads((ROOT / "target" / "p1" / "ui3" / "embed" / "b64.json").read_text(encoding="utf-8"))


def img(key, alt):
    return ('<img alt="%s" style="width:100%%;margin:8px 0 2px;border-radius:6px;border:1px solid var(--rule)" '
            'src="data:image/jpeg;base64,%s">' % (alt, B64[key]))


def img2(key_a, key_b, alt_a, alt_b):
    one = ('<img alt="%s" style="width:100%%;border-radius:6px;border:1px solid var(--rule)" '
           'src="data:image/jpeg;base64,%s">')
    return ('<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin:8px 0 2px">%s%s</div>'
            % (one % (alt_a, B64[key_a]), one % (alt_b, B64[key_b])))


FLOW = """
<div class="flow">
 <div><b>1. You say what you want</b>Your words become <b>decisions</b>: a numbered table (D1 to D34) in <code>docs/design/16_...md</code>, each with who said it, when, and when to revisit it.</div>
 <div><b>2. Claude designs and writes the contracts</b>For each module: the exact public names, numbered rules, a table of worked examples, and <b>hidden tests</b> the writer never sees.</div>
 <div><b>3. The Crew writes the modules</b>Cheaper models (Hoss, Deepak, Qwen, Kimi) write a module; another one writes tests from the contract alone. Nothing they write is trusted until it passes tests they cannot read.</div>
 <div><b>4. Checks nobody can argue with</b>Mutation testing (plant a bug; do the tests notice?), <code>clippy -D warnings</code>, <code>fmt</code>, a brand-free check, a layers check, a wasm check.</div>
 <div><b>5. Claude wires it into the window</b>The Crew's modules have no screen; the window (<code>apps/mk-desktop</code>) is where they meet a person.</div>
 <div><b>6. STATUS.md, one row per claim</b>Every row says the claim, the exact command, and the result. No command and number, no claim (rule 1 in <code>AGENTS.md</code>).</div>
 <div><b>7. The cloud builds it</b>GitHub Actions runs every gate again on a clean machine and publishes a Windows <code>makercad.exe</code>. That file is what you test.</div>
 <div><b>8. A test pass like this one</b>You look, mark what is wrong, and export text that comes back to Claude as findings.</div>
</div>
"""

GLOSSARY = """
<table>
<tr><th>Word</th><th>What it means here</th></tr>
<tr><td><b>Quest</b></td><td>"Start something new": nine questions (name, what to make, where it fits, tools, materials, when, budget, inspiration, workspace). Every one can be skipped.</td></tr>
<tr><td><b>My workshop</b></td><td>The tools and materials you own, kept in one small file on your PC (<code>library.db</code>) and reused by every project. Items typed into a project stay in that project (<i>local</i>) unless you add them at the end.</td></tr>
<tr><td><b>Sancho</b></td><td>The figure beside every design. Today: a still, posable scale figure (1.68 m placeholder, BodyKun). Later: a game-like character that reacts to your workspace. He is never the place a warning appears (D32).</td></tr>
<tr><td><b>Findings</b></td><td>Checks the rules make on a design (a screw too long, a chest that can tip). Today shown for a built-in demo dresser, not your design.</td></tr>
<tr><td><b>Environment</b></td><td>The tabs: Design, Stock &amp; Cut, Plan, Build, Space, Share. Only Design is live.</td></tr>
<tr><td><b>The Crew</b></td><td>The cheaper AI models that write modules and tests under gates they cannot fake. Claude architects and reviews.</td></tr>
<tr><td><b>Gates</b></td><td>The automatic checks that must be green before a commit counts (format, lints, tests, brand-free, layers, wasm).</td></tr>
<tr><td><b>STATUS.md</b></td><td>The repo's ledger of claims, each with its command and result.</td></tr>
<tr><td><b>D-number</b></td><td>A numbered decision (D30: global and local; D31: tool days; D32: Sancho is not an alert channel; D33: small common tools; D34: the document's measuring system).</td></tr>
<tr><td><b>.maker / .figcraft</b></td><td>A MakerCAD project file / a FigCraft design file. MakerCAD can read the second directly (D29).</td></tr>
<tr><td><b>The potato</b></td><td>The reference PC (i5-6600, Intel HD 530, 8 GB, Windows 10): if it runs well here it runs well anywhere.</td></tr>
</table>
"""

REAL = """
<table>
<tr><th>Real: built, tested, and in the window</th><th>Built as a module, <b>not</b> in the window yet</th><th>Not built</th></tr>
<tr>
<td>
The shell: project bar, environment tabs, toolbox frame, status bar, command search (Ctrl+K), three looks.<br><br>
The opening screen, project files (<code>.maker</code>, crash-safe saving with a recovery copy), Ctrl+S, recent projects.<br><br>
The Design workspace as a <b>viewer</b>: SolidCraft's examples, orbit/zoom, Sancho at true size.<br><br>
The Quest (nine questions) that makes a project.<br><br>
My workshop (SQLite), global and local items, tool days, the end-of-project offer.<br><br>
Each project's measuring system and typed lengths (D34).
</td>
<td>
The Intent card, the toolbox that follows your answers, observations, Sancho's voice and relationship memory, his animation clips and light-rig math, the journal and its prompts, the session-end interview, the plan-ready rule packs wired to a real design (<code>design_facts</code>).<br><br>
These are tested modules (see STATUS) with no screen yet.
</td>
<td>
<b>Any modelling tool</b>: the toolbox has only view buttons; "Sketch and make tools come next". You cannot yet draw or pull up a part in MakerCAD.<br><br>
Stock &amp; Cut, Plan, Build, Space, Share (each tab says which phase brings it).<br><br>
Findings on your own design (the panel shows a demo dresser).<br><br>
A plan that uses a tool's days.<br><br>
Sancho reacting to anything.
</td>
</tr>
</table>
"""

ITEMS = [
 {"n": "00", "t": "Start here: what this pass is for", "time": "~5 min",
  "note": "We have reached a good place to stop. The program is not finished, but many of its parts now exist, "
          "are tested, and can be looked at. This pass is not only a checklist: it is meant to help you see "
          "<b>what exists, how it was made, what is real and what is a placeholder</b>, and to collect what you "
          "want next, including what you want to design yourself in FigCraft. Nothing here needs you to "
          "write code. Mark each point OK, Needs work or n/a, and write in the box whatever you think. "
          "Everything saves in this browser as you type, and nothing leaves this machine. The earlier pass "
          "(<i>Quest, workshop and Findings</i>, 50 points) is the detailed one for those three areas; this "
          "one is wider, so do this one first. The handoff notes are in <code>HANDOFF.md</code> at the top "
          "of the repo.",
  "html": GLOSSARY,
  "items": [
   {"id": "s_run", "t": "Start the program", "isnew": 1,
    "d": "Download the newest <code>makercad-testNN-windows-x64.zip</code> from "
         "github.com/Messyhairkris/makercad/releases (about 88 MB, an optimised build), unzip it and run "
         "<code>makercad.exe</code>. Windows may say it is an unrecognised app, because the build is not "
         "signed. (Or run the debug build in the repo: <code>third_party\\solidcraft\\target\\debug\\makercad.exe</code>.)",
    "e": "The opening screen: a warm cream background (the Daydream look), two big cards (\"Start something "
         "new\", \"Keep going\"), \"Recent projects\", \"My workshop\" with its counts, and Sancho waving "
         "with a speech bubble. The bottom bar says \"MakerCAD 0.1.0 \u00b7 foundation build\", has mm and in "
         "buttons, and names the graphics backend (dx12 or gl). I measured 1.1 to 1.7 s from launch to the "
         "first frame on the potato in a debug build; I have not timed the optimised build or unzipped "
         "the release myself, so how long it took and what was in the zip is exactly what I need from you."},
   {"id": "s_words", "t": "The words in the table above", "isnew": 1,
    "d": "Read the table of words under this section's title.",
    "e": "Every word should make sense without asking me. Mark Needs work and list the ones that do not, in "
         "your own words. A word you would have chosen differently is just as useful as one you did not "
         "follow."},
  ]},

 {"n": "01", "t": "How this is being made", "time": "~10 min",
  "note": "You asked for help understanding the process. This is the whole chain, from what you say to a "
          "file you can run. The point of every step is that nothing is believed because someone said it "
          "worked: each claim has a command and a number.",
  "html": FLOW,
  "items": [
   {"id": "p_flow", "t": "Does the chain above match how you thought the work goes?", "isnew": 1,
    "d": "Read the eight boxes.",
    "e": "Say which box surprised you, which one you would drop, and which one you would like to see more "
         "of. One thing to know: the Crew's modules are written in separate pieces with no window of their "
         "own. The window is written by Claude, so a feature can be finished as a module and still be "
         "invisible to you for days. That gap (built, not yet shown) is the most common reason you might "
         "think something is missing."},
   {"id": "p_decisions", "t": "The decisions are yours: do they say what you said?", "isnew": 1,
    "d": "Open <code>docs/design/16_Modules_Compatibility_and_Growth.md</code> and find the decision table "
         "(rows D1 to D34). Read D26 to D34 (the last week): D26 the Sancho placeholder, D27 Sancho is a "
         "scale figure first, D28 the public repo and cloud build, D29 FigCraft files as UI, D30 global and "
         "local, D31 tool days, D32 Sancho is not an alert channel, D33 small common tools, D34 the "
         "document's measuring system.",
    "e": "Each row has the decision, your words (\"Source\"), when to revisit it, and a status. Every one "
         "marked Accepted (PM) is something you said. Quote any row that does not say what you meant. D29 "
         "(FigCraft files as UI) is only \"Proposed\": it is waiting for you."},
   {"id": "p_status", "t": "A STATUS row, read like a receipt", "isnew": 1,
    "d": "Open <code>STATUS.md</code> and read the two newest bold rows (the tool days one and the "
         "measuring-system one).",
    "e": "Each row has three cells: the claim, the exact command, and what happened, including what did not "
         "work and what is not done (\"Not yet: ...\"). If a row is written in words you do not follow, mark "
         "it Needs work and say which words. These rows are the writing I most need to get right for you."},
   {"id": "p_cloud", "t": "The cloud build agrees with this PC", "isnew": 1,
    "d": "Open github.com/Messyhairkris/makercad/actions and look at the newest runs.",
    "e": "Runs #24 to #31 finished green on both jobs (\"Gates\": format, brand-free, layers, wasm, clippy, "
         "tests; and \"Windows build\"). The newest run was still building when I wrote this, so look at it "
         "first. Every run publishes a test build: 31 so far at about 88 MB each is about 2.7 GB of release "
         "files (a decision for you in the last section)."},
   {"id": "p_crew", "t": "How much of this did the Crew build?", "isnew": 1,
    "d": "Skim the list of crates in <code>crates/</code> and the contracts in "
         "<code>ATLAS-api-harness/crew/contracts/mk_*.md</code>.",
    "e": "The repo has 8 library crates plus the window, about 66,000 lines of Rust (tests included) in 139 "
         "files, 35 commits, and 34 Crew contract files. The Crew wrote and tested the units, nesting, "
         "inventory, project-file, journal, rules and Sancho modules; Claude wrote the window, finished the "
         "contracts, and fixed what the Crew's tests missed. How do you feel about that split? Is there "
         "something you would want Claude to write by hand that the Crew now writes, or the other way round?"},
  ]},

 {"n": "02", "t": "The screens, as built (look, do not click)", "time": "~20 min",
  "note": "<b>Click a picture to see it larger</b> (click again to close). These are real screenshots of the running program, taken today with the program's own "
          "<code>--screenshot</code> mode (not mock-ups). The same screens are in "
          "<code>docs/design/ui-v3/</code> as full-size PNGs to import into FigCraft. For each: what you are "
          "looking at, then what I want to know. Several screens show the Quest filled with a sample "
          "dresser so they are not empty. I have looked at these pictures; nobody has yet clicked through "
          "them, which is the next section.",
  "items": [
   {"id": "t_home", "t": "Opening screen", "isnew": 0,
    "d": "Look at the picture. <b>Start something new</b> and <b>Keep going</b> are the two big cards; "
         "Recent projects and My workshop sit below; Sancho is on the right." + img("01-home", "The opening screen"),
    "e": "What you should see: no projects yet, so \"Keep going\" says nothing is under way; My workshop "
         "says \"14 tools \u00b7 3 material lines\" (the sample workshop I made for these pictures; yours "
         "will say what you have). Sancho is 168 cm (5' 6\"). Questions: what is your first word for the "
         "feeling? Is \"Start something new\" the right first thing, or should \"Keep going\" lead once a "
         "project exists? Is Sancho the right size and place, and should he say something different each "
         "time?"},
   {"id": "t_looks", "t": "The three looks", "isnew": 0,
    "d": "Daydream is the default (picture above). These two are Night Workshop (left) and Pastel (right)."
         + img2("02-home-night", "03-home-pastel", "Night Workshop look", "Pastel look"),
    "e": "Change them yourself in Settings (the gear, top right). Questions: which would you use all day? "
         "Is Pastel different enough from Daydream to keep? Is there a look missing (a high-contrast one "
         "is planned for accessibility)? Does anything become hard to read in the dark one?"},
   {"id": "t_design", "t": "The Design workspace", "isnew": 0,
    "d": "Look at the picture. Project bar on top, environment tabs under it, a narrow toolbox on the left, "
         "the part and Sancho in the middle, a status bar at the bottom." + img("04-design", "The Design workspace"),
    "e": "What you should see: SolidCraft's \"Sample Plate\" (a small part) on a workbench at table height, "
         "with Sancho at 1.68 m standing on the floor beside it, so you read the part's size against a "
         "person. The toolbox has only the View buttons (Fit, All, Corner, Front, Top, Right) and says "
         "\"Sketch and make tools come next\". Be honest: you cannot yet draw or pull up a part here. "
         "Questions: does the empty toolbox read as \"not built yet\" or as \"broken\"? Is the bench "
         "and the floor patch the right idea? What is the first modelling tool you want to see there?"},
   {"id": "t_search", "t": "Command search (Ctrl+K)", "isnew": 0,
    "d": "Look at the picture, where \"round\" was typed." + img("05-command-search", "Command search with round typed"),
    "e": "What you should see: three results. \"Round over\" (Make, \"comes in P1\"), \"Circle\" (Sketch), "
         "and \"Ask Sancho: round\" (\"Sancho explains, using your part\", \"comes in P2\"). Search "
         "understands maker words (\"round over\") and CAD words (fillet, extrude, chamfer). Commands "
         "that are not built yet are listed anyway, with the phase that brings them. Questions: is "
         "listing things that do not work yet useful (it shows where everything will live) or "
         "frustrating? Is \"Ask Sancho\" as the last row a good idea?"},
   {"id": "t_stock", "t": "A tab that is not built yet", "isnew": 0,
    "d": "Look at the picture of the Stock &amp; Cut tab." + img("06-stock-tab", "The Stock and Cut tab"),
    "e": "Each unbuilt environment (Stock &amp; Cut, Plan, Build, Space, Share) shows a page saying what "
         "it will do and \"comes in P3\" (or the phase that brings it). Question: are these pages worth "
         "having (the shape of the whole program is visible from day one) or would you rather hide the "
         "tabs until they work?"},
   {"id": "t_quest_name", "t": "The Quest: one question per page", "isnew": 0,
    "d": "Look at the picture of the first Quest page." + img("07-quest-name", "The first Quest question"),
    "e": "A rail on the left lists the nine questions (green ticks are answered, the coral one is where "
         "you are), the question is in the middle, and \"What your answers set up\" on the right fills in "
         "as you type (here empty). \"Skip to designing\" (coral, top right) and \"Cancel\" are always "
         "there. Questions: one question per page, or the whole form at once? Is the right-hand panel "
         "useful or too much? Is \"Skip to designing\" in the same coral as the main button a problem "
         "(two loud buttons)?"},
   {"id": "t_quest_tools", "t": "The Quest: tools, with your workshop", "isnew": 0,
    "d": "Look at the picture of the tools step." + img("09-quest-tools", "The Quest tools step"),
    "e": "Eight kits (Basic household, Starter woodshop, ...), a box to type a tool, then chips: filled "
         "blue chips are from your workshop, chips with a coral outline are only for this project (D30). "
         "The right panel says what the answers set up: the environments (each with the word that "
         "triggered it), the scale (\"Furniture, its longest size is 1500 mm\"), the toolbox it will "
         "start with, and a \"Sancho noticed\" hint. Questions: do the two kinds of chip read at a glance? "
         "Is a kit the right way to start, or would a checklist of ten everyday tools be quicker (your "
         "D33: a small list of what most people own)?"},
   {"id": "t_quest_review", "t": "The Quest: the review", "isnew": 0,
    "d": "Look at the picture of the last step." + img("11-quest-review", "The Quest review"),
    "e": "Step 9 is a review of everything the questions set up before the project is made. Question: is "
         "this the right place to stop and see the whole, or would you rather see it grow on the right "
         "all the way through (as it does now) and skip the review?"},
   {"id": "t_workshop", "t": "My workshop", "isnew": 0,
    "d": "Look at the picture of the first workshop step." + img("12-workshop-tools", "My workshop, tools"),
    "e": "Three steps (Your tools, Your materials, All set). Each tool row has how you have it (I own it, "
         "Shared, At a makerspace, I can rent it, Wishlist), when you can use it (\"any day\" or the days), "
         "and a \u00d7. Questions: is this a place you would keep up to date? What would make you open it "
         "(a reminder when a project finishes? when a plan needs a tool you lack?)?"},
   {"id": "t_finish", "t": "Finishing a project", "isnew": 0,
    "d": "Look at the picture of the end-of-project offer." + img("14-finish-offer", "The end-of-project offer"),
    "e": "Over the Design workspace: the project's own materials and tools, each with a tick; ticked ones "
         "go to your workshop (with how many are left, or how you have the tool). Today you reach it with "
         "Ctrl+K, \"Finish the project\"; the project does not yet know it is finished. Question: should "
         "finishing be a button, and where would you expect to find it?"},
   {"id": "t_findings", "t": "Findings", "isnew": 0,
    "d": "Look at the picture. A panel on the right lists what the rules found." + img("15-findings", "The Findings panel"),
    "e": "It is a <b>demonstration</b>: the built-in golden dresser with five faults put in on purpose (a "
         "screw that pokes out, a chest that can tip over, ...), sorted worst first, each with a \"Why?\" "
         "that shows the rule and where its numbers come from. It says so at the top. Questions: is this "
         "how you want warnings to look? Is the name \"Sancho's findings\" right now that Sancho is a "
         "character and not an alert channel (D32)?"},
  ]},

 {"n": "03", "t": "Try it yourself: a five-minute walk", "time": "~15 min",
  "note": "Now click. I have only seen these flows through screenshots and unit tests, so this is the "
          "first time a person drives them end to end. If anything is clumsy, slow, ugly or wrong, "
          "that is exactly what I want to know, and the note box is the place for it. Use a throwaway "
          "project name; projects are saved in <code>Documents\\MakerCAD</code>.",
  "items": [
   {"id": "a_new", "t": "Start a project", "isnew": 1,
    "d": "Press <b>Start something new</b>. Name it <b>Test dresser</b>. On the next page type: "
         "<b>a dresser for the bedroom alcove, four drawers</b>.",
    "e": "As you type the goal, the right-hand panel lists the environments and why: Solid, Sketch and "
         "Stock &amp; Cut, each \"because you said \"dresser\"\". The left rail ticks the name step "
         "green when you move on. Back works. If you press Enter in a box it moves to the next step."},
   {"id": "a_size", "t": "Say how big it is, in any units", "isnew": 1,
    "d": "On \"Where does it need to fit?\" pick <b>I'll type sizes</b> and type <b>5 ft</b> in "
         "<b>Its longest size</b>. Do not press Enter yet. Then switch the project to <b>Imperial "
         "(inches)</b> and back.",
    "e": "Under the box: \"That is 1524 mm. Press Enter to keep it.\" and \"That's Furniture scale.\" "
         "Enter turns the text into <b>1524 mm</b>. Switching to Imperial rewrites it as <b>60\"</b>. This "
         "is your D34 (the document's measuring system); the earlier pass has more points on it."},
   {"id": "a_tools", "t": "Tools and materials", "isnew": 1,
    "d": "Tools: press <b>Basic household</b>, then type <b>jigsaw</b> and Add. Materials: type "
         "<b>15 pine 2x4s, 3 foot</b> and Add.",
    "e": "The Basic household kit gives five chips (Tape measure, Utility knife, Screwdrivers, Hammer, "
         "Scissors); jigsaw is added as a sixth. The materials row reads back what it understood (a "
         "quantity, the board size 38 \u00d7 89 mm, the length 914 mm). Is the way it reads your words "
         "right? What would you have typed that it did not understand?"},
   {"id": "a_make", "t": "Make the project", "isnew": 1,
    "d": "Press <b>Skip to designing</b> (top right), or go to the end and confirm.",
    "e": "The Design workspace opens with an empty design, and a notice reads \"Test dresser is made and "
         "saved as Test dresser.maker.\" (plus the scale). The file is in <code>Documents\\MakerCAD</code>. "
         "Skipped questions stay open for later. Is the empty workspace a letdown after the Quest? What "
         "should it show first?"},
   {"id": "a_save", "t": "Save, go home, come back", "isnew": 1,
    "d": "Press Ctrl+S, then the home icon (top left). Open the project from the opening screen.",
    "e": "Home lists the project under Recent projects, and \"Keep going\" names it with when it was "
         "saved. Opening it puts you back in the Design workspace. If you damage the file (rename it "
         "to something else and back is fine; do not edit it), the app opens its recovery copy instead of "
         "failing; I tested that by unit test only."},
   {"id": "a_search", "t": "Find things with Ctrl+K", "isnew": 1,
    "d": "Press Ctrl+K and type <b>workshop</b>, then <b>finish</b>, then <b>findings</b>, then <b>inches</b>.",
    "e": "My workshop; Finish the project; Show the findings; Use inches. Does the search find the "
         "thing you meant for the words you would really use? Write the words you tried that found nothing."},
   {"id": "a_findings", "t": "Look at the Findings demo", "isnew": 0,
    "d": "Choose \"Show the findings\". Press <b>Why?</b> on the first finding.",
    "e": "A box titled \"Screw through the stock\" with the numbers (fastener screw_1, length 40 mm, stock "
         "36 mm) and a sentence on why the screw must stop inside the board. Does the explanation teach "
         "you something, or is it too technical?"},
   {"id": "a_inches", "t": "Change the project's measuring system", "isnew": 1,
    "d": "Look at the bottom right while the project is open. Hover <b>in</b> and <b>mm</b>, press "
         "<b>in</b>, and look at Sancho's height in the bar.",
    "e": "The hover says \"This project measures in inches\"; Sancho's height changes from \"1.68 m\" to "
         "feet and inches. This changes the open project, not your default for new ones (Settings has "
         "both rows)."},
   {"id": "a_break", "t": "Try to break it", "isnew": 1,
    "d": "Type nonsense in every box, resize the window small and large, press Esc everywhere, close the "
         "program in the middle of the Quest and start it again.",
    "e": "It never crashes (rule 2 of <code>AGENTS.md</code>: the program must never crash, whatever is "
         "typed). A bad date or an unreadable size says what is wrong and keeps what you typed. If anything "
         "crashed, froze or looked wrong, write exactly what you did in the box: that is the most valuable "
         "thing you can send me."},
  ]},

 {"n": "04", "t": "What is real, and what is still a placeholder", "time": "~5 min",
  "note": "Many pieces exist; fewer are visible. The first task of a pause is to be honest about the "
          "gap. This table is how I see it today. The most likely misunderstanding is the third "
          "column: <b>there are no modelling tools yet</b>, because the Crew and I built the layers "
          "under them first (rules, inventory, projects, the window).",
  "html": REAL,
  "items": [
   {"id": "r_expect", "t": "Which of these did you think already worked?", "isnew": 1,
    "d": "Read the three columns.",
    "e": "Name anything in the right-hand columns you assumed was already there. That tells me where my "
         "descriptions have been misleading, and where your expectations are ahead of the program."},
   {"id": "r_order", "t": "Is the order right?", "isnew": 1,
    "d": "Compare with the roadmap in <code>docs/design/14_Roadmap.md</code> (P0 to P10).",
    "e": "The roadmap puts the project layer (P2) before Stock &amp; Cut (P3) and the sketch tools (P4). "
         "The window is therefore a good place to start a project and a poor place to model anything. "
         "Would you have chosen a different order, for example a small modelling tool first so you could "
         "build something and test the rules on it?"},
  ]},

 {"n": "05", "t": "Feel: what to take into FigCraft", "time": "~15 min",
  "note": "You said you want to develop FigCraft examples of the UI to shape the project's feel, and "
          "that is the best use of the pause. Two sets of screens are ready: "
          "<code>docs/design/ui-v2/</code> (six screens of the Design workspace <i>in use</i>, as editable "
          "SVGs with 21 numbered questions in its README) and <code>docs/design/ui-v3/</code> (the "
          "fifteen real screens in this pass, as PNGs, with their questions). Design the screens in "
          "FigCraft however you like, then save the <code>.figcraft</code> file where it is easiest "
          "for you (the repo's <code>docs/design/ui-v3/notes/</code>, or the share). MakerCAD reads "
          "<code>.figcraft</code> files directly (D29, still only proposed).",
  "items": [
   {"id": "f_send", "t": "Getting your FigCraft work back to me", "isnew": 1,
    "d": "Import the ui-v3 PNGs (and the ui-v2 SVGs) into FigCraft, annotate or redraw them, and save a "
         "<code>.figcraft</code> file.",
    "e": "<code>tools/figcraft_read.py FILE.figcraft</code> reads frames, text, comments, annotations "
         "and prototype links. It passed three tests on a file I built from FigCraft's own schema; it has "
         "never read a real FigCraft file, so your first one is its first real test. If it fails, tell me "
         "(or send the REST export instead; D29 names that as the fallback). Did importing the pictures "
         "into FigCraft work? Were the SVGs editable?"},
   {"id": "f_first", "t": "Which screens do you want to design first?", "isnew": 1,
    "d": "Pick, in order: opening screen, the Quest page, the Design workspace, My workshop, Findings, "
         "or one that does not exist yet (Stock &amp; Cut, Plan).",
    "e": "Write your order in the box. Screens that do not exist yet (Stock &amp; Cut, Plan, Build) are "
         "the cheapest place to decide the feel, because nothing has to be undone."},
   {"id": "f_words", "t": "Words: maker words first", "isnew": 1,
    "d": "Look at the toolbox and command names you have seen (\"Round over\", \"Pull up\", \"Drawer "
         "wizard\", \"Shelf pins\").",
    "e": "The direction is maker words first, CAD words one step away (\"Round over\", search also knows "
         "\"fillet\"). Is that still what you want? List words you would never say, and words that "
         "are missing."},
   {"id": "f_density", "t": "How much is on the screen?", "isnew": 1,
    "d": "Look again at the Quest page and the Design workspace.",
    "e": "Both are calm: a lot of empty space. The cost is that a long list (the Quest's tool chips, "
         "the Findings) scrolls. Do you want it calmer, denser, or different per screen? (CAD users often "
         "want dense; the target maker is not necessarily a CAD user.)"},
   {"id": "f_colour", "t": "Colour and emphasis", "isnew": 1,
    "d": "Look at the Quest page again: \"Skip to designing\" and \"Next\" are both coral.",
    "e": "Coral is the action colour in Daydream. On the Quest two coral buttons compete, and \"Skip to "
         "designing\" is the one that discards the rest of the questions. Which should be loud? What "
         "colour is the program for you (a workshop, a storybook, a studio)?"},
   {"id": "f_icons", "t": "Icons", "isnew": 1,
    "d": "Look at the toolbox, tabs and kit chips.",
    "e": "The icons were drawn by the Crew to a set of rules (a 16 px grid, outline style); 60 are built. "
         "Which look wrong, too generic, or are unreadable at small size? Name them."},
   {"id": "f_sancho", "t": "Sancho: what should he react to?", "isnew": 1,
    "d": "Think of Sancho as a character in a game that notices what you do.",
    "e": "D32: he is never the channel for warnings; he reacts, like a game character reacts to the "
         "world. Write a list of moments you would want him to react to (a board placed, a first cut, "
         "a finished project, a long pause, a mistake caught) and how (a pose, a face, a line). This is "
         "the brief for his animation work, which waits until the planning tools exist."},
   {"id": "f_first60", "t": "The first sixty seconds", "isnew": 1,
    "d": "Imagine someone who has never seen the program opening it for the first time.",
    "e": "Today they see the opening screen, then the Quest. What should they feel, and what should "
         "they have done, after a minute? (A made project? A part on the bench? A joke from Sancho?)"},
   {"id": "f_ui2", "t": "The six \"in use\" screens", "isnew": 0,
    "d": "Open <code>docs/design/ui-v2/README.md</code> and look at its six screens.",
    "e": "That README has 21 numbered questions (project bar, toolbox, timeline, sketching, pull up, "
         "command search, changing a step, Sancho and the stage). You can answer any of them here, in "
         "FigCraft, or not at all. Which three matter most?"},
  ]},
]

CALL = {"n": "06", "t": "Your call", "time": "~10 min",
        "note": "Decisions only you can make, with the real numbers. I have not tucked a recommendation "
                "into any of them. Already settled by you: a project measures in one system (D34); Sancho "
                "is never the alert channel (D32); the tool list stays small and common (D33); tools and "
                "materials are global or local (D30), and tools can have days they are available (D31).",
        "items": [
   {"id": "y_next", "t": "What should we build after the break?", "isnew": 1,
    "d": "Five options. Their dependencies are real; their sizes are my honest guesses.",
    "e": "(A) Modelling tools in the Design toolbox: a button runs one of SolidCraft's engine commands "
         "(sketch a rectangle, pull up, round over). Needed before anyone can build anything here. "
         "Medium-large. (B) Findings on your own design: each part gets a material from the project's list "
         "and the rules read the real design instead of the demo dresser. Needs (A). Medium. (C) Stock "
         "&amp; Cut (phase P3): bind a part to a board, cut lists, the nesting crate (<code>mk-nest</code>, "
         "already written) in the window, the waste meter. Needs (A) and parts with materials. Large. "
         "(D) A plan from the steps, using tool days (phase P5). Needs (A) and (C). Large. (E) A polish "
         "round from your FigCraft notes, on the screens that exist. Small to medium. The order is "
         "yours. Which first, and why?"},
   {"id": "y_figcraft_first", "t": "FigCraft before or after the next build?", "isnew": 1,
    "d": "You said you want to develop the FigCraft examples during this pause.",
    "e": "Building the next features on the screens as they are is cheaper to start but may be redone; "
         "waiting for your FigCraft work costs time but means the next build is built to your design. "
         "(E) above is the only option that needs your notes first. Do you want the next build to wait "
         "for them?"},
   {"id": "y_releases", "t": "Keep every test build?", "isnew": 1,
    "d": "Every push publishes a Windows test build as a GitHub release.",
    "e": "31 releases so far at about 88 MB each is about 2.7 GB. They never need to be deleted for "
         "GitHub's sake as far as I know, but old ones are no use. Options: keep all; keep the newest ten; "
         "keep only the newest and the one you last tested. Which?"},
   {"id": "y_finished", "t": "Should a project know it is finished?", "isnew": 1,
    "d": "Today finishing is a command (Ctrl+K, Finish the project).",
    "e": "Options: (a) keep the command only; (b) a Finish button on the project that marks it finished "
         "in the .maker file, opens the leftovers offer, and shows it as done on the opening screen. (b) "
         "needs a new field in the project file and a finished state to draw. Which?"},
   {"id": "y_numbers", "t": "Sourced numbers, or marked handbook values?", "isnew": 1,
    "d": "The structure rules need each material's stiffness and limits and a sag allowance.",
    "e": "Now: pine 10,000 MPa, plywood 6,000 MPa, 2 mm sag per metre of span, all marked \"not yet "
         "sourced\". Gordon can research real figures with cited sources and I would check each against "
         "its source. Until then, should the Findings show warnings built on unsourced numbers, or stay "
         "silent for any material without sourced values?"},
   {"id": "y_relevant", "t": "\"You have 41 tools; these 9 matter for a dresser\"", "isnew": 1,
    "d": "Doc 05 section 5.3 promises the Quest narrows your workshop to what the project needs.",
    "e": "Today \"Use my workshop\" copies every tool you own into the project. Narrowing needs a table of "
         "which tools each kind of project needs; the Crew can write it, but for any kind of project the "
         "table lacks it is a guess. Do it now, or leave \"all my tools\" until a plan exists to say which "
         "steps need which tool?"},
   {"id": "y_precision", "t": "Is 1/64 inch (and 0.01 mm) the right precision?", "isnew": 1,
    "d": "An imperial project keeps lengths to 1/64\", a metric one to a hundredth of a millimetre; "
         "rounding is said before you confirm.",
    "e": "1/64\" is about 0.4 mm, so a length typed in millimetres can move up to 0.2 mm in an imperial "
         "project (10 mm becomes 9.92 mm). Options: (a) keep it; (b) each project chooses 1/16, 1/32, "
         "1/64 or decimal inches; (c) imperial keeps decimals when what is typed is not a fraction. Which?"},
   {"id": "y_findings_name", "t": "Should the Findings panel carry Sancho's name?", "isnew": 1,
    "d": "It is called \"Sancho's findings\"; the Quest review has a \"Sancho noticed\" box.",
    "e": "You said Sancho handling alerts would hurt him as a character (D32). A findings panel is an "
         "alert channel by nature. Options: (a) rename it plainly \"Findings\" and keep Sancho as a figure "
         "who reacts; (b) keep his name on both as friendly framing; (c) rename the panel but keep the "
         "Quest's hints in his voice, since they are suggestions, not warnings. I have changed nothing. "
         "Which?"},
   {"id": "y_instruments", "t": "Do measuring tools count as tools?", "isnew": 1,
    "d": "You said a tool modifies or refines a material. A tape measure, calipers and a level measure.",
    "e": "I left them in because a plan needs to know you can measure. Options: (a) keep them in the "
         "list; (b) a separate short \"Measuring\" list in the same walkthrough step. And pliers, an "
         "adjustable wrench and a pencil are not in the list: add them to the Basic household kit? "
         "That means a new revision of the tool table, which I will not do without your yes."},
 ]}

ITEMS.append(CALL)

# `d` and a section's note and html may carry markup; `e` and `t` are shown as plain text, so keep them plain.
for sec in ITEMS:
    for it in sec["items"]:
        it["e"] = re.sub(r"<[^>]+>", "", it["e"]).replace("&amp;", "&")
        it["t"] = it["t"].replace("&amp;", "&")

FINAL_JS = """const FINAL = [
  {id:"f_obs",   label:"Observations",
   ph:"Anything at all.",
   prompts:["In your own words, what is MakerCAD right now, to someone who has not seen it?",
            "What surprised you, in either direction?",
            "Anything that felt slow, awkward, confusing, or ugly?",
            "Anything that worked better than you expected?",
            "Did you do anything the instructions did not cover?"]},
  {id:"f_process", label:"Did this help you understand how the program is being made?",
   ph:"What is clearer now, and what is still a fog?"},
  {id:"f_first", label:"If you could only change one thing before the next build", ph:"The one that mattered most."},
  {id:"f_env",   label:"Anything about your setup worth knowing",
   ph:"Browser, machine, screen size, what else was running."}
];"""

html = TEMPLATE.read_text(encoding="utf-8")

# Extra styles: the flow diagram, tables and the raw-HTML block under a section's note.
extra_css = """
.sechtml{margin:0 0 14px}
.sechtml table{border-collapse:collapse;width:100%;font:13.5px/1.45 "Segoe UI",system-ui,sans-serif;background:var(--panel)}
.sechtml th,.sechtml td{border:1px solid var(--rule);padding:6px 9px;text-align:left;vertical-align:top}
.sechtml th{background:var(--panel-2)}
.flow{display:grid;gap:14px}
.flow>div{background:var(--panel);border:1px solid var(--rule);border-left:3px solid var(--accent);
          border-radius:6px;padding:8px 12px;font:14px/1.5 "Segoe UI",system-ui,sans-serif;position:relative}
.flow>div b:first-child{display:block;margin-bottom:2px}
.flow>div:not(:last-child)::after{content:"\\2193";position:absolute;left:14px;bottom:-14px;color:var(--faint);font-size:13px}
.what img{max-width:100%}
"""
html = html.replace("</style>", extra_css + "</style>", 1)

# A section may carry a raw HTML block (diagram, table) after its note.
html = html.replace(
    "+ (sec.note ? '<p class=\"snote\">' + sec.note + '</p>' : '');",
    "+ (sec.note ? '<p class=\"snote\">' + sec.note + '</p>' : '')\n"
    "       + (sec.html ? '<div class=\"sechtml\">' + sec.html + '</div>' : '');", 1)

# Click a picture to see it larger (a plain overlay: no network, nothing stored).
html = html.replace("</style>", "#lb{display:none;position:fixed;inset:0;background:rgba(0,0,0,.82);z-index:50;"
                    "overflow:auto;cursor:zoom-out}#lb img{display:block;margin:20px auto;width:min(1000px,96vw);"
                    "border-radius:6px}.what img{cursor:zoom-in}</style>", 1)
html = html.replace("</body>", """<div id="lb"><img alt=""></div>
<script>
document.addEventListener("click", function (e) {
  var lb = document.getElementById("lb");
  var pic = e.target.closest && e.target.closest(".what img");
  if (pic) { lb.firstChild.src = pic.src; lb.style.display = "block"; return; }
  if (lb.style.display === "block") lb.style.display = "none";
});
</script>
</body>""", 1)

html = html.replace("<title>PROJECT Test Pass \u2014 local</title>",
                    "<title>MakerCAD Test Pass \u2014 pause point \u2014 local</title>")
html = html.replace("<span>PROJECT \u00b7 local copy \u00b7 nothing leaves this machine</span>",
                    "<span>MakerCAD \u00b7 2026-11-23 \u00b7 local copy \u00b7 nothing leaves this machine</span>")
html = html.replace("<h1>Test Pass</h1>", "<h1>A good place to pause</h1>")
html = html.replace(
    "Saves to this browser as you type \u2014 no network, no\n  reconnecting, nothing to lose. <b>Export</b> when you want to send results\n  back.",
    "What exists, how it was made, what is real and what is a placeholder, what you want to shape in FigCraft, "
    "and what to build next. About 80 minutes, in any order, with no rush. Saves to this browser as you type; "
    "<b>Export findings</b> gives text to paste back into the conversation.")
start = html.index("const ITEMS = [")
end = html.index("/* Final notes prompts")
html = html[:start] + "const ITEMS = " + json.dumps(ITEMS, indent=1, ensure_ascii=False) + ";\n\n" + html[end:]
# Replace the template's FINAL block with ours.
fs = html.index("const FINAL = [")
fe = html.index("];", fs) + 2
html = html[:fs] + FINAL_JS + html[fe:]
html = html.replace('const KEY = "REPLACE-slug-test-pass-v1";', 'const KEY = "makercad-pause-point-2026-11-23-v1";')
html = html.replace('"PROJECT test pass \u2014 "', '"MakerCAD pause-point test pass \u2014 "')
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(html, encoding="utf-8", newline="\n")
n = sum(len(s["items"]) for s in ITEMS)
print(f"{OUT.name}: {len(ITEMS)} sections, {n} points, {len(html) // 1024} KB")
