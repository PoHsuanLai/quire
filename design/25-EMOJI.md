# 25 Emoji

Status: built 2026-09-26 on branch `animated-emoji` (quire), for review. The decision (user,
2026-09-26): the persona is not better than plain emoji, so the user's picture becomes an emoji
the user picks, and it should move, as the reference desktop's animated emoji do. Those are
proprietary; we use Google's open **Noto Animated Emoji** instead. Code, classes and assets say
**emoji**.

## 1. What this governs

`ds::AnimatedEmoji` (the picture), `ds::EmojiId` (which emoji, user data), `ds::EmojiDisc`
(none, or a tinted disc), `ds::EmojiPlayback` (awake or still), `ds::EMOJI_ATTRIBUTION`, the sheets and manifest under
`crates/ds/assets/emoji/`, and the pipeline that makes them, `tools/emoji`. Moods and wakes
are the persona's (`Mood`, `WakeStamp`, `PersonaSize`, design/24 section 4): a surface drives an
emoji exactly as it drives a persona.

## 2. Source, licence and attribution

- **Source**: Noto Animated Emoji, https://googlefonts.github.io/noto-emoji-animation/ . Each
  emoji is served as Lottie JSON and as 512 px animated WebP, GIF and AVIF from
  `https://fonts.gstatic.com/s/e/notoemoji/latest/<codepoint>/`, the URLs the site itself uses.
- **Licence: CC BY 4.0**, verified 2026-09-26 from the source site's own FAQ ("Can I use these
  animated assets commercially…?": "Animated Noto Emoji is licensed under CC BY 4.0. See the
  full license for all details.", linking https://creativecommons.org/licenses/by/4.0/legalcode).
  The full legal code is committed as `crates/ds/assets/emoji/CC-BY-4.0.txt`.
- **Attribution** (required; `ds::EMOJI_ATTRIBUTION` carries it, and a surface that shows these
  emoji puts it in its about box or credits):

  > Animated emoji: Noto Animated Emoji by Google, CC BY 4.0
  > (https://creativecommons.org/licenses/by/4.0/); frames resampled and packed.

- `crates/ds/assets/emoji/ATTRIBUTION.txt` states the same beside the files, with what we
  changed (CC BY asks that changes be indicated). `docs/licensing-references.md` has the row.
- **cargo deny**: `cargo deny check licenses` reads crate licences only. `ds`'s code stays
  MIT OR Apache-2.0; the sheets are data under their own licence, as the OFL fonts beside them
  are, so no allow entry is needed or added. The obligation (attribution, a link to the licence,
  a note of changes) is met by the files above and by consumers showing the credit line.

## 3. The curated set

42 emoji, `EmojiId::ALL` in this order (slug, codepoint): the faces a user would pick, a few
gestures and characters, and the reactions.

| Group | Emoji |
| --- | --- |
| Smiles and laughs | grinning 1f600, grinning-eyes 1f604, beaming 1f601, laughing 1f606, grin-sweat 1f605, joy 1f602, rofl 1f923, slight-smile 1f642, upside-down 1f643, blush 1f60a (**default**) |
| Warm | wink 1f609, halo 1f607, hearts 1f970, heart-eyes 1f60d, star-struck 1f929, kissing-heart 1f618, yum 1f60b, winky-tongue 1f61c, zany 1f92a, hugging 1f917, chuckling 1f92d |
| Thoughtful and cool | thinking 1f914, raised-eyebrow 1f928, smirk 1f60f, relieved 1f60c, sunglasses 1f60e, nerd 1f913, cowboy 1f920, mind-blown 1f92f |
| Sleepy | sleepy 1f62a, sleeping 1f634 |
| Party | partying 1f973 |
| Reactions | confounded 1f616, persevering 1f623, eyes 1f440 |
| Gestures | wave 1f44b, thumbs-up 1f44d, raising-hands 1f64c, victory 270c_fe0f |
| Characters | ghost 1f47b, robot 1f916, fox 1f98a |

Reactions a mood swaps in: `EmojiId::WRONG` = confounded (a wrong password), `UNLOCKED` =
partying, `ASLEEP` = sleeping. `ATTENTIVE` = eyes is in the set, but the Attentive mood keeps
the user's own emoji and plays it (the brief). `EmojiId::default()` is blush, a smiling face
that is neither reaction. Not every Noto emoji is animated upstream (881 are, 2026-09-26); the
set was checked against the site's `data/api.json`.

## 4. The pipeline

Blitz cannot play Lottie, and it decodes one frame of a GIF or WebP (verified: `blitz-dom`'s
`ImageHandler::parse` calls `image::ImageReader::decode`, which returns the first frame; the
`image` crate's `gif` and `webp` features are on, but no frame sequence is kept). So quire plays
pre-rendered frames itself.

`tools/emoji` (Python through uv, `pillow` only; never run by cargo):

```bash
cd tools/emoji && uv run emojitool build --out ../../crates/ds/assets/emoji
```

1. **Fetch** each emoji's `512.webp` (the animation) and `512.png` (the still, used only to find
   the rest pose; never shipped) into `tools/emoji/.cache/` (gitignored). The manifest records
   each source URL and sha256.
2. **Decode** every WebP frame composited to RGBA; the per-frame durations come from the ANMF
   chunks (Pillow does not report them for WebP). The WebPs are 30 ms frames with long holds.
3. **Rest pose**: the frame closest to the still emoji (each cropped to its own bounds and
   compared at 64 px, since the still and the animation are drawn at different scales), or a
   frame chosen by eye where the closest match is the wrong pose (`curated.py`: wink 34, where
   the wink is; kissing-heart 37, the kiss). The loop is rotated to begin there; the loops are seamless, so this is the same animation starting at
   rest, and **frame 0 is the static emoji**. (The Lottie files mark the same pose as a `rest`
   marker, but their timeline does not match the WebP's, so the still is the reference.)
4. **Resample** every 80 ms (12.5 fps); a run of samples on one source frame becomes one held
   frame, so holds stay holds and cost one frame.
5. **Pack** each emoji into one sheet per size: frames 128 px and 256 px, in a grid 8 frames wide
   (a 256 px sheet is 2048 px wide, inside the renderer's 4096 px texture limit that one long
   strip of 30 frames would break), quantised to one shared 256-colour palette with alpha,
   PNG-optimised.
6. **Manifest** `manifest.json`: per emoji its slug, codepoint, name, grid (`columns`, `rows`),
   `durations_ms` per frame, each size's file and bytes, and the source URL, sha256, frame count
   and rest frame; plus the licence and attribution.

The run is reproducible: a rebuild from the cache writes identical bytes.

## 5. Behaviour

The caller sets the mood; the emoji plays it. Mounting, a new `wake: WakeStamp`, every mood
change and a new pick **wake** it. Each wake computes a script (pure, `script.rs`) and a task
owned by the component plays it (`life.rs`, the persona's timer machinery):

| Mood | Shows | On change |
| --- | --- | --- |
| Idle | the pick, looping for the awake window, then its rest frame | none |
| Attentive | the pick, playing | none |
| Wince | the pick | `EmojiId::WRONG` once through (1.84 s), then the pick, playing |
| Happy | the pick | `EmojiId::UNLOCKED` once through, then the pick, playing |
| Asleep | `EmojiId::ASLEEP`'s rest frame, still | nothing plays |

Nothing plays twice for one mood change (a reaction does not escalate, as design/24 section 5).
Reduced motion (and `EmojiPlayback::Still`): only still frames; a reaction's rest frame is shown for as long as its loop
would have played, then the pick's.

## 6. Frame budget and the idle rule

- **Frames**: 9 to 38 per emoji (mean 22), loops 0.72 to 3.12 s; one frame change is one
  re-render of one element whose only changed property is `background-position` (the sheet's
  `background-image` is set once as its own style property, so Blitz neither re-parses the URI
  nor reloads the image: it reloads a background only when the URL differs).
- **Bytes**: 7,670,106 bytes of sheets in all (128 px 2,195,168; 256 px 5,474,938), plus a
  30 KB manifest, compiled into `ds` with `include_bytes!` and base64-encoded once per sheet on
  first use. A 66 ms step (15 fps) measured 9.16 MB, over the 8 MB budget, so the step is 80 ms;
  both sizes are kept so the 128 px picture has 2x pixels.
- **Idle rule** (design/CHECKLIST "Idle surface paints 0 frames"): the script plays whole loops
  only while they fit inside the 20 s window (`--t-awake`), so it ends on frame 0 with no jump,
  and then the task has ended: nothing is scheduled, no CSS animation or transition exists, and
  the document asks for no frame. Tested in `ds-native/tests/emoji_life.rs`: frames advance
  within 2 s of the wake and are still advancing at 9 s; at 21 s the frame is 0, stays 0 for a
  further second, and `Harness::is_animating()` is false. `emoji/tests.rs` checks every script
  of every emoji in every mood ends on frame 0 within the window.
- Loops are the one bounded exception to "nothing loops", the same exception design/24 section 5
  gives the persona's breathing and blinks.

## 7. Built (quire)

- `crates/ds/src/components/emoji/`: `id.rs` (`EmojiId`), `sheet.rs` (sheets, manifest, the
  frame position), `script.rs` (the wake script), `life.rs` (playing it), `disc.rs`
  (`EmojiDisc`), `mod.rs` (`AnimatedEmoji`, `EMOJI_ATTRIBUTION`); `emoji.css`.
- Markup: `div.ds-emoji[data-size][data-mood][data-disc]` with `--em-disc` inline when tinted,
  holding `div.ds-emoji-face[data-emoji][data-frame]` whose `background-size` is the grid in
  shares (`800% 400%`) and whose `background-position` is the frame in shares, so one sheet fits
  every size.
- The disc: none, or `Tinted(Backdrop)` on the icon palette's eight hues (design/08 section 2.10;
  OKLCh .91/.055 light, .40/.06 dark), the face inset 14 %.
- `EmojiPlayback::{Awake, Still}` (`data-playback`): `Still` shows rest frames only, as
  Reduced motion does, for a picker or any grid of many emoji (42 loops at once is motion
  nobody asked for).
- Gallery page **Emoji** (`--page emoji`): the set at Large, the reactions, one pick in every
  mood, the sizes and the discs, and the credit line; every picture there is `Still`, so the
  snapshot shows rest frames (a snapshot lets 120 ms of timers run at mount).

## 8. Integration and open decisions

1. `UserPicture::Persona` becomes `UserPicture::Emoji(EmojiId)` once the `user-picture` branch
   lands (a separate change; nothing in the lock or picture files changed here). The persona
   code stays until then.
2. **Settings key**: `account.emoji` (an `EmojiId`, default blush) and `account.emoji_disc`
   (an `EmojiDisc`, default none); not in design/22 yet.
3. **Frame rate**: 12.5 fps to fit 8 MB. Smoother means fewer emoji, 128 px only, or a larger
   budget: the user's call.
4. **Credits surface**: where the attribution line is shown (the about box of Settings is the
   proposal).
5. **Quantisation**: the shared 256-colour palette shows faint banding on the faces' radial
   gradients at 256 px (visible side by side, not at arm's length). Dithering or 128 px only would
   trade it against bytes; not tried.
6. The first swap to a reaction loads a sheet the picture has not shown yet; on a slow machine
   the first reaction frame may land a frame late (not measured).
