# Changelog

All notable changes to glass-evo are recorded here. The format follows Keep a Changelog; versions follow Semantic Versioning.

## [0.1.7] - 2026-10-01

A glass of its own colour for the clock and for the date. `clock.tint` and `date.tint` take a colour, or `tint` for the theme's own: the clock's glass, which a date above or below it shares, and the glass of a date at the top of the screen. The Glass Manager's look panel has a switch and a colour for each beside their other settings.

## [0.1.6] - 2026-10-01

The clock and the date. The idle screen's words follow the same rules as the controls, and are the theme's and the user's to set.

- **The controls lie over the clock.** The clock was drawn last and covered the sheet where the two met; it is drawn first now, the bar and the sheet over it.
- **The clock on glass.** Its plate is glass like the bar's: the artwork's or the user's tint, the hairline, frosted where frost is on, at an opacity of its own (`clock.glass`, 0 for no glass). The separate plate colour is gone from the theme format; `clock.plate` is still read as `clock.glass`.
- **The clock as a pattern.** `clock.format` says what shows and in which order, as `strftime` reads it: `%H:%M`, `%H:%M:%S` with the seconds, `%-I:%M %p` for 1:05 PM. `clock.show = off` leaves the clock out.
- **The date.** `date.show = on` adds the date in a pattern of its own (`%A %-d %B`, `%d/%m/%Y`, any order), at the top of the screen on a glass of its own, or above or below the clock on the clock's; with its own ink, opacity, glass and measure.
- **Nothing runs off the screen.** A line too wide or too tall for the room there is, a long clock at the car size, is set as large as fits, and the clock stands in what a date at the top and the bar leave.
- **Set once.** The clock and the date are rastered when their words change, not every frame, and their glass keeps its width while the digits change.

The Glass Manager's look panel gains the clock and the date, each with its switch and its pattern, the date's place, and for the navigation, the clock and the date a size of their own.

Built on Glass 0.7.96, which brings two fixes of its own to the screen: the bar's last faint frame is no longer left over the theme when it has faded out, and a letter no longer comes out as a grey smear (the w of some titles).

## [0.1.5] - 2026-10-01

The look. What the face draws with is no longer fixed in its code: it comes from a face theme, with the user's own word over it.

- **Colours from the artwork.** The glass takes the colour of the cover of what plays, made dark, and what is lit, a mode that is on, the More button while its sheet is open, takes the cover's most vivid colour. The cover is read once per track, off the frame's path, and only while the face has something to draw; a cover with no colour to speak of gives a neutral look.
- **Frosted glass.** What lies under the bar and the sheet is frosted while the theme moves, so a theme's own text and controls no longer show through the buttons. It is frosted once while the picture under the glass stands still. On by itself where the board has room for it, off on the small ones, and the user's switch decides either way.
- **The user's own look**, on the Glass Manager's Screen tab: the colours from the artwork or their own, the glass's opacity, frost, the buttons and the clock on their own, and a reset to the defaults.
- **Face themes.** A theme is a folder with a `face.txt`, sections of `key = value`, the way meter themes are written: colours, the glass's opacities, the hairline, frost, the buttons' and the clock's own ink and opacity, the bar's and the clock's measure. What it leaves out keeps the built-in look; a key a face does not know is passed over. `themes/Example/face.txt` documents every key; the wiki's Face themes page has the reference. The format is provisional until the first full preview.

Built on Glass 0.7.95, which hands the face its settings.

## [0.1.4] - 2026-10-01

More, and mute. The bar gains a sixth button at its right end, More, which opens a sheet above it with three tiles: repeat, walking off, all, single and off again; random; and mute. A tile is lit while its mode is on in the player, and repeat shows a one inside its arrows on single, so the sheet says how the player stands as well as changing it. The sheet stays open for the next tile, shuts on a tap beside it, which the theme under it does not act on, and leaves with the bar. A finger resting on volume down for six tenths of a second mutes and unmutes without stepping the volume; while the player is muted that button shows a muted speaker in place of its minus, and a tap on it unmutes.

The icons are drawn anew: each a shape rastered once for the size in use, with smooth edges at every face size, and blitted from then on; no fonts and no files. Sheet, tiles and icons scale with the face size as the bar does.

## [0.1.3] - 2026-09-30

A face size. Glass 0.7.89 hands the face a size, normal, large or car, set on the Manager's Screen tab; the bar's height, its glyphs and the clock scale together, the bar never taking more than a third of the picture, so a hand at arm's length or a glance while driving finds them.

## [0.1.2] - 2026-09-30

The bar lingers six seconds after a touch while playing, long enough for a hand reaching out, in a car too; it starts settled rather than mid-fade; and at the display's finest log level the face says what each touch met, the bar there or away, so a screen can be read from the journal.

## [0.1.1] - 2026-09-30

The bar is on demand. Playing shows the theme and nothing else: the bar leaves two seconds after playback begins, a touch on the picture brings it back for four seconds, a touch on it keeps it, a second touch on the picture sends it away; stopped or paused, the bar stays with the clock. It fades in and out over a fifth of a second, and while it is away the face draws nothing, so a playing screen costs what Glass costs. The theme's own controls see every touch on the picture in both states.

## [0.1.0] - 2026-09-30

The first face, and the component that carries it. glass-evo is Glass's display with a face over it: the same theme, meters, artwork and never-empty screen, drawn by Glass 0.7.87, and on top of it a bar at the foot of the picture with previous, play or pause, next, volume down and volume up, and a clock when the player stands still on a screen that is the display's own. The bar takes touches before the theme's controls and sends its commands the way the theme's buttons go. The release carries the component the Glass Manager installs: `manifest.json` with the version, the build and each binary's sha256, and the binaries for arm, armv7, armv8 and x64 under `bin/`. The switch that puts glass-evo on the screen lives in Glass and ships with 0.8.0.
