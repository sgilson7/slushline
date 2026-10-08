# Third-party files in the build

One row per file that this repository did not write. No third-party file enters the repository or the build without a row here (CLAUDE.md). `crates/content/tests/licenses.rs` fails on an unlisted audio file, and `packaging/package-web.sh` refuses to package one.

| file | what it is | source | license |
|---|---|---|---|
| `analysis/art/fonts/LilitaOne-Regular.ttf` (and `OFL.txt` beside it) | Lilita One, the lettering of the store names painted in the story mode art; used when the art is drawn, and not shipped to the site | Juan Montoreano, from https://github.com/google/fonts/tree/main/ofl/lilitaone | SIL Open Font License 1.1 (`analysis/art/fonts/OFL.txt`) |
| `web/voice/oh-no.wav` | Sam Gilson saying "Oh no", for a missed cup | recorded by Sam on 2026-10-07 for this game (Voice Memos, "Applecross Ct"), trimmed and level-matched here | Sam's own recording, used in this game at his request (2026-10-07) |
| `web/voice/darn.wav` | Sam Gilson saying "Darn" like a cowboy, for the flair on a missed cup | recorded by Sam on 2026-10-07 for this game (Voice Memos, "Applecross Ct 2"), trimmed and level-matched here | Sam's own recording, used in this game at his request (2026-10-07) |
| `https://w.soundcloud.com/player/api.js` (loaded at run time, not in the repo) | SoundCloud's widget API, which `web/music.js` uses to play, pause and set the level of the soundtrack | SoundCloud | SoundCloud's Terms of Use; loaded from SoundCloud as its embed instructions direct |
| "Feel the Heat" by sunnybeatzproduction (streamed, not in the repo) | the soundtrack (Sam, 2026-10-06) | https://soundcloud.com/sunnybeatz-production/feel-the-heat-sunnybeatz, through SoundCloud's own player | streamed by SoundCloud's widget, which the uploader allows (the track's oEmbed answers with an embed); no copy is made |
