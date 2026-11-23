"""Builds the MakerCAD test pass from the test-pass skill's template."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]  # the repo
TEMPLATE = Path.home() / ".claude" / "skills" / "test-pass" / "assets" / "template.html"
OUT = ROOT / "docs" / "test-passes" / "Test pass - MakerCAD Quest workshop Findings - 2026-11-22.html"

# Written relative to the repo folder: the repository is public, so no user names in paths (AGENTS.md rule 11).
EXE = r"third_party\solidcraft\target\debug\makercad.exe"

ITEMS = [
 {"n": "00", "t": "Start it up", "time": "~5 min",
  "note": "A debug build of today's work sits at <code>" + EXE + "</code> in the repo folder (it opens a little slower than the "
          "release build will). The cloud also builds a release <code>makercad.exe</code> for every push: "
          "GitHub, <code>Messyhairkris/makercad</code>, Actions. I have not measured the release build's "
          "start time on this PC, so the \"under 2 seconds\" goal of P1 is still unchecked.",
  "items": [
   {"id": "start_open", "t": "The window opens to the opening screen", "isnew": 0,
    "d": "Double-click the exe above (or the release one, if you have downloaded it).",
    "e": "A window with the opening screen. The first card is \"Start something new\". Tell me how long it took "
         "from double-click to a drawn window, roughly: under 5 seconds, 5 to 15, longer."},
   {"id": "start_profile", "t": "The hardware profile was written on the first start", "isnew": 1,
    "d": "Open <code>%LOCALAPPDATA%\\MakerCAD\\profile.toml</code> in Notepad. (To see it written again, close "
         "MakerCAD, delete that file, start MakerCAD, wait 20 seconds.)",
    "e": "Comment lines say \"dx12: ok Intel(R) HD Graphics 530\" and \"gl: ok Intel(R) HD Graphics 530\", then "
         "\"4 cores, 7885 MB RAM\", then [profile] with class = \"potato\" and backend = \"dx12\". This is what I "
         "saw on this PC at about 21:00. The window must not freeze while the probe runs."},
  ]},
 {"n": "01", "t": "Start something new (the Quest)", "time": "~15 min",
  "note": "The first card on the opening screen. Nine questions, every one skippable; the right-hand card "
          "\"What your answers set up\" changes as you answer. The questions, the reading of your words and the "
          "scale guess are the Crew's modules; I drew the screen. I checked it with screenshots and 91 unit "
          "tests, not by using it for a real project, so this is the first real use.",
  "items": [
   {"id": "q_rail", "t": "The nine steps are on the left", "isnew": 0,
    "d": "Click <b>Start something new</b>.",
    "e": "A rail down the left: Name it, What to make, Where it fits, Your tools, Your materials, When you need "
         "it, Budget, What inspires you, Your workspace. The first is highlighted. Top right: \"Skip to "
         "designing\" and \"Cancel\"."},
   {"id": "q_goal", "t": "Your words set up environments, with a reason each", "isnew": 0,
    "d": "Name it: <b>Alcove dresser</b>. Next. What to make: <b>a dresser for the alcove</b>. Next.",
    "e": "\"What your answers set up\" lists Solid, Sketch and Stock & Cut, each followed by \"because you said "
         "\u201cdresser\u201d\". If it says something that does not follow from your words, that is a bug: write it down."},
   {"id": "q_fit", "t": "A typed size sets the scale class", "isnew": 0,
    "d": "Where it fits: choose the option that lets you type sizes and enter <b>1500 mm</b> (or "
         "<b>60 in</b> if the window is in inches).",
    "e": "Scale reads \"Furniture, its longest size is 1500 mm.\" with a line about a floor patch and Sancho at "
         "true size, grid 10 mm. Try <b>6 in</b> instead: it should become Bench."},
   {"id": "q_skip", "t": "Skipping is allowed and visible", "isnew": 0,
    "d": "On any step press <b>Skip for now</b>.",
    "e": "The rail shows a dash for that step instead of a green tick, and you move on. At the end the "
         "project says how many questions are still open (it is a number in the notice after you make the "
         "project)."},
   {"id": "q_date", "t": "A date that is not a date is refused", "isnew": 0,
    "d": "When you need it: type <b>soon</b> and press Next. Then type <b>2026-12-24</b>.",
    "e": "\"soon\" stays on the step with a message that it is not a date; 2026-12-24 moves on. Budget: "
         "\"150\" is accepted, \"lots\" is refused."},
  ]},
 {"n": "02", "t": "Use my workshop, and what stays in the project (new)", "time": "~10 min",
  "note": "Your rule (decision D30): tools and materials are <b>global</b> (your workshop, for every project) "
          "or <b>local</b> (this project only, say a board bought for the task). The workshop is a SQLite file, "
          "<code>%LOCALAPPDATA%\\MakerCAD\\library.db</code>: one per person, "
          "not per project. Opening the Quest makes an empty one if there is none. To try it without touching "
          "that file, open a Command Prompt in the repo folder and run:<br>"
          "<code>python tools\\make_sample_library.py %TEMP%\\sample-library.db</code><br>"
          "<code>set MAKERCAD_LIBRARY=%TEMP%\\sample-library.db</code><br>"
          "<code>third_party\\solidcraft\\target\\debug\\makercad.exe</code><br>"
          "The sample holds 14 tools (10 owned, 2 shared, 1 wishlist, 1 at a makerspace) and 3 material lines. "
          "Two of the tools are only there some days (D31, your multimeter-on-weekends example): the "
          "soldering iron on weekends (\"at Sam's\") and the track saw on Tuesdays and Thursdays. "
          "(There is no multimeter in the tool list and, since you said a small list of what most people have is "
          "enough, I am not adding electronics gear; I used the soldering iron.) "
          "Do not run the script against <code>library.db</code> itself: it replaces the file.",
  "items": [
   {"id": "w_tools", "t": "The tools step offers your workshop", "isnew": 1,
    "d": "Start something new, go to <b>Your tools</b> (step 4).",
    "e": "\"Your workshop has 12 tools I know (2 more are on a wishlist, borrowed, or not known to me yet).\" "
         "and a button <b>Use my workshop</b>. (12 = the 10 owned plus the shared router and soldering iron; "
         "the wishlist drill press and the makerspace track saw are the 2.)"},
   {"id": "w_use", "t": "Using it lists the tools once", "isnew": 1,
    "d": "Press <b>Use my workshop</b>, then press it again if it is still there.",
    "e": "Chips for the tools appear (Tape measure, Square, Clamps, Hand saw, Hammer, Screwdrivers, Drill "
         "driver, Jigsaw, Circular saw, Sander, Router, and <b>Soldering iron · weekends (at Sam's)</b>: a tool "
         "that is only there some days says which on its chip). Clicking a chip removes it. Nothing is added twice. "
         "The button disappears once every workshop tool is listed."},
   {"id": "w_days_global", "t": "A workshop tool's days show, and are changed in My workshop", "isnew": 1,
    "d": "Hover the <b>Soldering iron · weekends (at Sam's)</b> chip, then click it.",
    "e": "\"From your workshop, available weekends (at Sam's). Click to leave it out of this project; change "
         "the days in My workshop.\" The chip is filled (a workshop tool), and clicking it just leaves the tool "
         "out of this project; there is no way to change its days here. Why: a project cannot change your "
         "workshop (D30), so a workshop tool's days are edited only in My workshop. Is that the right "
         "split, or should a project be able to say \"for this project I only have it on Saturdays\"?"},
   {"id": "w_days_local", "t": "A tool you type in has its own days", "isnew": 1,
    "d": "Type <b>band saw</b> and Add. Click its chip (the one with the outline). Press "
         "<b>Weekends</b>, then untick <b>Sun</b>, type <b>Dad's garage</b> in the box, and press <b>Save</b>.",
    "e": "A window \"When can you use Band saw?\" with Any day, Weekends, Weekdays and seven day buttons "
         "(ticked ones coral), a box, a line \"Available: ...\", and Save, Cancel and \"Take it out of this "
         "project\". After Weekends then Sun unticked it reads \"Saturdays\", and after the note \"Saturdays "
         "(Dad's garage)\". After Save the chip reads \"Band saw · Saturdays (Dad's garage)\". Cancel keeps "
         "nothing; \"Take it out\" removes the chip. Try unticking the only ticked day: it stays ticked and "
         "says a tool needs at least one day. (Clicking an outlined chip used to remove it at once; now it "
         "opens this window first. Is the extra click a nuisance?)"},
   {"id": "w_materials", "t": "The materials step offers the material lines", "isnew": 1,
    "d": "Go to <b>Your materials</b> (step 5) and press <b>Use my workshop</b>.",
    "e": "\"Your workshop has 3 material lines on hand.\" Then three lines: 15 pine boards 38 \u00d7 89 mm and "
         "914 mm long; 1 plywood sheet 19 mm thick; 2 oak boards. Each is shown with the words it came "
         "from (\"15 pine 2x4s, 3 foot\", \"half sheet 3/4 ply\", \"2 oak 1x6s, 4 foot\") and a \u00d7 to remove it. "
         "The exact sizes shown for the plywood and oak are what the file holds; tell me if any looks wrong."},
   {"id": "w_local", "t": "What you type stays in the project (global and local)", "isnew": 1,
    "d": "On the tools step type <b>band saw</b> and Add; on the materials step type <b>2 yards 500D "
         "Cordura</b> and Add. Look at the chips and rows, then make the project (Skip to designing is fine).",
    "e": "The band saw chip has a coral outline (the workshop's tools are filled); hovering it says \"Only "
         "for this project. Click to say when you can use it, or to take it out.\"; under the chips: \"... the 1 with an outline are only for "
         "this project. Keep your workshop up to date in My workshop.\" The Cordura row says \"only for this "
         "project\" in coral; the other rows say \"your workshop\". After making the project the notice "
         "ends \"1 material line is only for this project; what is left can be offered to your workshop when "
         "it is finished.\" Your workshop must NOT have gained the band saw or the Cordura: check in My "
         "workshop (next section). This is your D30 rule; I checked it with a unit test and screenshots only."},
   {"id": "m_system", "t": "The sizes step asks which system the project measures in", "isnew": 1,
    "d": "Start something new, go to step 3 (<b>Where does it need to fit?</b>). Look at the top, then press "
         "<b>Imperial (inches)</b>, then <b>Metric (millimetres)</b> again.",
    "e": "A line \"This project measures in\" with two choices, Metric (millimetres) and Imperial (inches); the one "
         "in use is highlighted, and it starts from the app's setting (millimetres unless you changed it). Under "
         "it: \"Type a size any way you like (millimetres, inches, feet, fractions); it is turned into this when "
         "you press Enter.\" This is your D34: the document has a measuring system and everything typed into it "
         "is turned into it."},
   {"id": "m_convert", "t": "A size typed another way is shown in the project's system, then kept in it", "isnew": 1,
    "d": "Pick <b>I'll type sizes</b>. In <b>Its longest size</b> type <b>5 ft 6 in</b> and do not press "
         "Enter yet. Look under the box. Then press Enter. Try the same with <b>120 cm</b>, <b>1.2 m</b>, "
         "<b>5'-6\"</b>, <b>3 1/2 in</b> and a bare <b>1200</b>.",
    "e": "In a metric project, before Enter: \"That is 1676.4 mm. Press Enter to keep it.\" and a line saying "
         "which scale that makes. After Enter the box itself reads <b>1676.4 mm</b> and the line says \"Kept as "
         "1676.4 mm.\" (120 cm becomes 1200 mm, 1.2 m the same, 3 1/2 in becomes 88.9 mm, and a bare 1200 is "
         "1200 mm because a bare number is in the project's own unit.) Clicking away from the box confirms it too."},
   {"id": "m_imperial", "t": "In an imperial project the same typing comes out in inches and fractions", "isnew": 1,
    "d": "Press <b>Imperial (inches)</b>. The size box should change by itself. Then type <b>10 mm</b> and read "
         "the line under the box. Press Enter. Press <b>Metric (millimetres)</b> again.",
    "e": "Switching system rewrites the size already there as the same length (1676.4 mm becomes 66\"). For 10 mm: "
         "\"That is 25/64\", to the nearest 1/64\". Press Enter to keep it.\" After Enter the box reads 25/64\" "
         "(which is 9.92 mm, not 10: what you see is what is kept, and I say so before it happens). Back in "
         "metric the box reads 9.92 mm. Is 1/64\" the right precision? (Your call, last section.)"},
   {"id": "m_unreadable", "t": "What cannot be read says why", "isnew": 1,
    "d": "In the size box type <b>a lot</b>, then <b>3/0</b>, then <b>-4</b>, pressing Enter after each.",
    "e": "\"I can't read that as a length yet (try 1200 mm, 120 cm or 1.2 m).\" (inches: \"try 48 in, 4 ft or 3 "
         "1/2\"\"), then \"A fraction can't have 0 below the line.\", then \"A length can't be below zero.\" "
         "The text you typed stays as you typed it; nothing is changed or lost."},
   {"id": "m_status", "t": "The project keeps its system, and the status bar changes it", "isnew": 1,
    "d": "Make the project (Skip to designing is fine). Look at the bottom right: hover <b>in</b> and <b>mm</b>, "
         "press <b>in</b>, then save with Ctrl+S, go to the opening screen and open the project again. Open "
         "<b>Preferences</b> (Settings) and look at \"Sizes in\".",
    "e": "The hover says \"This project measures in millimetres\" (or inches), and pressing <b>in</b> changes "
         "this project, not your default: Sancho's height in the bar becomes feet and inches. After saving and "
         "opening again it is still inches. Preferences has two rows, \"This project\" (only while one is open) "
         "and \"New projects\" (the default the Quest starts from). Everything underneath is still millimetres, "
         "so changing system later changes how lengths are written and read, not the lengths."},
  ]},
 {"n": "03", "t": "My workshop: the walkthrough (new)", "time": "~10 min",
  "note": "The global tools and materials get their own three steps so they are easy to keep up to date. "
          "Use the same sample-library setup as above (so the real <code>library.db</code> is untouched). "
          "Layout is plain for now: tell me what is awkward.",
  "items": [
   {"id": "ws_open", "t": "Open it from the opening screen", "isnew": 1,
    "d": "Go Home. In the My workshop card press <b>Open workshop</b>. (Or Ctrl+K and type <b>workshop</b>.)",
    "e": "A screen titled \"My workshop\" with three steps on the left (Your tools, Your materials, All set), "
         "\"Done\" top right, and \"What tools do you have?\" with the kit buttons (Basic household, Starter "
         "woodshop, ...), a box to type a tool, and 14 tools each with a dropdown (\"I own it\", the router "
         "\"Shared with me\", the drill press \"Wishlist\"), a days button (\"any day\" in grey for most; "
         "\"weekends (at Sam's)\" on the soldering iron; \"Tue and Thu (makerspace open evenings)\" on the track "
         "saw) and a ×."},
   {"id": "ws_when", "t": "Say which days you can use a tool", "isnew": 1,
    "d": "On the soldering iron press its <b>weekends</b> button. Press <b>Any day</b>, then <b>Weekdays</b>, "
         "then tick <b>Sat</b>, put <b>when I visit Sam</b> in the box and press <b>Save</b>.",
    "e": "A window \"When can you use Soldering iron?\" with a line \"Available: ...\" that follows every click "
         "(\"any day\", then \"weekdays\", then \"every day except Sun\" once Sat is ticked, and with the note: "
         "\"every day except Sun (when I visit Sam)\"). After Save the button on its row says the same words and the screen says \"Soldering iron: "
         "...\". Close and reopen My workshop: it is still there (it is kept in the library file). On the last "
         "step the sentence ends \"N tools are only there on some days.\""},
   {"id": "ws_add", "t": "Add tools by name and by kit", "isnew": 1,
    "d": "Type <b>band saw</b> and press Add; type it again; then press the <b>Laser</b> kit twice.",
    "e": "\"Added Band saw.\"; the second time \"You already have Band saw.\"; the kit says how many tools it "
         "added (\"Laser: N tools added...\") and the second press says \"you already have all of these.\""},
   {"id": "ws_access", "t": "Say how you have a tool, or take it out", "isnew": 1,
    "d": "Change the Jigsaw's dropdown to <b>Wishlist</b>. Press the × on the band saw.",
    "e": "The jigsaw stays in the list as Wishlist. The band saw disappears. Then start a new project (Home, "
         "Start something new): its tools step now counts the jigsaw among the ones \"on a wishlist\", and Use "
         "my workshop leaves it out."},
   {"id": "ws_materials", "t": "Materials: add, count, used up, take out", "isnew": 1,
    "d": "On Your materials type <b>6 pine 1x4s, 4 foot</b> and Add. Press its − twice, then untick \"on "
         "hand\". Then press its ×.",
    "e": "A line \"6 × pine board 19 × 89 mm 1219 mm long\" with 6 beside it; after two − it shows 4; "
         "unticked, it stops being offered by a new Quest's \"Use my workshop\"; × removes it. The count "
         "never goes below 0."},
   {"id": "ws_summary", "t": "All set says what you have and what changed", "isnew": 1,
    "d": "Press Next to reach <b>All set</b>.",
    "e": "One sentence: \"Your workshop has N tools (M you have at hand) and K material lines (J on hand)\" "
         "followed, if anything changed on this visit, by \", X more tools than when you opened this\" and "
         "the same for material lines. Press <b>Done</b>: you are back where you opened it from."},
   {"id": "ws_home", "t": "The opening screen counts the workshop", "isnew": 1,
    "d": "Look at the My workshop card on the opening screen.",
    "e": "Its second line reads \"N tools · K material lines · this PC: ...\" instead of \"No tools or "
         "materials yet\". (With --demo it still shows the sample's 41 tools.)"},
  ]},
 {"n": "04", "t": "Making the project", "time": "~5 min",
  "note": "Confirming the last step, or \"Skip to designing\" at any time, makes a project file. I have not "
          "looked at the Design workspace that opens on the new, empty design, so I do not know what it "
          "looks like.",
  "items": [
   {"id": "m_make", "t": "A project file is made and saved", "isnew": 0,
    "d": "On step 9, confirm (or press <b>Skip to designing</b> from any step).",
    "e": "The Design workspace opens with a message \"Alcove dresser is made and saved as Alcove dresser.maker.\" "
         "and a count of open questions. The file is in <code>Documents\\MakerCAD\\</code>. Making a second "
         "project with the same name gives \"Alcove dresser 2.maker\", never replaces the first."},
   {"id": "m_empty", "t": "The empty design looks right", "isnew": 1,
    "d": "Look at what opened.",
    "e": "Say what you see. I expect the stage for the scale class (a floor patch for Furniture) and Sancho "
         "beside it, with nothing modelled. I did not check this myself."},
   {"id": "m_reopen", "t": "Keep going: the project comes back", "isnew": 0,
    "d": "Go Home (the house icon, top left). Look at the list; open the project you just made.",
    "e": "The opening screen lists it newest first with \"saved today\". Opening it gives back the empty design "
         "and its name."},
  ]},
 {"n": "05", "t": "Finishing the project: what is left over (new)", "time": "~10 min",
  "note": "Your rule: any materials left from a local project can be added to the workshop at the end. "
          "Today that is a command, because a project does not yet remember being finished: press Ctrl+K and "
          "type <b>finish</b>. To see it without making a project, start the exe with <code>--finish</code> "
          "(a sample project with three materials and two tools of its own). Tools are offered too (your "
          "answer on 2026-11-23), each with how you have it now and the days it came with.",
  "items": [
   {"id": "fin_open", "t": "The offer lists the project's own tools and materials", "isnew": 1,
    "d": "With the project you made in the last section open (one with the band saw and the Cordura in it), "
         "Ctrl+K, <b>Finish the project</b>.",
    "e": "A window \"Finish the project\": \"The project is finished. Anything left over?\", then \"Tools you got "
         "for this project\" with the band saw, then \"Materials with some left\" with the Cordura line (what "
         "came from your workshop is NOT listed). Each has a tick box; a tool has a dropdown for how you have "
         "it (greyed until ticked), a material a \"left:\" number (greyed until ticked). The button says "
         "\"Add to my workshop\", greyed until one is ticked, and \"Not now\". With --finish: two tools "
         "(Soldering iron, \"available: weekends (at Sam's)\", ticked and \"Shared with me\"; Band saw), three "
         "material lines (Cordura ticked with left: 1), and the button says \"Add 2 to my workshop\"."},
   {"id": "fin_add", "t": "Ticking and adding hands them to the workshop", "isnew": 1,
    "d": "Tick the band saw and the Cordura, set left to <b>1</b> (it starts at the number you listed), "
         "press <b>Add to my workshop</b>. Then open My workshop, Your tools and Your materials.",
    "e": "A message \"1 material line and 1 tool added to your workshop.\" followed by the save message. In "
         "My workshop a Cordura line is there with <b>1</b> beside it (the number left, not the number you "
         "bought) and a Band saw with \"I own it\" and \"any day\". Ctrl+K, Finish the project again: \"This "
         "project has no materials or tools of its own left to offer.\""},
   {"id": "fin_tool_days", "t": "A tool keeps its days when it joins the workshop", "isnew": 1,
    "d": "Start the exe with <code>--finish</code> (sample library set as before), press <b>Add</b> with the "
         "soldering iron ticked, then open My workshop, Your tools.",
    "e": "The message says \"1 material line added to your workshop (1 tool was already there)\" (your sample "
         "library already has a soldering iron, so it is not added a second time and keeps the days it had). "
         "To see a tool come in with its days: start the exe plainly, open My workshop and remove the "
         "soldering iron, close it, then start it with <code>--finish</code> again (do not rerun "
         "make_sample_library: that puts the iron back) and press Add; the iron returns as \"Shared with "
         "me\" with \"weekends (at Sam's)\"."},
   {"id": "fin_notnow", "t": "Not now changes nothing", "isnew": 1,
    "d": "Open the offer on another project, tick something, press <b>Not now</b>.",
    "e": "The window closes; My workshop has nothing new; the offer is still there next time."},
   {"id": "fin_none", "t": "A project with nothing of its own says so", "isnew": 1,
    "d": "On an example you opened but never made a project from, Ctrl+K, Finish the project.",
    "e": "\"Open or make a project first: finishing is something a project does.\""},
  ]},
 {"n": "06", "t": "Sancho's Findings panel", "time": "~10 min",
  "note": "This panel draws the Crew's models: how findings are sorted and counted, and the \"Why?\" "
          "derivations. It shows <b>the golden dresser's findings</b>, not the design you have open: the "
          "window cannot yet read a design's parts, materials and loads into the facts the rules read (the "
          "Crew has built the geometry for that, <code>design_facts</code>, and it is not connected). The "
          "golden dresser is the P2 exit fixture: a three-drawer chest that the four rule packs leave alone, "
          "with five faults seeded.",
  "items": [
   {"id": "f_open", "t": "Open the panel", "isnew": 1,
    "d": "Open the sample plate (Examples on the opening screen), press <b>Ctrl+K</b>, type <b>findings</b>, "
         "choose <b>Show the findings</b>. (Or start the exe with <code>--findings dresser</code>.)",
    "e": "A panel on the right: \"Sancho's findings\", \"5 things to look at, 1 serious\", and a small grey "
         "note that this is the golden dresser, a demonstration and not your design. Under \"Critical (1)\": "
         "\"screw_1 pokes out\" with a red dot. Under \"Warning (4)\": the chest tipping over, the top "
         "sagging, a screw near an edge, and a missing tool, each with a yellow dot."},
   {"id": "f_sag", "t": "The sag has real numbers", "isnew": 1,
    "d": "Scroll to \"carcase_top sags about 2.2 mm\".",
    "e": "Its reason reads: over 864 mm, 12 mm of wood.softwood.pine carrying 20 kg per metre sags about 2.2 mm; "
         "more than 1.7 mm shows. Those numbers are handbook-range values I marked \"not yet sourced\" "
         "(pine 10,000 MPa, 2 mm of sag per metre of span allowed). Do they read as a real warning or as a "
         "made-up one?"},
   {"id": "f_why", "t": "Why? opens the derivation", "isnew": 1,
    "d": "On \"screw_1 pokes out\" press <b>Why?</b>.",
    "e": "A box under it titled \"Screw through the stock\": fastener screw_1, length_mm 40, stock_mm 36, then "
         "a rule saying a screw longer than the stock passes through and must stop inside it. The button "
         "changes to \"Hide why\". The row labels are still the raw names (length_mm): is that too "
         "technical for you?"},
   {"id": "f_dismiss", "t": "Dismiss and bring it back", "isnew": 1,
    "d": "Press <b>Dismiss</b> on \"screw_1 pokes out\", then <b>Bring it back</b>.",
    "e": "After dismissing: the headline reads \"4 things to look at\" (no \"serious\" any more), the card "
         "greys and says \"dismissed\". Tick \"Hide the ones I dismissed\" and it disappears; untick and it "
         "returns. Bringing it back restores \"5 things to look at, 1 serious\"."},
   {"id": "f_group", "t": "Group by", "isnew": 1,
    "d": "Press <b>Kind</b>, then <b>Place</b>, then <b>How serious</b>.",
    "e": "The same five findings under different headings each time (kinds in words; places named by "
         "their ids, such as chest, screw_1 and Step 4). Nothing is lost or doubled."},
   {"id": "f_sound", "t": "The clean chest says nothing", "isnew": 1,
    "d": "Close MakerCAD and start it with <code>--findings sound</code> (from a Command Prompt: "
         "<code>makercad.exe --findings sound</code>).",
    "e": "\"Nothing to look at\" and \"Nothing to look at. Sancho has no findings for this design.\" This is the "
         "\"0 false flags\" half of the P2 exit test."},
   {"id": "f_demo", "t": "Is the demonstration note clear enough?", "isnew": 1,
    "d": "Read the grey note at the top of the panel as if you had not read this page.",
    "e": "It says: \"Shown for the golden dresser with its five seeded faults: a demonstration, not your design "
         "(the window cannot read a design's facts yet).\" Would you have been misled into thinking these "
         "are findings about your design? If so, say how you would word it."},
  ]},
 {"n": "07", "t": "Older things, one line each", "time": "~5 min",
  "note": "These are from earlier days. If you already checked them, mark n/a.",
  "items": [
   {"id": "o_save", "t": "Save with Ctrl+S", "isnew": 0,
    "d": "Open the sample plate, press <b>Ctrl+S</b>.",
    "e": "A notice that it was saved, and a .maker file in Documents\\MakerCAD. A second save keeps the "
         "previous one beside it as a .backup file."},
   {"id": "o_sancho", "t": "Sancho beside the design", "isnew": 0,
    "d": "On a design, look at the top bar toggle \"Sancho 1.68 m\" and press it.",
    "e": "A figure about 1.68 m tall stands to the design's right on the floor patch; the toggle hides and "
         "shows him. The figure is a free placeholder (BodyKun); you are designing the real Sancho."},
   {"id": "o_cloud", "t": "The cloud checks are green", "isnew": 0,
    "d": "Open github.com/Messyhairkris/makercad/actions and look at the newest runs.",
    "e": "Runs #24 to #31 finished green (both the gates and the Windows build). The newest run is the "
         "measuring-system work, and I had not seen it finish when I wrote this, so look at it first. Every run builds a test "
         "<code>makercad.exe</code> you can download."},
  ]},
 {"n": "08", "t": "What the Crew built today", "time": "~5 min",
  "note": "Gordon wrote contracts, reference code and hidden tests as a PA; Deepak, Hoss, Kimi and Qwen wrote "
          "the modules and a second set of tests; I checked each with mutation testing (seeded bugs the "
          "tests must catch) and wrote the missing tests by hand where they did not. Everything is in "
          "<code>STATUS.md</code>, one row per module.",
  "items": [
   {"id": "c_status", "t": "Skim the STATUS rows", "isnew": 1,
    "d": "Open <code>STATUS.md</code> in the repo and read the bold rows for the mk-assist, mk-sancho, "
         "mk-log, mk-inventory and mk-project modules.",
    "e": "Each row says what the module does in plain words, who wrote what, how many requests it took, and "
         "how many seeded bugs the tests caught. Modules added since you last looked include the golden "
         "dresser (five faults, no false flags), design_facts (boxes to the facts the structure rules read), "
         "why, voice, toolbox, panel, intent_card, stage, drift and observations. Tell me if a row is "
         "written in words you do not follow."},
  ]},
]

CALL = {"n": "09", "t": "Your call", "time": "~5 min",
        "note": "Decisions only you can make. The numbers are real; I have not tucked a recommendation "
                "into the question. Already settled by your direction (D30 to D34): a part's material is picked "
                "from the project's list of materials, which is the workshop's global ones plus the project's "
                "own local ones (I will build that when the Findings panel is wired to your real design); local "
                "tools are offered at the end of a project too; a tool can have days it is available, and when "
                "a plan uses that it shows it on the plan, never through Sancho (D32); the tool list stays small "
                "and made of what most people own (D33); and a project measures in one system, with everything "
                "typed into it turned into that system (D34).",
        "items": [
   {"id": "y_precision", "t": "Is 1/64 of an inch (and 0.01 mm) the right precision?", "isnew": 1,
    "d": "An imperial project keeps lengths to 1/64\", a metric one to a hundredth of a millimetre. The rounding "
         "is said before you confirm it.",
    "e": "1/64\" is about 0.4 mm, so a length typed in millimetres can move by up to 0.2 mm when it is kept "
         "in an imperial project (10 mm becomes 9.92 mm). Options: (a) keep it; (b) each project chooses "
         "1/16, 1/32, 1/64 or decimal inches; (c) imperial keeps decimals when the typed length is not a "
         "fraction. Which?"},
   {"id": "y_findings_name", "t": "Should the Findings panel still carry Sancho's name?", "isnew": 1,
    "d": "Open the Findings panel (Ctrl+K, \"Show the findings\") and look at the Quest's review card, which has "
         "a box headed \"Sancho noticed\".",
    "e": "You said Sancho handling alert-type messages would hurt him as a character (D32). A findings panel is "
         "an alert channel by nature: it lists what is wrong, worst first. Today it is called Sancho's Findings "
         "panel and the Quest has the \"Sancho noticed\" hints. Options: (a) rename the panel plainly "
         "\"Findings\" and keep Sancho only as a figure who reacts; (b) keep his name on the panel and the "
         "hints, as friendly framing; (c) rename the panel but keep the Quest's hints in his voice, since "
         "they are suggestions, not warnings. I have changed nothing here. Which?"},
   {"id": "y_instruments", "t": "Do measuring tools count as tools?", "isnew": 1,
    "d": "You said a tool is anything that can modify or refine a material. A tape measure, calipers or a "
         "level measure; they do not change the material. The phone camera, LiDAR and the projector do not "
         "either.",
    "e": "I left them in because a plan needs to know you can measure. Options: (a) keep them in the tool list; "
         "(b) a separate short \"Measuring\" list next to the tools, in the same walkthrough step. Which? "
         "And pliers, an adjustable wrench and a pencil are not in the list yet: add them to the Basic household "
         "kit? (It means a new revision of the tool table; I would not do it without your yes.)"},
   {"id": "y_finished", "t": "Should a project know it is finished?", "isnew": 1,
    "d": "Today finishing is a command (Ctrl+K, Finish the project) that you run yourself.",
    "e": "Options: (a) keep the command only; (b) a Finish button on the project (in Build or Share) that "
         "marks it finished in the .maker file, opens this offer, and then shows it as done on the opening "
         "screen. (b) means a new field in the project file and a finished state to display. Which?"},
   {"id": "y_numbers", "t": "Sourced numbers, or marked handbook values?", "isnew": 1,
    "d": "The structure rules need each material's stiffness and limits, and a sag allowance.",
    "e": "Now: pine 10,000 MPa, plywood 6,000 MPa, 2 mm of sag per metre of span, marked \"not yet "
         "sourced\". I can have Gordon research real figures with cited sources (he is the most accurate "
         "at quoting them), and I would check each against its source. Until then, should the panel show "
         "warnings built on unsourced numbers at all, or stay silent for any material without sourced values?"},
   {"id": "y_relevant", "t": "\"You have 41 tools; these 9 matter for a dresser\"", "isnew": 1,
    "d": "Doc 05 section 5.3 promises the Quest narrows your workshop to what the project needs.",
    "e": "Today \"Use my workshop\" copies every tool you own into the project. Narrowing needs a table of "
         "which tools each kind of project needs; the Crew can write it, and it would be a guess for any "
         "kind of project the table lacks. Do it now, or leave \"all my tools\" until a plan exists to say "
         "which steps need which tool?"},
 ]}

ITEMS.append(CALL)

html = TEMPLATE.read_text(encoding="utf-8")
html = html.replace("<title>PROJECT Test Pass \u2014 local</title>",
                    "<title>MakerCAD Test Pass \u2014 Quest, workshop, Findings \u2014 local</title>")
html = html.replace("<span>PROJECT \u00b7 local copy \u00b7 nothing leaves this machine</span>",
                    "<span>MakerCAD \u00b7 2026-11-22 \u00b7 local copy \u00b7 nothing leaves this machine</span>")
html = html.replace("<h1>Test Pass</h1>", "<h1>Quest, workshop and Findings</h1>")
html = html.replace(
    "Saves to this browser as you type \u2014 no network, no\n  reconnecting, nothing to lose. <b>Export</b> when you want to send results\n  back.",
    "Written for your look at noon. It covers what changed since your last pass: the Quest, the workshop "
    "(global and local tools and materials, your rule of 2026-11-23), the Findings panel, and the Crew's "
    "modules. Saves to this browser as you type; <b>Export "
    "findings</b> gives text to paste back into the conversation. Points about things I could not check "
    "myself say so.")
start = html.index("const ITEMS = [")
end = html.index("/* Final notes prompts")
html = html[:start] + "const ITEMS = " + json.dumps(ITEMS, indent=1, ensure_ascii=False) + ";\n\n" + html[end:]
html = html.replace('const KEY = "REPLACE-slug-test-pass-v1";', 'const KEY = "makercad-p2-quest-workshop-findings-v2";')
html = html.replace('"PROJECT test pass \u2014 "', '"MakerCAD test pass \u2014 "')
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(html, encoding="utf-8", newline="\n")
n = sum(len(s["items"]) for s in ITEMS)
print(f"{OUT.name}: {len(ITEMS)} sections, {n} points, {len(html)} chars")
