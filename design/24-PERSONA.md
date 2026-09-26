# 24 Persona (dropped)

Status: **dropped 2026-09-26 in favour of emoji** (user decision). The user's picture is a
letter disc, an animated emoji or their photo: design/25-EMOJI.md, section 7 for the picture
and section 5 for its moods.

What was here: the user's own animated character, drawn from flat shapes (people, bears, cats,
bunnies and blobs from a `PersonaSpec`), with five moods played by five keyframes
(`persona-blink`, `-breathe`, `-wince`, `-hop`, `-drift`). The user found it no better than plain
emoji. Removed from quire on branch `user-picture-emoji`: the `Persona` component and its CSS,
`PersonaSpec` and its parts, `PersonaFinish`, `PersonaSize` (now `PictureSize`), the persona's
tests, goldens and gallery page, `--persona-frames`, the five `Anim::Persona*` entries and the
`--t-drift` token. The persona's hop became every picture's accept beat, `Anim::PictureAccept`
(design/25 section 7); `Anim::PersonaHop` is kept for one release as a deprecated alias of it.

What carried over: `Mood` and its five values, `WakeStamp`, the 20 s awake window
(`--t-awake`) and the rule that a picture at rest paints 0 frames; the emoji plays them now.
The progress page's archive keeps the persona's screenshots as a record of the decision.
