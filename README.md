# glass-evo

The native face for Volumio players: browse, queue and playback on the device's own screen, drawn by Glass's engine, with no browser on the device. A greenfield project: there is nothing to install yet.

## What it will be

Volumio installs and runs as it always has, Node backend and every plugin included; the [Glass](https://github.com/foonerd/glass) plugin is the driver. A switch in that plugin, "use the Glass interface", fetches glass-evo and hands the device screen to it, with the kiosk browser off while it is on, and back when it is off. On the screen: now playing with Glass's meters and themes, browse across every source a plugin provides, the queue, transport and volume, and every settings page, rendered from the same UIConfig data the web interface renders, so a plugin needs nothing of its own to appear. Phones and computers keep Volumio's web interface in their own browsers. Remotes show the same face through Glass's channel, and the browser is a second target from the start.

The look follows the artwork: the cover fills the screen softly blurred, the controls sit on frosted glass above it, and the colours are sampled from the artwork, so the whole face changes mood with every album.

## Relation to Glass

glass-evo is built on Glass's crates rather than beside them: the renderer, the theme engine, the channel and the fonts. Glass remains the display for every player that keeps Volumio's kiosk; glass-evo is the face for the players that choose it.

## Credits

- [Artwork One](https://community.volumio.com/t/artwork-one-a-new-interface-for-volumio-os-work-in-progress/77568) by Michal Tamas, for the design language the face follows.
- [Now Playing](https://github.com/patrickkfkan/volumio-now-playing) by patrickkfkan, for the screen set: now playing, browse, queue, an idle screen with clock and weather, an action panel.
- [PeppyMeter](https://github.com/project-owner/PeppyMeter) by project-owner, for the theme format Glass reads and the face inherits.

## Status

Scaffold: the workspace, the toolchain pin, the workshop check and CI. No release.

## Building

```text
scripts/check.sh
```

runs formatting, lints with warnings denied, the tests and the documentation, as CI does.

## Licence

MIT; see `LICENSE`.
