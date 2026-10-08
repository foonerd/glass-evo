# Changelog

All notable changes to glass-evo are recorded here. The format follows Keep a Changelog; versions follow Semantic Versioning.

## [0.2.4] - 2026-10-07

Built on Glass 0.9.3, for the look panel's likeness with Glass 0.9.4.

- **The likeness finds a look's skies by the look's name alone.** `forecast_preview` looked for the look's folder by its `face.txt`, which a page puts only for the saved look; it takes the folder by the look's name under the first of the faces folders now, so a look chosen on its card draws its own skies from the files the page put.

## [0.2.3] - 2026-10-07

Built on Glass 0.9.3: a face theme's own skies reach the browser views, the remote displays and the look panel's likeness.

- **The likeness draws the look's own skies.** `forecast_preview` takes the folders face themes are kept in and looks for the chosen look's skies there, as the screen does; the Manager's look panel puts the look's sky files into the module's table under the look's folder.
- **A bundle remote brings a look's skies** beside its `face.txt`, by the files the player's configuration lists (Glass 0.9.3), and the Face tab and Anymote have them in their table before the first frame.

## [0.2.2] - 2026-10-07

Gelo5 on the forum: "where are the weather icons located?" Just a Nerd: "lets take on weather icons first". Built on Glass 0.9.2.

- **A theme's own skies.** A `skies` folder beside a `face.txt`, the meter theme's on show first and then the chosen look's, holds one picture per sky, PNG, animated GIF, WebP or JPEG, square, fitted to the sky's side: `clear-day`, `clear-hot`, `clear-night`, `partly-day`, `partly-night`, `cloudy`, `fog`, `drizzle`, `rain`, `rain-heavy`, `snow`, `snow-heavy`, `thunder`, `thunder-night`. A missing name falls back along a short chain (hot, heavy and a night's thunder to the plain name; a night sky never to a day's) and then to the drawn sky, so a theme may replace one or all. An animated GIF keeps its motion by its own delays while the skies move, the frame picked by the wall clock as the drawn skies' is; `Skies move` off shows its first frame. The colour and heat switches leave a theme's pictures as they are. A meter theme's skies travel with the theme to the Face tab, Anymote and the remotes; a face theme's skies reach those, and the look panel's likeness, in the next step.

## [0.2.1] - 2026-10-07

Built on Glass 0.9.1, for the browser views: a text or time field's own font file, named in a meter theme as `fonts/font.ttf`, is found in the page's table as in a theme folder on the player, so the Face tab and Anymote draw the field in the theme's font. Nothing of the face itself changes.

## [0.2.0] - 2026-10-07

glass-evo 0.2.0 gathers the 0.1 series since the first pair users got, 0.1.10 with Glass 0.8.0 on 1 October 2026, into one release with Glass 0.9.0. Every change below has its own entry under its number.

- **The face costs nothing where nothing changes**, and a change of track is not the player standing still (0.1.11, 0.1.21); a two-sided look keeps its two sides through silence and a mono recording (0.1.12, 0.1.16).
- **The clock in five faces**, type, seven and sixteen segments, a flip clock, a dial with hands in four styles, each with its colours, drawn as shapes (0.1.15); a size the user sets drawn as set, over the edges (0.1.13).
- **The idle screen on a grid.** The clock, the date and the forecast each on nine cells above the bar, aligned inside them with a margin of the user's own, a piece larger than its cells running over on the side it is aligned to, pieces on the same cells sharing one glass (0.1.39 to 0.1.44).
- **The weather.** Today's forecast, the weather now and the day's low and high with their captions; the span, the next hours or the week as columns; the heatmap, numbers only, the week's days and the date by their switches; the skies in colour, moving at their own pace, thunder flashing by its switch, heavy weather murkier (0.1.44 to 0.1.51).
- **A picture of your own when nothing plays**, in the theme's place, darkened as you say, on the player's screen and, from 0.1.52, on the browser views and the remote displays; the screen off after minutes with nothing playing, over a fade (0.1.32, 0.1.35 to 0.1.36, 0.1.52).
- **When the player stops**, the idle screen at once or after the plugin's persist period, `[idle] wait` (0.1.53).
- **Every piece's own colour, strength, background and background colour**, the buttons' included, each from the look's Colours and Backgrounds and from no other piece (0.1.47, 0.1.54); a meter theme's own `face.txt` laid over the chosen look (0.1.26).
- **The face for a browser.** The module Glass's Manager serves to its Face tab and Anymote, drawing the face over the theme and setting the look panel's likeness, the clock in its faces, the date and a clock in type in the look's own font, the forecast at its moment (0.1.14, 0.1.15, 0.1.43 to 0.1.45, 0.1.51).
- **The face on remote displays**, the bundles for Linux, Windows and Android beside the component, bringing themselves up to date (0.1.18 to 0.1.20, 0.1.30 to 0.1.31).
- **With the themes.** Text fields with a font of their own, the radio icons by signal and mono, the volume number's words, a cue track's type, rotation quality pacing the turning, a Lyrion server's covers, the caches bounded, the screen taken on a DSI panel at every boot, no black frame at a countdown's end (0.1.22 to 0.1.25, 0.1.27 to 0.1.29, 0.1.33 to 0.1.34, 0.1.37 to 0.1.38).

## [0.1.54] - 2026-10-07

Just a Nerd: "FIX - never reported: Screen -> Buttons - does not have custom background override like all other element have." Built on Glass 0.8.90.

- **The buttons' own background.** `[buttons] glass`: how solid the bar's glass is, 0 to 1, or `bar` (unless said) for the look's `[glass] bar`; `[buttons] tint`: the bar's and the More sheet's colour, or `tint` (unless said) for the look's. As the clock, the date and the forecast have theirs. A look that says neither draws as before.

## [0.1.53] - 2026-10-07

Just a Nerd: "I selected to wait 15 second - NO LONGER RESPECTED! THIS SHOULD BE A SWITCH IN SCREEN - IMMEDIATE - RESPECT TIMEOUT". The face had decided idle by its own rule since 0.1.11 and never read the plugin's persist period. Built on Glass 0.8.88.

- **The idle screen can wait for the persist period.** `[idle] wait = persist`: while the plugin keeps the display after a pause or a stop ("Keep Display Active After Pause/Stop" on its settings page, a mode named and seconds left), the player does not count as standing still: the theme stays, with its countdown where that setting says so, and the clock, the date, the forecast and the picture come when the period ends. `none`, unless said, is the idle screen at once, as before. The five-second grace for a stop between two tracks is unchanged.

## [0.1.52] - 2026-10-07

Just a Nerd: "REMOTE AND FACE AND ANYMOTE - ALL LOST CUSTOM BACKGROUND", on the picture when nothing plays, which 0.1.32 drew on the player's own screen alone ("Not yet on remote displays and in the browser views"). Built on Glass 0.8.87.

- **The picture when nothing plays, on the browser views and the remote displays.** The face asks the host for the picture the look names before it looks in the folder its launcher names: in the Face tab and on Anymote the page brings it (`overlay::face::host_picture`, wanted from the Manager as the album art is), on a remote display the sync brings it beside the faces and names the folder. The theme shows until the picture is in, as on the player; the stamp carries its arrival, so the screen redraws the moment it is.

## [0.1.51] - 2026-10-07

Just a Nerd's step 4 of the weather: "Now - Dramatise icons. Snow, thunderstorm, rain, drizzle, heat, etc. I am sure we can add some light frame animations here." and "If Dramatise is off - should not affect current refresh rate." Built on Glass 0.8.86.

- **The skies move.** Each sky keeps its shape and gains one small motion at its own pace: rain and drizzle fall, snow drifts with a sway, the sun's rays turn once a minute, a star winks beside the moon, clouds and fog sway, heat haze shimmers under the sun from 30 °C, and thunder's cloud darkens with the bolt under it. A moving sky is a cycle of frames rastered once per size and ink; the wall clock picks the frame, so every screen and the likeness show the same frame at the same moment, and the face asks the display to redraw only when a frame changes, at the sky's own pace. `[weather] motion`, on unless said; off, every sky stands still and nothing about the refresh changes.
- **The skies in colour**, `[weather] colour`, on unless said: a yellow sun and a red one when scorching, a pale moon with a white star, clouds and fog in greys, drizzle and rain in blues, snow white, the storm's cloud dark with a yellow bolt, and heavy weather (heavy rain or snow, violent showers, by the reading's code) murkier with more falling from it. Off, every sky is in the forecast's ink as before.
- **Thunder's flashes**, `[weather] thunder`, off unless said: the bolt and the cloud light up for a tenth of a second every eight to twenty seconds, the same on every screen.
- **For a page** `forecast_preview` draws the frame of its moment, so the likeness moves with the screen.

## [0.1.50] - 2026-10-07

Just a Nerd's step 3 of the weather: "Forecast - add Heatmap toggle/slider - perhaps color picker for start and end? with some logical defaults? This is for numbers only. Also, second toggle - apply to days, still forecast. Apply to date - override date with heatmap - first override exception." Built on Glass 0.8.85.

- **The heatmap.** `[weather] heat` sets every temperature the forecast shows in the colour of its degree, numbers only: from `cold` at -10 °C, through the forecast's own ink at 12, to `warm` at 30, straight between and held beyond, Celsius inside and a Fahrenheit reading converted. `heat.days` colours the week's lows and highs too, each its own. `heat.date` sets the date in the colour of the temperature now, with the date's own ink in the middle of the scale: the one link between pieces, by the user's switch, and only while the player holds a reading. Every switch is off unless said; `cold` is `#3b8bff` and `warm` `#ff4b2b` unless said.
- **For a page** `line_preview` takes the player's reading beside the keys, so the likeness colours the date as the screen does.

## [0.1.49] - 2026-10-07

Just a Nerd: "The slider for the forecast is still regressed. Today - as every other slider should permit spill over the screen... REQUIREMENT - restore slider function exactly like it was." Built on Glass 0.8.84.

- **Today's size is its numbers' size again, exactly as before the captions.** 0.1.46 and 0.1.48 hung today's line from its skies; the numbers came out smaller at every size and the slider's top no longer ran the line over the screen. The numbers are set at the forecast's size as in 0.1.45, and T21's proportions hang from them: the skies five thirds of the numbers, the captions three eighths, drawn an eighth up into the numbers' descent.

## [0.1.48] - 2026-10-07

Just a Nerd: "Forecast 'Today' lost full scale range. All others scale fine." Built on Glass 0.8.84.

- **Today's line reaches as far as it did.** 0.1.46 made today's skies as high as the forecast's size itself, where the line before it was as high as a line of type at that size, about a seventh more; with the numbers at three fifths of the skies, today came out smaller than before at every size. The skies are a line of type high again, the numbers and the captions their shares of that, so the size runs over the same range it did.

## [0.1.47] - 2026-10-07

Just a Nerd: "Global is background, not date - should not be linked at all." Built on Glass 0.8.84.

- **The forecast's colour, opacity, glass and tint are its own.** 0.1.44 made them the date's unless the look said otherwise; nothing links one piece to another now. `weather.ink` and `weather.tint` are the theme's ink and tint unless said, as the clock's and the date's are; `weather.opacity` is 0.86 and `weather.glass` 0.55 unless said, as the date's.

## [0.1.46] - 2026-10-07

Just a Nerd, on the span's columns and today's line, with mockups drawn by the face itself: "C4 - certainly." and "T21 is a sweetspot." Built on Glass 0.8.82.

- **Today's line as T21.** The skies at the forecast's size, the numbers at three fifths of it, and NOW, MIN and MAX set letter-spaced under their numbers, drawn a little up into the numbers' descent; the skies centred on number and caption together.
- **The columns as C4.** The skies at the forecast's size, the figures at a little over a half of it, the hour or weekday labels at two fifths; half a sky between columns, an eighth of a sky between the rows.
- The forecast's size now names the skies' height in every span; a size the look comes with is fitted to the cells, a size the user set is kept, as before.

## [0.1.45] - 2026-10-07

Just a Nerd's step 2 of the weather, from his question of 2026-10-06: "What if user wants a day (24h) forecast, or day every 2 or 4 forecast? Or week forecast?" Built on Glass 0.8.81.

- **The forecast's span.** `[weather] span`: `today`, a line as before; `hours2`, `hours3`, `hours4` or `hours6`, the next 24 hours every so many as columns, each its hour (in the clock's twelve or twenty-four), its sky by day or by night and its temperature; `week`, seven columns, each its weekday, its sky and its low and high. The columns share the forecast's cells at the forecast's size: a size the look comes with is fitted to the cells, a size the user set is kept. The hours and the days come with the player's reading from Glass 0.8.81.
- **The module sets the span for a page** as it sets the line, through `forecast_preview`.

## [0.1.44] - 2026-10-07

Just a Nerd, on the accepted clock and date: "Now - weather. Exact same principle - grid, the same way as clock and date." Built on Glass 0.8.79.

- **Today's forecast on the grid.** Where the player holds a reading for a place the user chose (Glass 0.8.79's Forecast section), the face draws it: a sky, the temperature now, then a sky, today's lowest and highest, the skies drawn as shapes by day and by night (sun, moon, cloud, fog, drizzle, rain, snow, thunder). `[weather] show`, `place` (cells of the grid as the clock's; the bottom row across the three columns unless said; empty hides it: the forecast has no place off the grid), `align`, `margin`, and its own `ink`, `opacity`, `glass` and `tint`, each the date's unless said; `[measure] weather` its height, the date's unless said. A size the look comes with is fitted to the cells, a size the user set is kept, and a forecast larger than its cells runs over the side it is aligned to, as the clock does. On the same cells as the clock or the date, aligned the same, it shares their glass, in the order clock, date, forecast.
- **The module sets the forecast for a page.** `forecast_preview` draws it from the look's keys and the player's reading, in the font the page put at `fonts/bold.ttf` under the module's home, as the face draws it on the screen.

## [0.1.43] - 2026-10-07

Just a Nerd, on the date on the grid, set in the browser's type in the likeness and in the face's on the glass: "Rejected. I requested date to be wired exactly as the clock is." and, before, "module works better like clock, why date is not a module?" Built on Glass 0.8.78.

- **The module sets a line of type for a page.** `line_preview` sets the date, or the clock in type, as a look's keys describe it, in the font the page put at `/glass/fonts/bold.ttf`, in the room its widest shape takes with the words in the middle, as the face places a line on the screen. The Manager's look panel draws the date and a clock in type through it, in the look's own font, pixel for pixel as the screen.

## [0.1.42] - 2026-10-07

Just a Nerd, on the accepted clock: "Now - wire date exactly the same way in grid. We are moving the face to grid and only." Built on Glass 0.8.75.

- **The date on the grid.** `[date] place` takes cells of the grid beside its three words (`top`, `above`, `below`), as the clock's place names them; `[date] align` is where it stands inside them and `[date] margin` its room from the sides it is aligned to, 20 unless said. A size that came with the look is fitted to the cells; a size the user set is kept; a date larger than its cells runs over the side it is aligned to, as the clock does.
- **Together on the grid.** A clock and a date on the same cells with the same alignment stand one under the other on one glass, the clock first; the date is set first and the clock takes what it leaves of the cells' height, as before the grid.
- **Nothing else moves.** A date with one of its three words stands as before; a clock with no place stands in the middle of what a date off the grid leaves, and a date on the grid leaves it the whole.

## [0.1.41] - 2026-10-07

Just a Nerd, aligning a clock larger than its row: "Up is down, down is up." Built on Glass 0.8.75.

- **A clock larger than its cells runs over on the side it is aligned to.** Before, its edge on the side named stood its margin from the cells' edge and the excess hung off the far side, so "top" moved a large clock down and "left" moved it right. Now the far edge stands the margin from the cells' far side and the excess runs over the side named: "top" always moves it up and "left" always left. A clock that fits its cells stands as before, and one in the middle runs over equally on both sides, as before.

## [0.1.40] - 2026-10-07

Just a Nerd, placing the clock in a corner of the grid and finding it twenty units short: "Perhaps a margin should be introduced?" and "meant user controlled margins!"

- **The clock's margin on the grid is the user's.** `[clock] margin`, in units of a 720th of the picture's height: the room between the clock's glass and the sides of its cells it is aligned to. 20 unless said, as before; 0 puts the glass in the corner.

## [0.1.39] - 2026-10-07

The first step of the idle screen's grid, at Just a Nerd's word: "introduce grid. We will test only clock in grid end-to-end. Move clock to grid." Built on Glass 0.8.71.

- **The clock on a grid.** Three rows by three columns in equal thirds above the bar: `top`, `middle`, `bottom` by `left`, `centre`, `right`. `[clock] place` names the cells the clock occupies, rows then columns, each a name or a range: `middle left-right` is the middle row, `middle-bottom centre-right` four cells, `top-bottom left-right` all nine. `[clock] align` is where it stands inside those cells: `left`, `centre` or `right` and `top`, `middle` or `bottom`, the middle unless said. A size that came with the look is fitted to the cells; a size the user set is kept and runs over them where it is larger.
- **Nothing else moves.** A clock with no place stands as before, in the middle of what the date and the bar leave, to the pixel. The date stands as before; placed above or below a clock that is on the grid, it has the middle to itself.

## [0.1.38] - 2026-10-06

Built on Glass 0.8.71, with what 0.8.70 and 0.8.71 bring to the display.

- **A text field takes a font of its own.** `playinfo.title.font` and `playinfo.title.fontsize` in a meter theme, and the same for artist, album and the other text fields, as the time fields have.
- **A meter stepped to in the Manager's Face tab is shown on the screen too.** A `meter.next` button pressed in the browser moves the player's screen, and every view that follows it.

## [0.1.37] - 2026-10-06

Built on Glass 0.8.69, which redraws the radio icons and takes on the FM STEREO word.

- **The radio icons as Glass 0.8.69 draws them**, and a station the radio plugin names `FM STEREO` with its signal gets the stereo icon at that level, as a mono one does.

## [0.1.36] - 2026-10-06

Asked by Just a Nerd on the screen-off: gradual, not abrupt. Built on Glass 0.8.68, whose look panel has the setting.

- **The screen goes black and comes back over a fade.** "Over, milliseconds" beside "Screen off after": 500 unless said, 0 for at once, up to 120000. On its way the face draws black over everything by how far it has come, every frame; whole, it is drawn once and stands. A tap during the fade wakes the screen as a tap on black does.
- For a face theme: `[idle] fade`, milliseconds.

## [0.1.35] - 2026-10-06

Asked by a user with an AMOLED screen, which must not show the same picture for hours. Built on Glass 0.8.67, whose look panel has the setting.

- **Screen off after a set time.** With "Screen off after, minutes" set on the Manager's Screen tab (0, the default, never), the screen glass-evo holds goes black after that many minutes with nothing playing and no touch. A tap wakes it and is the wake alone, no button under it acts; music starting wakes it too. The pixels go dark, which on an OLED is the screen off; a panel's backlight is not touched. Black costs nothing while it stands: the face draws it once.
- For a face theme: `[idle] off`, minutes.

## [0.1.34] - 2026-10-06

Built on Glass 0.8.66; nothing of the face's own changes.

- **Icons for FM in mono and by signal level for FM and DAB** (Glass 0.8.66): the radio's icon shows the signal as bars, and MONO under the letters where the station comes in mono.

## [0.1.33] - 2026-10-04

Built on Glass 0.8.65; nothing of the face's own changes.

- **The volume number of a theme takes an alignment, the italic style, words around it and a word for muted** (Glass 0.8.65): `volume.value.align`, `italic` in `volume.value.pos`, `volume.value.format` (such as `VOL {} %`) and `volume.value.mute`.

## [0.1.32] - 2026-10-04

Asked on the forum: a photograph of one's own behind the clock and the date when nothing plays. Built on Glass 0.8.63, whose Manager chooses and keeps the pictures.

- **A picture of your own when nothing plays.** Where one is chosen (the Manager's Screen tab, "Picture when nothing plays"), the screen glass-evo holds shows it in the theme's place while the player stands still, with the clock, the date and the bar of controls over it as the look has them; when music plays the theme is back. The picture is scaled to cover the screen and cut from its middle, darkened as far as "Darken the picture" says, and the glass behind the clock and the date blurs it where blur is on. It is read once, off the frame loop, and the theme shows until it is in.
- For a face theme: `[idle] picture` (a file's name in the player's `glass/backgrounds` folder) and `dim` (0 to 0.9); `themes/Example/face.txt` documents both.
- Not yet on remote displays and in the browser views: they show the theme when nothing plays.

## [0.1.31] - 2026-10-04

Built on Glass 0.8.61; nothing of the face's own changes.

- **The Android bundle says when a later release is out** (Glass 0.8.61): the Version panel on its settings page links glass-evo's new package, and Android's installer installs it over the app with the settings kept. Not tried on a device.

## [0.1.30] - 2026-10-04

Built on Glass 0.8.60; nothing of the face's own changes.

- **A bundle remote on Windows brings itself up to date** (Glass 0.8.59): the Version panel on its settings page upgrades it from glass-evo's releases, as on Linux, the display before kept as `glass.prev.exe`. Tried under Wine, not yet on a Windows machine.
- **A picture the theme brings stands in for a folder layer** (Glass 0.8.60): a skin's own sample back cover shows for the albums that have none.

## [0.1.29] - 2026-10-04

Built on Glass 0.8.58; nothing of the face's own changes. Where glass-evo is the display, on the player's screen, on a bundle remote and in the browser views, it brings:

- **Rotation quality paces the turning** (Glass 0.8.57): a record, a reel and turning art are drawn anew Low 4, Medium 8, High 15 times a second or at the custom number, as the setting says and as PeppyMeter Screensaver did, and stand between.
- **A track of a cue sheet is of the type `cue`** (Glass 0.8.56): a theme's `cue.png` shows.

## [0.1.28] - 2026-10-04

Built on Glass 0.8.54.

- **A bundle remote on Linux brings itself up to date.** The face says where its display is released (`Overlay::origin`), so the Version panel on a bundle's settings page looks at glass-evo's latest release and upgrades to it: the archive checked against the release's checksum, the new binary tried before it takes the place, the one before kept and put back by itself if the new one cannot hold on. It follows glass-evo's releases, so a bundle stays a bundle. The mechanism is Glass's (0.8.54).

## [0.1.27] - 2026-10-04

Built on Glass 0.8.51; nothing of the face's own changes.

- **No black frame at the end of a countdown.** With the persist display set to the countdown, the screen glass-evo holds, a bundle remote and the browser views that carry the face could turn black for a frame or a few when the countdown ran out, before the theme stood again under the clock; after a plugin that stopped in the middle of a countdown the black stayed until the next play. The theme stands throughout (Glass 0.8.51).

## [0.1.26] - 2026-10-04

Built on Glass 0.8.50, where a face is told which theme is on show.

- **A meter theme may bring a look for the face.** A `face.txt` beside a theme's `meters.txt`, written as any face theme is (`themes/Example/face.txt` names every key), is read while that theme is on show. Three texts then lie one over the other: the look chosen on the Manager's Screen tab, the theme's own over it, and what the user set in the Manager over both, key for key. A theme that sets only its colours leaves the sizes, the clock and the rest as the user has them; one that sets everything draws the same on every player, but for what its user adjusted. The file is optional and a theme without one draws as before. It travels with the theme's folder, so a bundle remote and the browser views that carry the face draw the same look as the player's screen, and a theme cut to another size or packaged keeps it.
- **The look follows a change of theme or of settings under a face that goes on.** The face read its look once and kept it while it ran, which holds on the player's screen, where the display starts again at every change. A page or a remote takes the player's new configuration under the same face: the look is now read again whenever the theme on show or the face's settings are not the ones it was read for.
- From Glass 0.8.50 the Manager's look panel shows what the theme on show brings: the likeness is laid as the face lays it, and a line says that the theme brings a look of its own.

## [0.1.25] - 2026-10-04

Built on Glass 0.8.49; nothing of the face's own changes. Where glass-evo is the display, on the player's screen, on a bundle remote and in the browser views, it brings:

- **A bar dragged the way it is drawn** (Glass 0.8.48): a volume fader that moves up and down is dragged up and down, also where its box is wider than tall.
- **A theme's format icon under the type's own name** (Glass 0.8.47): `webradio.png` in a theme's `format-icons` is found, as `radio.png` is.

## [0.1.24] - 2026-10-04

Built on Glass 0.8.46; nothing of the face's own changes.

- **glass-evo takes the screen on a Raspberry Pi whose panel is on DSI, at every boot.** Whether the screen could be drawn on depended on the order the system listed its graphics cards in, which changes from boot to boot: SDL's search for the screen's card forgets the card it found when one with nothing connected is listed after it, and the face ended three starts with "kmsdrm not available" and gave the screen back to the kiosk. The display tells SDL the card now (Glass 0.8.46), and this release is the face built on it. With Glass 0.8.46 on the player an earlier glass-evo is told the card by the plugin as well.
- The face's binary answers `--probe-graphics` as Glass's does (Glass 0.8.37): whether the system can draw on a screen without an X server, step by step.

## [0.1.23] - 2026-10-04

Built on Glass 0.8.35; nothing of the face's own changes. Where glass-evo is the display, on the player's screen, on a bundle remote and in the browser views, it brings:

- **The sample rate line aligned as the theme says** (Glass 0.8.33): centred or right aligned in its box in a theme that centres or right aligns its texts, where it always stood at the left; `playinfo.samplerate.align` sets it by itself.
- **The pictures fetched for the display kept to the newest 48** (Glass 0.8.32), where every cover and fanart picture was kept without end.
- **A cover that keeps its place while the same track's next address is fetched** (Glass 0.8.34), for a player that gives one cover a new address at every state, as the Squeezelite plugin does with a Lyrion server's.

## [0.1.22] - 2026-10-04

Built on Glass 0.8.31; nothing of the face's own changes.

- **Album art from a Lyrion server shows** where glass-evo is the display: on the player's screen and on a bundle remote. The Squeezelite plugin hands a Lyrion server's covers on with no `Content-Type`, and the display took only what a server called an image; Glass 0.8.31 tells a picture by its content, and this release is the face built on it. The Face tab and Anymote need only that Glass.

## [0.1.21] - 2026-10-02

Built on Glass 0.8.28; nothing of the face's own changes.

- **The fanart slideshow carries on across a change of meter** where glass-evo is the display: on the player's screen, on a bundle remote, in the Face tab and in Anymote. With meters rotating, the artist's pictures started again at every change; Glass 0.8.26 keeps the show's place and its last advance, and this release is the face built on it.
- **A snapshot of a theme carries no face** whatever binary takes it (`--snapshot`, Glass 0.8.25), so a theme's previews are the theme alone.

## [0.1.20] - 2026-10-02

The face on a remote display on Android. Built on Glass 0.8.24.

- **A bundle for Android.** The release carries `glass-evo-<version>-android.apk`: Glass's remote app with the face in it (`bins/glass-evo-android`, the app's native side, enters the display through Glass's own Android entry with the face). It is the same app as the standalone, under the same identity and the same key, so it installs over the standalone and keeps its settings, and the standalone installs over it again; its version code is that of the Glass it is built on, as the standalone's is, and Android refuses the one built on an older Glass. The bar comes at a touch; the clock shows while the player stands still; the settings page has the choice of when. Android 7 or later, 64 bit arm, 32 bit arm or x86_64.
- With this the bundle exists for every platform the standalone remote does: Linux, Windows and Android.

## [0.1.19] - 2026-10-02

The face on a remote display on Windows. Built on Glass 0.8.23.

- **A bundle for Windows.** The release carries `glass-evo-<version>-windows-x64.zip`: the display with the face in it, the SDL2.dll it loads and Glass's own remote installer for Windows, the display and the installer scripts signed as Glass's are. Installed, it is Glass's remote for Windows with the face in it, in the standalone's place; its settings page has the choice of when the face shows. The clock reads Windows' local time (Glass 0.8.23). Windows 10 or later, 64 bit; the controls need a mouse or a touch screen. Android has the standalone remote only for now.

## [0.1.18] - 2026-10-02

The face on a remote display, on Linux. Built on Glass 0.8.22.

- **A remote display shows the face.** The release carries, beside the component the Manager installs, a bundle per architecture for a remote display (`glass-evo-<version>-<arch>.tar.gz`: x64, armv8, armv7, arm): the same binary with Glass's own remote installer. A remote installed from it is Glass's remote with the face in it, in the standalone remote's place: the same settings, cache and menu entries. It shows the clock and the date when the player stands still and the bar of controls, in the player's look, where the player's own screen shows them; its settings page has the choice of always, or never. The controls need a touch screen or a mouse on the remote. The player needs Glass 0.8.20 or later; Windows and Android have the standalone remote only for now, and Anymote for the face.
- The face says what it is called (`Overlay::name`), which a remote shows on its settings page and tells the player.

## [0.1.17] - 2026-10-02

Built on Glass 0.8.19. No change in what is shown.

- The face is laid over the picture by the same code on the player's screen and in the module the Face tab and Anymote bring (Glass's `overlay::Laid`), where the display and the browser's pipeline each had their own; the two cannot come to differ there.

## [0.1.16] - 2026-10-02

Built on Glass 0.8.15, for one fix of the display's.

- **What plays no longer changes a spectrum look's layout.** A look laid out for two channels became one area for as long as a mono recording played, in the single palette where the theme names only `palette.left` and `palette.right`. It keeps its two sides now, on the player's screen and in the module the Face tab and Anymote bring; a section that asks for a one-channel bank (`channels = 1`) draws one, as before.

## [0.1.15] - 2026-10-02

Clock faces: the clock set in type as before, or drawn.

- **Five faces** (`clock.face`): `type`, the time set in the theme's type; `seven` and `sixteen`, segments, with the unlit ones showing faintly (`clock.unlit`); `flip`, cards that fall as a number changes; `dial`, hands over a dial in four styles (`clock.dial`): `station`, bars and strong hands on a light disc with a red second hand, `numbers`, `roman`, and `plain`, marks alone. A dial has its second hand when the clock's pattern shows seconds, and a twelve hour pattern gives the segments and the cards their AM and PM (seven segments show the A and the P).
- **Colours of their own**, each the user's or the theme's where one is said: the hands (`clock.hands`), the marks and numerals (`clock.marks`), the second hand (`clock.second`, the look's accent unless said), the disc behind the hands (`clock.disc`: a colour, `none`, or the style's own), a flip clock's cards (`clock.card`); the lit segments and the digits on a card are the clock's ink. The glass behind the clock, its size, its place and the date go with any face. A drawn face is as high as `measure.clock`, a dial twice that.
- **Drawn, not pictured.** Segments, hands, marks and numerals are shapes rastered at the size wanted, as the bar's icons are: sharp at any size, no font and no file. What stands still is rastered once and kept (a character, a card, a dial with its marks), so a second costs a few copies and the hands; a falling card is drawn for under half a second and then stands.
- The browser module has `clock_preview`: the clock alone as a look's keys draw it, for the Glass Manager's look panel to show a drawn face before it is saved (Glass 0.8.12).

## [0.1.14] - 2026-10-02

The face made fit for a browser, and its module carried in the component. Nothing changes on the player's screen. Built on Glass 0.8.9.

- **The face builds for a browser as it does for a player.** It is written against Glass's `overlay` crate, the contract of a face without the display's window, where it was written against the display itself. The time of day comes with the view (`View::wall`) and is set in a pattern by the face's own reading of `strftime` (`face::when`), which a test holds against the C library's for every conversion in every flag and width; the theme folders can be told to it (`Face::with_faces`) where no launcher names them; a text is read where the display reads its own; and the cover's colours are read on a thread where there is one and at the frame where there is none.
- **`glass-evo-face.wasm`**: Glass's pipeline for a page with the face over it (`bins/glass-evo-face`, one line over `page::exports!`), built by `scripts/ship.sh` and carried in the component under `face/`, named in the manifest with its digest. A Glass Manager that knows of it serves it to the Face tab and to Anymote; one that does not leaves it in the zip.

## [0.1.13] - 2026-10-02

One change, to how large the clock and the date may be.

- **A size you set is the size you get.** The face set a clock or a date smaller than asked whenever it did not fit beside the margin and the glass's room, and never let the clock be taller than half the picture, so the upper part of the size's range did nothing: a clock could not be made to fill the screen. A size set by the user (`measure.clock`, `measure.date` among the face's settings) is now drawn as set, in the middle of its place; what does not fit on the picture runs over its edges, which is the user's to choose. The glass keeps the room designed about its words while there is that much beside them, and gives it up as they take more, the margin first, down to a sliver (6 units sideways, 4 above and below).
- **A size a look comes with is still fitted**, so a look is right on a small or an upright screen without being touched: the date as before, the clock now in the whole width and in all the height the date and the bar leave, the margin and the glass's room giving way to it. A face theme may ask for a clock of up to 720 units (it was 360) and a date of up to 360 (200).
- The frost under a glass that begins off the picture's left or top edge takes only the part on the picture.

The Glass Manager's size control and its likeness follow from Glass 0.8.8.

## [0.1.12] - 2026-10-02

Built on Glass 0.8.3, for its one fix.

- **A two-sided spectrum keeps its two sides while the player stands still.** Stopped or paused, a look laid out for two channels fell back to one area in the one-channel palette, and a waterfall filled with that palette's lowest colour, green in a look that names only its two side palettes. Glass 0.8.3 leaves the layout as the look asks through silence.

## [0.1.11] - 2026-10-02

Lighter on the player, and no clock between two tracks. Built on Glass 0.8.2. The figures are a Raspberry Pi 5 at sixty frames a second with a spectrum across the whole screen.

- **No clock and no bar at a change of track.** The player says "stop" for a moment between two tracks, and the face took that for the player standing still: the clock flashed and the bar came up for two seconds, at every track. A stop that still names a track now counts only once it has lasted five seconds, as the plugin itself waits before it believes one; a pause counts at once, and so does the end of the queue. With a track change every ten seconds: 57 percent of a core before, 46.7 after.
- **Nothing drawn costs nothing.** While music plays and the bar is away the face tells the display it has nothing to draw, and the display no longer copies the picture for it on every frame. Steady play: 52 percent of a core before, 46.5 after, the same as Glass with no face.
- **A clock that says the same is not drawn again.** The face tells the display when it would draw what is on the screen already; a standing clock or bar is drawn when it changes, not sixty times a second.
- **Standing still, fifteen frames a second.** Three seconds after the player last played or the screen was last touched the display slows down, and is back at its full rate with the next touch or the next note. Paused with the clock on the screen: 60 percent of a core before, 24 after.

## [0.1.10] - 2026-10-01

Built on Glass 0.7.99, for players whose screen is reached through an X server.

- **An X server of its own.** Where the Glass plugin brings up a plain X server for the face, as it does on an x86 player, the face treats that screen as its own: it stays on it whether or not anything plays, a touch outside a control does nothing, and it turns the picture itself.
- **No screen is a failure.** With no X server, no Wayland and no screen the kernel drives, the face no longer runs drawing to nowhere: it stops with "no screen to draw on", so the plugin can give the screen back to the kiosk.

## [0.1.9] - 2026-10-01

The component says which Glass it works with. Its manifest names the least Glass plugin it needs (`requires.glass`, 0.7.97 for this release), kept in `Cargo.toml` under `workspace.metadata.glass`; the Glass Manager installs a component only on a Glass that is at least that, as Glass names the least glass-evo it works with. Nothing changes on the screen.

## [0.1.8] - 2026-10-01

Looks. A look is a face theme chosen whole, and four ship with glass-evo beside the built-in one that follows the artwork.

- **Dark Glass, Clear, Warm and Night Drive.** Black glass whatever the artwork; no backgrounds at all, the words and the buttons on the picture itself; amber on brown; and large and high in contrast, for a glance while driving in the dark. Each is a face theme like any other, under `themes/`, and a starting point for one's own.
- **Shipped with the component.** The looks travel in the component under `themes/`, and the built-in look written out, `face.txt`, beside the manifest, so the Glass Manager shows what the face draws when nothing is said.
- **The user's themes first.** `GLASS_FACES` names the folders face themes are kept in, parted by a colon; a theme is the first of its name found, so one of the user's own stands before a shipped look of the same name.
- **A line about a theme.** `description` in a theme's `[theme]` section says in a few words what the look is, for the list it is chosen from.

The Glass Manager's look panel is drawn anew around them: a row of looks to choose from, a likeness of the screen, and the adjustments in plain words, a section for each part of the screen.

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
