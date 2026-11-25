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
| MIDI note editing | Partial | Piano-roll note, velocity, selection and trim families are mapped; MIDI controller editing and the numeric-keypad Step Input focus model are not present. |
| Clip gain, clip effects, playlists and automation | Partial | Several engine commands exist; shortcut coverage and context ownership are incomplete. |
| Memory locations and markers | Partial | Recall, add, rename and delete paths exist; every Pro Tools marker/navigation chord is not yet mapped. |
| Add New Tracks, Clip List, Track List and Groups List | Partial | UI and commands exist for several operations; focus/navigation families still need mapping. |
| Dialog-window shortcuts | Partial | Some dialogs have keyboard handling; this requires a per-dialog pass. |
| Dashboard shortcuts | Not applicable until an equivalent dashboard exists | SoundCraft opens sessions directly and has no Pro Tools Dashboard/template/project browser with the guide's tab and option state. |
| Workspace shortcuts | Not applicable until equivalent workspace actions exist | Soundcraft does not currently expose the full Pro Tools Workspace/Soundbase browser model. |
| Deck control and Satellite commands | Not applicable until transport/hardware support exists | These depend on external deck or satellite integrations not present in Soundcraft. |
| Right-click shortcuts | Partial | Context menus exist in some windows, but Pro Tools' complete context-click matrix is not a single keyboard shortcut map. |

## Current implementation checkpoints

- Plain Edit key families are dispatched without requiring the `a-z` toggle.
- `B` invokes Edit > Separate when an Edit selection is available.
- Keyboard Focus switching now covers `Cmd/Ctrl+Alt+1` (Commands), `+2` (Clip List) and `+3` (Groups List), using the corresponding SoundCraft focus state.
- Pro Tools Search now accepts the guide's `Control+Shift+S` on macOS. On Windows/Linux the guide's `Start+Shift+S` has no safe equivalent in the current host; `Ctrl+Shift+S` remains the usable Save As fallback, while `Ctrl/Cmd+S` remains session Save. macOS Save As uses the official `Cmd+Ctrl+S`.
- MIDI Operations coverage now includes `Alt+Q/T/Y/P` (Option on macOS) to open or close the existing MIDI Operations window on Quantize, Transpose, Split Notes and Duration; the remaining gap is the numeric-keypad Step Input focus model.
- MIDI Note Editing coverage now reserves the entire arrow-key family for the piano roll: Up/Down transpose by semitone and Shift+Up/Down by octave, Alt copies and transposes, Left/Right select adjacent notes, Cmd/Ctrl Up/Down adjust velocity, Shift+Left/Right extends selection, and physical Control/Alt combinations trim note starts/ends. The implementation uses the active grid as the nudge value; Pro Tools' key-signature transpose and its separate next-nudge-value preference remain model gaps.
- Arrow navigation, transport shortcuts, marker shortcuts and save/export shortcuts are covered by the existing UI shortcut dispatcher.
- File coverage now includes `Cmd/Ctrl+Shift+O` for Open Most Recent, using SoundCraft's existing recent-session list and unsaved-session prompt; Dashboard, Sketch and Project Revision History remain non-equivalents.
- The Commands Keyboard Focus batch now covers track-view toggle (`-`: waveform/volume and notes/regions), timeline-selection edge display in the main counter (`Q/W`), clip start/sync/end placement (`Y/U/I`, `H/J/K`) and earlier-by-grid nudge (`\\`).
- The same family now covers edit-boundary audition: `6/8` play to the edit start/end and `7/9` play from the edit start/end.
- Transport conflicts audited against the official guide: `Ctrl+Space` pauses, `Alt+Space` toggles Transport Online, and `Ctrl+J`/`Cmd+J` toggles Transport Online according to the platform modifier model.
- Marker entry now follows the guide's modifier split: plain `Enter` creates a marker and opens name entry, while `Cmd/Ctrl+Enter` creates a marker without opening the dialog.
- Playlist/navigation coverage now includes `Shift+Up/Down` for the previous/next alternate playlist and `Shift+T` for Track/Edit selection linking.
- Auditioning coverage now uses the configured Pre/Post-Roll values: `Alt+Left/Right` auditions before an edge, `Ctrl/Cmd+Left/Right` after it, and the combined chord covers the edge through both rolls.
- Edit-selection display coverage now includes `Shift+F` for containing folders and `Shift+U` for the Marker Track lane.
- Marker option coverage now includes `Shift+Y` for Marker Follows Edit.
- Marker display coverage now includes the three-modifier `R`/`F` chords for Marker Ruler Lines and Marker Folder Lines.
- Automation coverage now maps `Cmd/Ctrl+Shift+/` to Trim to Current, `Alt+/` to Glide to Current, `Cmd/Ctrl+Alt+Shift+/` to Trim to All Enabled, and `Alt+Shift+/` to Glide to All Enabled; Current targets the displayed lane while All targets enabled lanes.
- Track View coverage now maps `Control+Command+Left/Right` (and the Windows/Linux collapsed Start+Control form) to cycle the selected tracks' available views; adding Option/Alt cycles every track.
- Automation Special Paste coverage now resolves `Cmd+Ctrl+C/X/B/V` (and the corresponding primary Control chord) by the active track view; it routes to All Automation on an automation lane and to Clip Gain on a clip view. The distinct official `Option/Alt+Shift+X` Cut Special > All Automation chord is also registered.
- The shared `Cmd+Control+B/X/C/V` family is now resolved by the active track view for All Automation, while `Control+Shift+B/X/C` remains Clip Gain; this prevents the command registry from selecting the wrong special-edit family.
- Vertical-navigation conflict coverage now prioritizes MIDI Record Enable movement on `Ctrl/Cmd+Up/Down` when a MIDI/Instrument track is armed; otherwise the chord remains Track Height.
- Target Playlist coverage now uses the existing target-playlist commands: `Shift+Right` shows the target, `Shift+Left` toggles the recent playlist, `Cmd/Ctrl+Shift+Right` designates the selected track(s), and `Cmd+Alt+Shift+Right` designates all shown tracks; the backslash creation/duplication chords route to the selected-track playlist model.
- Zoom/Edit Tool coverage now includes `Ctrl/Cmd+E` for Zoom Toggle and the primary-modifier tool family (`1` Zoom, `2` Trim, `3` Selector, `+` Grabber, `5` Scrubber, `6` Pencil, `7` Smart Tool).
- Clip Gain coverage now includes the line/info toggles, `Control+Shift+B/X/C`, and contextual `Cmd/Ctrl+V` paste when the Clip Gain clipboard is active.
- Clip Gain breakpoint coverage now uses the existing gain envelope: `Control+Shift+E` on macOS and the primary `Start/Ctrl+Shift+E` chord on Windows/Linux add a point at the edit insertion while preserving the audible gain at that point.
- Track insert/send coverage now includes the guide's selected-track toggles: `Shift+A` bypasses all inserts, `Shift+2/3` bypasses A–E/F–J, `Shift+E/C/V/D/W` bypasses EQ/Dynamics/Reverb/Delay/Modulation, `Shift+4/5` mutes sends A–E/F–J, and `Shift+Q` mutes all sends.
- Mixer modifier-click coverage now maps macOS `Control`/Windows `Start` click on an Insert or Send to bypass/mute that slot and every following slot; adding `Option`/`Alt` applies it to all tracks. Normal clicks retain plugin opening or send assignment.
- Track monitoring coverage now makes `Alt+K` alternate record-enabled tracks between Auto Input and Input Only monitoring, instead of only forcing Input Only.
- View coverage now maps the existing Track Transcription Lane flag to Pro Tools' `Control+Shift+U` chord on macOS; the Linux/Windows primary-modifier collision with Clip List Unused remains documented by the host-modifier limitation.
- Edit coverage now maps Snap to Next/Previous to macOS `Option+Control+.`/`,`; the Windows `Alt+Start` form remains unavailable while the host collapses the primary Start/Control modifiers.
- Edit coverage now maps `Option+Shift+H` to Space Clips, `Cmd+Ctrl+R`/`Y` to Trim Clip to File Start/End, and `Cmd+Ctrl+T` to the combined File Boundaries operation.
- Track coverage now maps `Option/Alt+Shift+P` to Save Track Preset, using SoundCraft's existing path dialog and JSON preset writer.
- Event/Clip coverage now maps `Option/Alt+Shift+U` to TCE Edit to Timeline and `Cmd/Ctrl+8` to Beat Analysis. Beat Analysis uses the main-key fallback because egui does not expose keypad identity separately; Pro Tools documents the latter as a numeric-keypad shortcut. Elastic Properties was already mapped to `Option/Alt+5` for the same reason.
- Window coverage now maps macOS `Control+=` and `Control+Option+Shift+=` to the existing MIDI Editor panel; the docked and undocked Pro Tools variants converge on SoundCraft's single docked MIDI Editor.
- Fades coverage now uses the guide's macOS `Control+D`/`Control+G` for Fade to Start/End; the prior Command+Control registrations were corrected.
- Fades coverage also maps macOS `Command+Control+F` to applying a fade/crossfade through SoundCraft's existing selection-based fade command, while `Command+F` remains the Fades dialog action.
- Zoom coverage now handles the standard macOS `Control+1–5` zoom preset family; Windows/Linux remain limited by the host's keypad/main-row and Start/Control modifier collapse.
- Event coverage now includes `Cmd/Ctrl+I` for Identify Beat and `Alt+Shift+Z` for Retrospective Record, backed by the existing Event commands.
- Group coverage now includes `Cmd/Ctrl+Shift+G` to suspend or restore all groups; `Cmd/Ctrl+G` remains New Group.
- Clip List coverage now routes `Cmd/Ctrl+Shift+F` to the existing name filter, `Cmd/Ctrl+Shift+D` clears that filter, and `Cmd/Ctrl+Shift+R` opens Auto Rename for selected clips; category and unused filters are covered below.
- Clip List coverage also reserves macOS physical `Control+Shift+R` for the existing Auto Rename dialog as the closest equivalent to Pro Tools Batch Rename; Windows/Linux cannot distinguish its physical Start chord from the primary rename assignment in the current host.
- Setup coverage now maps `Cmd/Ctrl+Shift+K` to the existing Keyboard Shortcuts window; editing, conflict resolution, presets and export of custom assignments remain unavailable.
- The Keyboard Shortcuts window now uses the guide's physical `Control+Shift+K` chord on macOS. On Windows/Linux the host collapses physical Start with the primary modifier used by Clip List > Export Clips as Files, so the Setup menu remains the unambiguous route.
- Clip List category coverage now maps `Cmd/Ctrl+Shift+1/2/3/4` to Audio/MIDI/Video/Clip Groups, `Cmd/Ctrl+Alt+K` to Auto-Created, and `Cmd/Ctrl+Shift+U` to Unused. Unused is defined by audio sources with no timeline clips in the current model.
- `Cmd/Ctrl+Shift+A` now selects every source and clip visible under the current Clip List name/category filters.
- Options coverage now includes `Cmd/Ctrl+Shift+J` for Video Track Online and `Cmd/Ctrl+P` for Auto-Spot Clips, backed by the existing video/offline and option-flag commands.
- The guide's `Cmd/Ctrl+P` conflict between Print Score and Auto-Spot Clips is now resolved by context: selected MIDI/Instrument content opens Print Score, while other edit contexts toggle Auto-Spot Clips.
- Options coverage now maps `Cmd+Alt+Tab` to the existing Tab-to-Transients flag; the Windows `Ctrl+Alt+Tab` spelling remains subject to the host's primary/Control modifier collapse.
- Selection coverage now registers the official multi-modifier Move/Halve/Double/Duplicate-and-Extend chords against SoundCraft's existing edit-selection operations. Windows/Linux collapsed Start+Control events are dispatched contextually, including the shared L chord for Clip Time Lock versus Move Edit Left.
- Selection vertical navigation now covers `Control/Start+P` and `Control/Start+;` for moving the edit selection, plus `Option/Alt+Control/Start+P/;` for removing its top or bottom track; `;` is now a first-class parsed shortcut key.
- Paste Special coverage now maps `Option/Alt+M` to Merge MIDI and `Option/Alt+V` to Merge Markers. Marker-zone movement/extension is also mapped for macOS `Cmd+Alt+P/;` (and the Shift variants); the Windows spelling requires the unavailable physical Start+Control distinction.
- Clip/Event coverage now includes `Cmd/Ctrl+Alt+0` for Quantize MIDI to Grid, routed through the existing MIDI quantize engine operation; `Cmd/Ctrl+0` remains Quantize Clip to Grid.
- Clip Effects coverage now maps the existing copy/cut/clear actions to the official `Cmd+Alt+Ctrl+C/X/B` (or `Ctrl+Alt+Start+C/X/B`) family and uses contextual `Cmd/Ctrl+V` for Paste Clip Effects when that clipboard is active; preset save/recall remains unavailable.

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
| Clip Effects | `Option/Alt+6`, `Cmd+Alt+Ctrl+B/X/C/S` | Display and copy/cut/clear/paste are mapped to SoundCraft's existing clip-effect model; Pro Tools preset save/recall and the full processing UI remain unavailable. |
| Elastic Audio | Warp-marker add/move/delete and rendered/real-time toggle | No Elastic Audio event/warp-marker model exists. |
| MIDI Operations and Step Input | Numeric duration keys, panel navigation | SoundCraft opens Time/Tempo/MIDI Operations and Real-Time Properties with `Alt+1–4` main-key fallbacks; it lacks Pro Tools' numeric-keypad Step Input focus model. |
| Workspace/Browser | Workspace search, browser navigation and drag/drop modifier matrix | No Workspace/Soundbase browser model; ordinary file drag/drop is implemented separately in the Edit window. |
| Deck/Satellite/ICON control surfaces | Deck, Satellite and hardware-specific modifier shortcuts | No external deck, satellite or ICON control-surface integration exists. |
| Custom Keyboard Shortcuts window | Setup > Keyboard Shortcuts, conflict resolver, presets and export | SoundCraft opens the read-only command/shortcut catalogue with the guide's platform-specific shortcut, but has no user-editable assignment window or preset format equivalent to Pro Tools. |
| Marker target/priority shortcuts | `Shift+[ / ]`, marker-name/comment priority clicks | SoundCraft maps `Shift+X` to follow the first selected track and `Shift+N` to toggle marker names/comments; it still has no independent target navigation or per-track display-priority state. |
| Advanced automation editing | Automation write-on-stop, plugin automation dialogs and multi-playlist special paste | SoundCraft now maps Trim/Glide to Current and All Enabled onto its automation lanes; it still has no write-on-stop/plugin-automation state model for those shortcut families. |
| Clip Gain fine navigation | `Control/Start+Shift+M`, `,`, `.` and `/` for moving selected gain data back/forward | SoundCraft supports Clip Gain up/down and breakpoint insertion, but its model has no separate horizontal gain-point nudge operation. |
| Track compositing extras | `Shift+S` playlist-lane solo and `Cmd/Start+Shift+Up` cycle material from the previous playlist | SoundCraft maps playlist movement, target designation and recent-playlist switching; it has no independent lane-solo or material-cycle state. |
| Plugin insert/send assignment modifiers | Control/Start-click an insert or send; Control/Option-click all tracks | Insert bypass and Send mute modifier-click families are mapped; SoundCraft still has no equivalent plugin/send assignment popup matrix for modifier-click assignment changes. |
| Clip List filters and bulk actions | Clip Definition export, Time Stamp selected Clips and remaining bulk-action semantics | SoundCraft now has Find/Clear, Audio/MIDI/Video/Groups, Auto-Created, Unused, filtered Select All, Clear and the closest Rename/Batch Rename dialog mapping; it still lacks Pro Tools' Clip Definition export, Time Stamp command and some bulk-action semantics. |
| Options: Dynamic Transport and Edit Tool Mode Keyboard Lock | `Command+Control+P` / `Control+Start+P`; `Control+Shift+T` / `Start+Shift+T` | SoundCraft maps the exact macOS Control forms; Linux/Windows remain available from Options because the current host collapses the physical Start/Control modifier into the primary chord, which would steal `Auto-Spot Clips`/`TrackPunch`. |
| Options: cycle Video Track Online | `Shift+J` | SoundCraft has one global Video Online state rather than Pro Tools Ultimate's per-visible-video-track cycling model; `Cmd/Ctrl+Shift+J` toggles the available global state. |
| Numeric-keypad memory recall | `.` + number + `.` / numeric-keypad marker entry | The host exposes logical egui keys without a reliable keypad-vs-main-row distinction; SoundCraft therefore cannot safely assign these without stealing ordinary number-row shortcuts. |
| Numeric-keypad window and Session Setup shortcuts | `Cmd/Ctrl+1/2/3/5/7` on the numeric keypad | These chords collide with SoundCraft's already-dispatched Pro Tools Edit Tool/Commands Focus number-row family (`1` Zoom, `2` Trim, `3` Selector, `5` Scrubber, `7` Smart Tool). The misleading window/session registrations were removed; use the Window/Setup menus until the host exposes keypad identity. |
| Project/revision-history dialog shortcuts | `Cmd+Control+O`, revision list navigation and restore/import/delete keys | SoundCraft stores `.scraft` sessions directly and has no Pro Tools Project revision-history model. |
| Dialog-specific preset/navigation shortcuts | Bounce, Fades, Commit, Import Session Data, Import Audio and Save Track Presets dialog key matrices | The corresponding SoundCraft dialogs expose a smaller model; assigning Pro Tools keys that target controls absent from those dialogs would not be equivalent. |
