# glass-evo

The Glass interface: a face for Volumio players, drawn by Glass's engine on the player's own screen, with no browser on the device. Part of the [evo framework](https://evoframework.org) family, built to run on today's Volumio and at home on evo devices.

It is a preview. The [Glass](https://github.com/foonerd/glass) Manager gets it and hands the player's screen to it, the Manager's Face tab and Anymote show it in a browser, and the releases here carry a bundle that puts it on Glass's remote displays. The [wiki](https://github.com/foonerd/glass-evo/wiki) has how to get it, choose it and theme it.

## What it is

Volumio installs and runs as it always has, Node backend and every plugin included; the Glass plugin is the driver. The Glass Manager gets glass-evo on its System tab, and its Screen tab hands the device screen to it, with the kiosk browser off while it holds the screen, and back when the screen is given back. Phones and computers keep Volumio's web interface in their own browsers.

On the screen today: Glass's theme and meters with a bar of controls on demand (previous, play or pause, next, volume down and up, and More, a sheet with repeat, random and mute) and, when the player stands still, a clock and a date. To come: browse across every source a plugin provides, the queue, search, and the settings pages, rendered from the same UIConfig data the web interface renders, so a plugin needs nothing of its own to appear.

The look follows the artwork: the controls sit on glass over the theme, frosted where the player has room for it, and the glass and what is lit take their colours from the cover, so the face changes mood with every album.

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

0.1.44. Glass's display with a face over it: a bar of controls on demand (previous, play or pause, next, volume down and up, and More, a sheet with repeat, random and mute), a long press on volume down that mutes, and a clock, a date and today's forecast for a place of the user's choosing when the player stands still, the clock and the date each in a pattern of the user's choosing, the clock set in type or drawn (seven or sixteen segments, a flip clock, a dial with hands in four styles); everything at the face size the Manager's Screen tab sets, normal, large or car. The clock and the date can be placed on a grid of nine cells above the bar, each on one cell or a block of them, aligned inside what it occupies and a margin of its own from the sides. The look follows the artwork on frosted glass, or is one of the looks that ship (Dark Glass, Clear, Warm, Night Drive), takes the user's own adjustments, and can be themed: a face theme is a folder with a `face.txt` (see `themes/Example` and the wiki's Face themes page), and a meter theme may bring one beside its `meters.txt`, laid over the chosen look while that theme is on show. Released as the component the Glass Manager installs. From Glass 0.8.0 the Glass Manager gets it and keeps it up to date on its System tab, and hands the screen to it and back on its Screen tab; the kiosk stays the player's interface until then and returns whenever the screen is given back. On a Raspberry Pi the face draws on the screen itself; on x86 it draws on a plain X server brought up for it. A preview: verified on a Raspberry Pi 5 with a DSI screen and on an x86 player.

## On a remote display

A Glass remote display, another machine showing a player's meters, comes in two flavours. The **standalone** remote, from [Glass's releases](https://github.com/foonerd/glass/releases), shows the player's theme and nothing over it. The **bundle**, from this repository's releases, is the same display with the face in it: the clock and the date when the player stands still, and the bar of controls.

| | Standalone | Bundle |
| --- | --- | --- |
| Linux (x64, armv8, armv7) | yes | yes, from 0.1.18 |
| Windows (10 or later, 64 bit) | yes | yes, from 0.1.19 |
| Android (7 or later) | yes | yes, from 0.1.20 |

On Linux, unpack `glass-evo-<version>-<arch>.tar.gz` (`x64` for a PC, `armv8` for a 64-bit Raspberry Pi OS, `armv7` for a 32-bit one) and run its installer as the user who will run the display:

```sh
tar xzf glass-evo-<version>-<arch>.tar.gz
glass-evo-<version>-<arch>/remote/linux/install.sh
```

The installer is Glass's own and says which flavour it installed. The bundle takes the standalone's place, and the standalone the bundle's when its archive is installed again: the settings, the cache and the menu entries stay. The remote's settings page (port 5583) then has **The Glass interface**: the Glass interface while glass-evo holds the player's screen (the default; before Glass 0.8.36 this choice read "what the player's screen shows"), the Glass interface always, or the theme alone. The look is the player's: the remote brings it with the theme. The controls need a touch screen or a mouse on the remote; the clock needs neither. The player needs Glass 0.8.20 or later.

On Windows, unpack `glass-evo-<version>-windows-x64.zip` and run its installer in a PowerShell window, as the user who will run the display:

```powershell
powershell -ExecutionPolicy Bypass -File glass-evo-<version>-windows-x64\remote\windows\install.ps1
```

The display and the installer scripts are signed. The installer says which flavour it installed, and the bundle takes the standalone's place as on Linux. The controls need a mouse or a touch screen.

On Android, download `glass-evo-<version>-android.apk` to the phone or tablet, open it and allow the install. It is the same app as Glass's standalone remote, **Glass Remote**, signed with the same key: it installs over the standalone and keeps its settings, and the standalone installs over it again. Android itself refuses an app built on an older Glass than the one installed, so change flavour to one built on the same Glass or a newer one, or uninstall first. The bar comes at a touch.

For a device with nothing installed, Anymote shows the same face in any browser, served by the player's manager at `/anymote`.

## Building

```text
scripts/check.sh
```

runs formatting, lints with warnings denied, the tests and the documentation, as CI does.

## Licence

MIT; see `LICENSE`.
