# 25 Emoji

Status: built 2026-09-26 on branch `animated-emoji` (quire), for review. The decision (user,
2026-09-26): the persona is not better than plain emoji, so the user's picture becomes an emoji
the user picks, and it should move, as the reference desktop's animated emoji do. Those are
proprietary; we use Google's open **Noto Animated Emoji** instead. Code, classes and assets say
**emoji**.

2026-09-26 (branch `user-picture-emoji`): the persona is removed (design/24 is a note now), and
the emoji is one kind of user picture beside the letter and the photo (section 7).

## 1. What this governs

`ds::AnimatedEmoji` (the picture), `ds::EmojiId` (which emoji, user data), `ds::EmojiDisc`
(none, or a tinted disc on a `DiscHue`), `ds::EmojiPlayback` (awake or still),
`ds::EMOJI_ATTRIBUTION`, the sheets and manifest under `crates/ds/assets/emoji/`, and the
pipeline that makes them, `tools/emoji`; and the user's picture (section 7): `UserPicture`,
`UserPortrait`, `PictureChoice`, `resolve_picture`, `UserPicturePicker`, `Mood`, `WakeStamp`,
`PictureSize` and the accept beat `Anim::PictureAccept`.

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
partying, `ASLEEP` = sleeping, `ATTENTIVE` = eyes (a glance when typing starts; section 5). `EmojiId::default()` is blush, a smiling face
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
owned by the component plays it (`life.rs`). The mapping (2026-09-26, the persona's moods on
the emoji; `emoji/script.rs` `reaction` and `pace`, tested row by row in `emoji/tests.rs`):

| Mood | When (lock prompt) | On change, once through | Then, inside the 20 s window | At rest |
| --- | --- | --- | --- | --- |
| Idle | nothing typed | nothing | **slow**: the pick's loop, 4 s on its rest frame (`IDLE_REST`), again | the pick, frame 0 |
| Attentive | typing, or checking | `eyes` (1.36 s): a glance at the field | **steady**: the pick, loop after loop | the pick, frame 0 |
| Wince | wrong password | `confounded` (1.84 s) | slow | the pick, frame 0 |
| Happy | accepted | `partying` (1.68 s), and the accept beat | slow | the pick, frame 0 |
| Asleep | display off | nothing | nothing | `sleeping`, frame 0 |

Idle is slow so a lock screen at rest is alive but calm; Attentive is steady because the person
is at the keyboard. A new wake stamp with the same mood (activity in the prompt) replays the
pace, never the reaction. Nothing plays twice for one mood change (a reaction does not
escalate).
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
  within 2 s of the wake and (Attentive) are still advancing at 9 s; at 21 s the frame is 0,
  stays 0 for a further second, and `Harness::is_animating()` is false; Idle rests at least
  3.8 s between loops; as a `UserPortrait`, each mood shows its emoji and is at rest with no
  frame asked for 21 s after the last wake. `emoji/tests.rs` checks every script of every emoji
  in every mood ends on frame 0 within the window.
- Loops are quire's one bounded exception to "nothing loops" (design/05 section 2): finite
  plays inside the awake window, never `infinite`.

## 7. The emoji as the user's picture (2026-09-26)

The user dropped the persona (design/24). A user's picture is one of three kinds:

```rust
pub enum UserPicture {
    Face(AvatarFace),   // the letter disc, unchanged
    Emoji(EmojiId),     // an animated emoji, in the same disc size and place
    Photo(ImageSource), // $HOME/.face or AccountsService's icon, cropped round
}
```

- **Drawing.** `UserPortrait { picture, size: PictureSize, mood, wake }`, or `LockPrompt` and
  `PolkitPrompt` through `LockUser`. Every kind sits in `div.ds-user-picture[data-mood]`, the
  wrapper that plays the accept beat; inside it the letter is the avatar exactly as before, the
  emoji is `AnimatedEmoji` bare (no disc) at the picture's size, and the photo is
  `div.ds-user-photo`. At the lock screen all three are 64 px; in the polkit sheet 48 (the emoji
  drawn at `Medium` in a 48 box).
- **Moods.** The emoji plays the section 5 mapping; the letter and the photo have no moods of
  their own. The lock prompt derives the mood from its own state (design/04 section 42) and is
  woken by activity only when the picture is an emoji, so a letter costs no render per pointer
  move. The polkit sheet shows Wince when `Wrong` and Happy when `Accepted`, otherwise Idle.
- **The accept beat** (`Anim::PictureAccept`, keyframe `picture-accept`, `--t-big
  --e-spring`: 420 ms at Standard, 300 Calm, 560 Extra, as the persona's hop it replaces): when
  the mood changes to Happy, the whole picture, of any kind, lifts 8 % and lands once; the pulse
  is taken off at its settle. A lock screen unlocks at `settle(Anim::PictureAccept, level, StaggerIndex::default())`
  (sill times it with `use_motion_timer(Anim::PictureAccept)`). Not played on mount, and not
  played at all under Reduced motion. `Anim::PersonaHop` remains one release as a deprecated
  alias.
- **Reduced motion.** The emoji shows still frames only (a reaction's rest frame for its
  length); the beat is not fired; the 20 s window and the idle rule are unchanged.

**The stored choice.** `PictureChoice::{Auto, Letter, Emoji(EmojiId), Photo}` (serde, adjacently
tagged, the emoji by its stable slug: `{"kind":"emoji","v":"heart-eyes"}`; default `Auto`).
`resolve_picture(choice, face: FaceFile, letter: AvatarFace) -> UserPicture` is pure:

| Choice | `FaceFile::Found(src)` | `FaceFile::Missing` |
| --- | --- | --- |
| Auto | Photo(src) | Face(letter) |
| Letter | Face(letter) | Face(letter) |
| Emoji(id) | Emoji(id) | Emoji(id) |
| Photo | Photo(src) | Face(letter) |

The caller (sill's auth service) reads `~/.face` or AccountsService's icon; `resolve_picture`
never touches the disk. Proposed settings row (design/22, once sill lists it):
`session.user_picture` | `PictureChoice` | `Auto` | Users page, first run.

**The picker.** `UserPicturePicker { letter, choice, onpick, columns (8), label ("Picture") }`:
the letter disc first, then the 42 as `AnimatedEmoji` still frames (`EmojiPlayback::Still`), each
a 64 px disc in a 76 px cell (`PICTURE_CELL`), one radio group (`role=radiogroup`, cells
`role=radio` with `aria-checked`), the current choice marked with the emoji grid's selection
look (`--accent-soft`). A click or an arrow key picks (the arrows follow `grid_step`, the emoji
grid's rule). `Auto` and `Photo` mark no cell: the Users page offers them as rows beside the
picker. It is not an `EmojiGrid`: that grid draws text glyphs in the colour font, one caller
value per glyph, and holds a separate selection cursor; the picker draws pictures and is a radio
group. It reuses the grid's step rule and column style.

## 8. Built (quire)

- `crates/ds/src/components/emoji/`: `id.rs` (`EmojiId`), `sheet.rs` (sheets, manifest, the
  frame position), `script.rs` (the wake script), `life.rs` (playing it), `disc.rs`
  (`EmojiDisc`), `mod.rs` (`AnimatedEmoji`, `EMOJI_ATTRIBUTION`); `emoji.css`.
- Markup: `div.ds-emoji[data-size][data-mood][data-disc]` with `--em-disc` inline when tinted,
  holding `div.ds-emoji-face[data-emoji][data-frame]` whose `background-size` is the grid in
  shares (`800% 400%`) and whose `background-position` is the frame in shares, so one sheet fits
  every size.
- The disc: none, or `Tinted(DiscHue)` on the icon palette's eight hues (design/08 section 2.10;
  OKLCh .91/.055 light, .40/.06 dark), the face inset 14 %.
- `EmojiPlayback::{Awake, Still}` (`data-playback`): `Still` shows rest frames only, as
  Reduced motion does, for a picker or any grid of many emoji (42 loops at once is motion
  nobody asked for).
- Gallery page **Emoji** (`--page emoji`): the set at Large, the reactions, one pick in every
  mood, the sizes and the discs, and the credit line; every picture there is `Still`, so the
  snapshot shows rest frames (a snapshot lets 120 ms of timers run at mount).

- The user's picture (section 7): `crates/ds/src/components/user_picture/`: `picture.rs`
  (`UserPicture`), `mood.rs` (`Mood`, `PictureSize`, `WakeStamp`), `portrait.rs`
  (`UserPortrait` and the shared drawing), `accept.rs` (the accept beat), `choice.rs`
  (`PictureChoice`, `FaceFile`, `resolve_picture`), `picker.rs` (`UserPicturePicker`);
  `user_picture.css`. Goldens: `tests/snapshots/user_picture/` (letter, emoji at rest, photo,
  picker) and `tests/snapshots/lock_switcher/*-emoji*.html`; harness: `emoji_life.rs` and
  `lock_switcher.rs`. The picker is on the gallery's Emoji page, the three kinds in the
  prompt on Lock and switcher.

## 9. Integration and open decisions

1. Done 2026-09-26: `UserPicture::Emoji(EmojiId)` replaced `UserPicture::Persona` (section 7).
2. **Settings key**: proposed `session.user_picture`, a `PictureChoice`, default `Auto`
   (section 7); not in design/22 yet (sill lists its keys first). It supersedes the earlier
   `account.emoji` proposal; a tinted disc is not offered in the picture yet.
3. **Frame rate**: 12.5 fps to fit 8 MB. Smoother means fewer emoji, 128 px only, or a larger
   budget: the user's call.
4. **Credits surface**: where the attribution line is shown (the about box of Settings is the
   proposal).
5. **Quantisation**: the shared 256-colour palette shows faint banding on the faces' radial
   gradients at 256 px (visible side by side, not at arm's length). Dithering or 128 px only would
   trade it against bytes; not tried.
6. The first swap to a reaction loads a sheet the picture has not shown yet; on a slow machine
   the first reaction frame may land a frame late (not measured).
