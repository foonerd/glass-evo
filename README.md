# glass-evo

The native face for Volumio players: browse, queue and playback on the device's own screen, drawn by Glass's engine, with no browser on the device. Part of the [evo framework](https://evoframework.org) family, built to run on today's Volumio and at home on evo devices. A greenfield project: there is nothing to install yet.

## What it will be

Volumio installs and runs as it always has, Node backend and every plugin included; the [Glass](https://github.com/foonerd/glass) plugin is the driver. A switch in that plugin, "use the Glass interface", fetches glass-evo and hands the device screen to it, with the kiosk browser off while it is on, and back when it is off. On the screen: now playing with Glass's meters and themes, browse across every source a plugin provides, the queue, transport and volume, and every settings page, rendered from the same UIConfig data the web interface renders, so a plugin needs nothing of its own to appear. Phones and computers keep Volumio's web interface in their own browsers. Remotes show the same face through Glass's channel, and the browser is a second target from the start.

The look follows the artwork: the cover fills the screen softly blurred, the controls sit on frosted glass above it, and the colours are sampled from the artwork, so the whole face changes mood with every album.

## Built for evo

[evo](https://evoframework.org) is a brand-neutral steward for appliance-class devices: a catalogue of racks, shelves and slots that plugins stock, with consumers reading projections and happenings, so that sources, processing, outputs, metadata, networking and presentation compose into one coherent device without becoming a monolith. Everything is a plugin. The framework lives at [evo-core](https://github.com/foonerd/evo-core), the reference audio device at [evo-device-audio](https://github.com/foonerd/evo-device-audio), the Volumio vendor layer at [evo-device-volumio](https://github.com/foonerd/evo-device-volumio), and the Rust Volumio backend at [volumio-evo](https://github.com/foonerd/volumio-evo).

glass-evo is the face of that family for the listening room. On today's Volumio it runs through the Glass plugin; on an evo device it consumes the steward's projections and happenings directly. The same face, one engine, wherever the player lives.

## Relation to Glass

glass-evo is built on Glass's crates rather than beside them: the renderer, the theme engine, the channel and the fonts. Glass remains the display for every player that keeps Volumio's kiosk; glass-evo is the face for the players that choose it.

## Credits

- [Artwork One](https://community.volumio.com/t/artwork-one-a-new-interface-for-volumio-os-work-in-progress/77568) by Michal Tamas, for the design language the face follows.
- [Now Playing](https://github.com/patrickkfkan/volumio-now-playing) by patrickkfkan, for the screen set: now playing, browse, queue, an idle screen with clock and weather, an action panel.
- [PeppyMeter](https://github.com/project-owner/PeppyMeter) by project-owner, for the theme format Glass reads and the face inherits.

## Status

0.1.7. Glass's display with a face over it: a bar of controls on demand (previous, play or pause, next, volume down and up, and More, a sheet with repeat, random and mute), a long press on volume down that mutes, and a clock and a date when the player stands still, each in a pattern of the user's choosing; everything at the face size the Manager's Screen tab sets, normal, large or car. The look follows the artwork on frosted glass, takes the user's own colours, and can be themed: a face theme is a folder with a `face.txt` (see `themes/Example` and the wiki's Face themes page). Released as the component the Glass Manager installs. The switch that puts glass-evo on the screen ships with Glass 0.8.0; until then the kiosk stays as it is on every player.

## Building

```text
scripts/check.sh
```

runs formatting, lints with warnings denied, the tests and the documentation, as CI does.

## Licence

MIT; see `LICENSE`.
