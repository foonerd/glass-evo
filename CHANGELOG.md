# Changelog

All notable changes to glass-evo are recorded here. The format follows Keep a Changelog; versions follow Semantic Versioning.

## [0.1.4] - 2026-10-01

More, and mute. The bar gains a sixth button at its right end, More, which opens a sheet above it with three tiles: repeat, walking off, all, single and off again; random; and mute. A tile is lit while its mode is on in the player, so the sheet says how the player stands as well as changing it. The sheet stays open for the next tile, shuts on a tap on the picture, and leaves with the bar. A finger resting on volume down for six tenths of a second mutes and unmutes without stepping the volume, and the button wears a slash while the player is muted. Sheet, tiles and glyphs scale with the face size as the bar does.

## [0.1.3] - 2026-09-30

A face size. Glass 0.7.89 hands the face a size, normal, large or car, set on the Manager's Screen tab; the bar's height, its glyphs and the clock scale together, the bar never taking more than a third of the picture, so a hand at arm's length or a glance while driving finds them.

## [0.1.2] - 2026-09-30

The bar lingers six seconds after a touch while playing, long enough for a hand reaching out, in a car too; it starts settled rather than mid-fade; and at the display's finest log level the face says what each touch met, the bar there or away, so a screen can be read from the journal.

## [0.1.1] - 2026-09-30

The bar is on demand. Playing shows the theme and nothing else: the bar leaves two seconds after playback begins, a touch on the picture brings it back for four seconds, a touch on it keeps it, a second touch on the picture sends it away; stopped or paused, the bar stays with the clock. It fades in and out over a fifth of a second, and while it is away the face draws nothing, so a playing screen costs what Glass costs. The theme's own controls see every touch on the picture in both states.

## [0.1.0] - 2026-09-30

The first face, and the component that carries it. glass-evo is Glass's display with a face over it: the same theme, meters, artwork and never-empty screen, drawn by Glass 0.7.87, and on top of it a bar at the foot of the picture with previous, play or pause, next, volume down and volume up, and a clock when the player stands still on a screen that is the display's own. The bar takes touches before the theme's controls and sends its commands the way the theme's buttons go. The release carries the component the Glass Manager installs: `manifest.json` with the version, the build and each binary's sha256, and the binaries for arm, armv7, armv8 and x64 under `bin/`. The switch that puts glass-evo on the screen lives in Glass and ships with 0.8.0.
