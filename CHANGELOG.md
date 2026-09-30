# Changelog

All notable changes to glass-evo are recorded here. The format follows Keep a Changelog; versions follow Semantic Versioning.

## [0.1.0] - 2026-09-30

The first face, and the component that carries it. glass-evo is Glass's display with a face over it: the same theme, meters, artwork and never-empty screen, drawn by Glass 0.7.87, and on top of it a bar at the foot of the picture with previous, play or pause, next, volume down and volume up, and a clock when the player stands still on a screen that is the display's own. The bar takes touches before the theme's controls and sends its commands the way the theme's buttons go. The release carries the component the Glass Manager installs: `manifest.json` with the version, the build and each binary's sha256, and the binaries for arm, armv7, armv8 and x64 under `bin/`. The switch that puts glass-evo on the screen lives in Glass and ships with 0.8.0.
