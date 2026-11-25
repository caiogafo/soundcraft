# Pro Tools shortcut audit

Reference: [Avid Pro Tools Shortcuts Guide 2025.10](https://resources.avid.com/SupportFiles/PT/Pro_Tools_Shortcuts_2025.10.pdf).

This is the scope for the Soundcraft compatibility pass. A shortcut is only marked
implemented when the corresponding action exists and is exercised by the Soundcraft UI.

## Scope and status

| Pro Tools guide section | Soundcraft status | Notes |
| --- | --- | --- |
| Global parameter, toggle, fine-adjust and multi-select shortcuts | Partial | Track, clip, edit-selection and mixer cases exist; plugin, I/O and hardware-specific cases still need their equivalent actions. |
| Commands Keyboard Focus | Partial | Edit commands are routed in `crates/ui-egui/src/shortcuts.rs`; `-`, `\\`, `Q/W`, `Y/U/I`, `H/J/K`, `6–9` and the existing edit families now use Soundcraft actions. Some Pro Tools-specific view semantics still need reconciliation. |
| Edit and Mix transport, editing, zoom and selection | Partial | The current command registry covers many standard actions; the audit must still reconcile every Pro Tools modifier family and context. |
| MIDI note editing | Partial | Piano-roll arrow editing and the MIDI Operations window exist; the complete note, velocity, controller and Step Input focus families are not yet reconciled. |
| Clip gain, clip effects, playlists and automation | Partial | Several engine commands exist; shortcut coverage and context ownership are incomplete. |
| Memory locations and markers | Partial | Recall, add, rename and delete paths exist; every Pro Tools marker/navigation chord is not yet mapped. |
| Add New Tracks, Clip List, Track List and Groups List | Partial | UI and commands exist for several operations; focus/navigation families still need mapping. |
| Dialog-window shortcuts | Partial | Some dialogs have keyboard handling; this requires a per-dialog pass. |
| Workspace shortcuts | Not applicable until equivalent workspace actions exist | Soundcraft does not currently expose the full Pro Tools Workspace/Soundbase browser model. |
| Deck control and Satellite commands | Not applicable until transport/hardware support exists | These depend on external deck or satellite integrations not present in Soundcraft. |
| Right-click shortcuts | Partial | Context menus exist in some windows, but Pro Tools' complete context-click matrix is not a single keyboard shortcut map. |

## Current implementation checkpoints

- Plain Edit key families are dispatched without requiring the `a-z` toggle.
- `B` invokes Edit > Separate when an Edit selection is available.
- Keyboard Focus switching now covers `Cmd/Ctrl+Alt+1` (Commands), `+2` (Clip List) and `+3` (Groups List), using the corresponding SoundCraft focus state.
- Pro Tools Search now accepts `Cmd+Ctrl+S` on macOS and `Ctrl+Shift+S` on Windows/Linux; `Ctrl/Cmd+S` remains session Save.
- MIDI Operations coverage now includes `Alt+Q/T/Y/P` (Option on macOS) to open or close the existing MIDI Operations window on Quantize, Transpose, Split Notes and Duration; the remaining gap is the numeric-keypad Step Input focus model.
- Arrow navigation, transport shortcuts, marker shortcuts and save/export shortcuts are covered by the existing UI shortcut dispatcher.
- The Commands Keyboard Focus batch now covers track-view toggle (`-`: waveform/volume and notes/regions), timeline-selection edge display in the main counter (`Q/W`), clip start/sync/end placement (`Y/U/I`, `H/J/K`) and earlier-by-grid nudge (`\\`).
- The same family now covers edit-boundary audition: `6/8` play to the edit start/end and `7/9` play from the edit start/end.
- Transport conflicts audited against the official guide: `Ctrl+Space` pauses, `Alt+Space` toggles Transport Online, and `Ctrl+J`/`Cmd+J` toggles Transport Online according to the platform modifier model.
- Marker entry now follows the guide's modifier split: plain `Enter` creates a marker and opens name entry, while `Cmd/Ctrl+Enter` creates a marker without opening the dialog.
- Playlist/navigation coverage now includes `Shift+Up/Down` for the previous/next alternate playlist and `Shift+T` for Track/Edit selection linking.
- Auditioning coverage now uses the configured Pre/Post-Roll values: `Alt+Left/Right` auditions before an edge, `Ctrl/Cmd+Left/Right` after it, and the combined chord covers the edge through both rolls.
- Edit-selection display coverage now includes `Shift+F` for containing folders and `Shift+U` for the Marker Track lane.
- Marker option coverage now includes `Shift+Y` for Marker Follows Edit.
- Marker display coverage now includes the three-modifier `R`/`F` chords for Marker Ruler Lines and Marker Folder Lines.
- Vertical-navigation conflict coverage now prioritizes MIDI Record Enable movement on `Ctrl/Cmd+Up/Down` when a MIDI/Instrument track is armed; otherwise the chord remains Track Height.
- Target Playlist coverage now uses the existing target-playlist commands: `Shift+Right` shows the target, `Shift+Left` toggles the recent playlist, and the backslash creation/duplication chords route to the selected-track playlist model. The engine cannot distinguish every Pro Tools “all tracks” scope on keyboards where Control and Command are collapsed by the host.
- Zoom/Edit Tool coverage now includes `Ctrl/Cmd+E` for Zoom Toggle and the primary-modifier tool family (`1` Zoom, `2` Trim, `3` Selector, `+` Grabber, `5` Scrubber, `6` Pencil, `7` Smart Tool).
- Clip Gain coverage now includes the line/info toggles and Clear/Cut/Copy chords; adding a breakpoint at the current edit location remains unimplemented because the engine has no equivalent point-insertion command.

## Completion rule

The compatibility pass is complete only when every shortcut in the reference guide is either:

1. mapped to a verified Soundcraft action; or
2. listed here with the exact missing Soundcraft capability that prevents an equivalent implementation.

## Known non-equivalents from the guide

These are not omitted accidentally; the corresponding Soundcraft capability is absent or
currently exposed only as a disabled preference, so assigning the Pro Tools key would be
misleading:

| Guide area | Pro Tools shortcut family | SoundCraft blocker |
| --- | --- | --- |
| Numeric Keypad: Transport/Shuttle | `0–9`, keypad combinations, Shuttle Lock | The Preferences screen lists Classic/Transport/Shuttle, but the control is disabled and the input layer cannot distinguish keypad digits from the main number row. |
| Scrub/Jog/Shuttle | Control/Start-drag, Shuttle Lock numeric chords | No equivalent transport scrub/shuttle mode or host jog state is implemented. |
| Clip Effects | Option/Alt+6, Command/Control+Option+Control+B/X/C/S | No Clip Effects processing/preset model exists in the SoundCraft clip model. |
| Elastic Audio | Warp-marker add/move/delete and rendered/real-time toggle | No Elastic Audio event/warp-marker model exists. |
| MIDI Operations and Step Input | Numeric duration keys, panel navigation | SoundCraft has a MIDI Operations window and maps Quantize/Transpose/Split Notes/Duration actions, but it lacks Pro Tools' numeric-keypad Step Input focus model. |
| Workspace/Browser | Workspace search, browser navigation and drag/drop modifier matrix | No Workspace/Soundbase browser model; ordinary file drag/drop is implemented separately in the Edit window. |
| Deck/Satellite/ICON control surfaces | Deck, Satellite and hardware-specific modifier shortcuts | No external deck, satellite or ICON control-surface integration exists. |
| Custom Keyboard Shortcuts window | Setup > Keyboard Shortcuts, conflict resolver, presets and export | SoundCraft has a command registry, but no user-editable shortcut-assignment window/preset format equivalent to Pro Tools. |
| Marker target/name/comment shortcuts | `Shift+X`, `Shift+[ / ]`, `Shift+N`, marker-name/comment priority clicks | SoundCraft has marker recall/add/edit/delete and marker lanes, but no independent marker target selector or per-track marker-name/comment display-priority state. |
| Advanced automation editing | Automation trim/glide/write-on-stop, plugin automation dialogs and multi-playlist special paste | SoundCraft has volume/pan/send automation lanes, point editing and write/thin/copy-to-send commands, but no trim/glide/write-on-stop/plugin-automation state model for these shortcut families. |
| Numeric-keypad memory recall | `.` + number + `.` / numeric-keypad marker entry | The host exposes logical egui keys without a reliable keypad-vs-main-row distinction; SoundCraft therefore cannot safely assign these without stealing ordinary number-row shortcuts. |
