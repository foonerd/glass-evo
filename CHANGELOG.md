# Changelog

All notable changes to glass-evo are recorded here. The format follows Keep a Changelog; versions follow Semantic Versioning.

## [0.1.2] - 2026-09-30

The bar lingers six seconds after a touch while playing, long enough for a hand reaching out, in a car too; it starts settled rather than mid-fade; and at the display's finest log level the face says what each touch met, the bar there or away, so a screen can be read from the journal.

## [0.1.1] - 2026-09-30

The bar is on demand. Playing shows the theme and nothing else: the bar leaves two seconds after playback begins, a touch on the picture brings it back for four seconds, a touch on it keeps it, a second touch on the picture sends it away; stopped or paused, the bar stays with the clock. It fades in and out over a fifth of a second, and while it is away the face draws nothing, so a playing screen costs what Glass costs. The theme's own controls see every touch on the picture in both states.

## [0.1.0] - 2026-09-30

The first face, and the component that carries it. glass-evo is Glass's display with a face over it: the same theme, meters, artwork and never-empty screen, drawn by Glass 0.7.87, and on top of it a bar at the foot of the picture with previous, play or pause, next, volume down and volume up, and a clock when the player stands still on a screen that is the display's own. The bar takes touches before the theme's controls and sends its commands the way the theme's buttons go. The release carries the component the Glass Manager installs: `manifest.json` with the version, the build and each binary's sha256, and the binaries for arm, armv7, armv8 and x64 under `bin/`. The switch that puts glass-evo on the screen lives in Glass and ships with 0.8.0.
