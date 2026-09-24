"""The style brief (design/08-ICONS.md 3.3), the bake-off subjects (3.4) and their families (2.3).

Written once; only the subject and the palette change between prompts. Never mention the
platform names or the words for the thing we are making (08 3.3, settled).
"""

from __future__ import annotations

from dataclasses import dataclass

BRIEF = (
    "A clean, simple 3D illustration of a single {subject}, one object only, not a photograph. "
    "Centred, seen from a slight three-quarter front angle, filling most of the frame with even "
    "space around it. Clean, simple, friendly shapes with softly rounded edges, smooth matte and "
    "lightly glossy materials, no fine texture. Soft diffuse studio light from the top left, one "
    "gentle contact shadow under the object. Colours: {palette}, with white and warm off-white "
    "accents. Plain flat light grey background, no floor line, no horizon. No text, no letters, "
    "no numbers, no logo, no frame, no border, no badge, no rounded square behind the object, no "
    "outline stroke, no people, no hands."
)

NEGATIVE = (
    "text, watermark, signature, frame, border, rounded square, app tile, background pattern, "
    "photograph, realistic skin, busy detail, multiple objects, cropped object"
)

PALETTE = {
    "red": "tomato red and deep brick red",
    "amber": "warm amber and honey",
    "green": "fresh leaf green and deep forest green",
    "blue": "bright cobalt blue and deep ultramarine",
    "violet": "soft violet and deep indigo-violet",
}


@dataclass(frozen=True)
class Subject:
    """One bake-off subject: its slug, the words for the object, its plate, its object palette."""

    slug: str
    words: str
    plate: str
    palette: str

    def prompt(self) -> str:
        return BRIEF.format(subject=self.words, palette=PALETTE[self.palette])


# 08 3.4 names Mail, Files, Terminal, Notes, Photos. The plate family per subject and the
# neighbouring object palette (08 3.3: the object contrasts with its plate) are this run's
# proposal, decided per icon at review.
SUBJECTS = (
    Subject("mail", "closed paper mail envelope with a folded flap", "blue", "amber"),
    Subject("files", "document folder with a rounded tab, slightly open", "amber", "blue"),
    Subject(
        "terminal",
        "small computer screen block showing only a glowing prompt caret and a cursor bar, "
        "no letters",
        "violet",
        "green",
    ),
    Subject("notes", "square sticky note pad, top sheet slightly curled", "green", "amber"),
    Subject("photos", "small instant camera with a round lens", "red", "blue"),
)

SEEDS = (11, 22, 33, 44)

# Round two (2026-09-24): the user found round one "too realistic" and asked for something more
# abstract that speaks the design language (design/00 section 3, design/07 section 3): paper and
# ink, one weight, round caps, colour only where it means something. The object now lives in two
# tones of its plate family plus paper white and ink, so it reads as a cut-out laid on the plate,
# not a thing photographed in front of it. The ground is a neutral mid grey so paper white and
# ink both key cleanly (round one's light grey ate white rims).
FLAT_BRIEF = (
    "A flat abstract geometric emblem: {subject}. Built from a few simple paper cut-out layers "
    "with crisp clean edges, like shapes cut from coloured card and laid flat on top of each "
    "other. Only four flat colours: {palette}, paper white, and near-black ink. Thick uniform "
    "lines with round caps and rounded joins, one line weight everywhere. Completely flat solid "
    "fills, no lighting, no shading, no gradients, no shadows, no highlights, no gloss, no "
    "texture, no depth, no perspective, straight-on front view. Centred, compact, with generous "
    "empty space around it. Plain flat solid mid grey background. No text, no letters, no "
    "numbers, no logo, no frame, no border, no rounded square behind it, no people, no hands."
)

# The style-trigger variation: the same brief with a leading flat-illustration trigger, to see
# whether either model has a stronger flat mode behind a style word.
FLAT_TRIGGER = "Flat vector illustration, minimalist, Swiss graphic design style. "

FLAT_NEGATIVE = (
    "3D render, realistic, photo, glossy, shading, bevel, depth of field, gradient, shadow, "
    "perspective, texture, text, letters, watermark, frame, border, rounded square, app tile, "
    "busy detail, multiple objects, cropped"
)

# The family's two tones in words and hex (08 2.3 base and deep); models follow the words.
FLAT_PALETTE = {
    "red": "tomato red (#E8483C) and deep brick red (#B7352B)",
    "amber": "warm amber (#F0A81E) and deep ochre brown (#8E5A05)",
    "green": "leaf green (#28B24A) and deep forest green (#1A7A33)",
    "blue": "cobalt blue (#2B7CFF) and deep ultramarine (#0B5FE0)",
    "violet": "soft violet (#8B5CF0) and deep indigo-violet (#6B3FCC)",
}

# The same five subjects, described as arrangements of shapes rather than objects. Terminal is
# "a prompt chevron and a cursor block" (round one's "prompt caret" drew checkmarks and pointers).
FLAT_WORDS = {
    "mail": "a paper white rectangle with a folded triangular flap across its top, like a closed "
    "envelope reduced to two shapes",
    "files": "two paper folder shapes with rounded tabs, offset one behind the other",
    "terminal": "a prompt chevron and a cursor block: an ink right-pointing chevron like a greater-than "
    "sign, then a solid paper white bar, side by side on one line",
    "notes": "a square paper sheet with one lifted, folded-over corner and three short lines",
    "photos": "a circle sun above a gentle hill shape inside a rounded frame",
}

STYLES = ("3d", "flat", "flat-trigger", "emboss", "emboss-tile")


def prompt(s: Subject, style: str) -> str:
    """The positive prompt of one subject in one style; round one's brief is style "3d"."""
    if style.startswith("emboss"):
        return emboss_prompt(s.slug, style)
    if style == "3d":
        return s.prompt()
    body = FLAT_BRIEF.format(subject=FLAT_WORDS[s.slug], palette=FLAT_PALETTE[s.plate])
    return (FLAT_TRIGGER + body) if style == "flat-trigger" else body


def negative(style: str) -> str:
    if style.startswith("emboss"):
        return EMBOSS_NEGATIVE
    return NEGATIVE if style == "3d" else FLAT_NEGATIVE


def subject(slug: str) -> Subject:
    for s in SUBJECTS:
        if s.slug == slug:
            return s
    raise KeyError(f"unknown subject {slug!r}; known: {[s.slug for s in SUBJECTS]}")


def variation_prompt(target: Subject) -> str:
    """The reference-conditioned edit prompt (08 3.6): same object style as image 1, new subject."""
    return (
        f"Using the object in image 1 as the style reference (same materials, same lighting, same "
        f"rounded shapes, same camera angle, same plain light grey background), draw a single "
        f"{target.words} instead. Colours: {PALETTE[target.palette]}, with white and warm "
        f"off-white accents. One object only, centred. No text, no letters, no frame."
    )


# Round three (2026-09-24): the user's reference asks for a symbol crafted into the plate, not
# drawn on it: shallow embossing, soft internal geometry, no outlines or strokes, a plate with
# thickness whose bevel catches light in a few narrow spots, matte diffusion, and a soft two-hue
# gradient ground whose pair of hues differs per app. Two ways to ask for it:
#   emboss       the model draws the symbol pressed into a full-bleed gradient surface; the
#                whole render becomes the plate's face (tools/icons tile --mode ground)
#   emboss-tile  the model draws the whole tile on a plain ground; tools/icons keys it and
#                re-masks it with our squircle (tools/icons tile --mode tile)
EMBOSS_SYMBOL = (
    "A single abstract geometric symbol: {subject}. The symbol is crafted into the surface, not "
    "drawn on it: shallow embossed, slightly raised filled shapes in a pale tint of the surface "
    "colour, with a subtle light edge on top and a soft darker edge below, soft internal "
    "geometry. No outlines, no stroke, no edge lines, no borders around the shapes. Matte, "
    "smooth diffuse light, gentle, no sharp highlights, no glow. Straight-on front view, "
    "centred, the symbol spans about half the width with generous empty space around it. No "
    "text, no letters, no numbers, no logo, no people."
)

EMBOSS_GROUND = (
    " The whole image is one smooth matte surface, edge to edge, with a soft two-colour gradient "
    "from {start} at the top left to {end} at the bottom right. No tile, no frame, no border, "
    "no vignette."
)

EMBOSS_TILE = (
    " The symbol is crafted into a single rounded square tile with a subtly beveled edge "
    "catching light in narrow spots: a thin highlight arc along the top edge and a small soft "
    "point of light near one corner. The tile face has a soft two-colour gradient from {start} "
    "at the top left to {end} at the bottom right. The tile is centred, straight on, filling "
    "most of the frame, on a plain flat mid grey background. No shadow under the tile, no glow."
)

EMBOSS_NEGATIVE = (
    "frosted glass, glossy, outline, stroke, text, realistic, photo, 3D render, glow, letters, "
    "watermark, border lines, drop shadow, busy detail, multiple objects"
)

# Per-app ground pair (tools/icons/specs/*.toml use the same pairs).
EMBOSS_PAIR = {
    "mail": ("cobalt blue", "soft violet"),
    "files": ("warm amber yellow", "leaf green"),
    "terminal": ("deep violet", "warm amber yellow"),
    "notes": ("leaf green", "cobalt blue"),
    "photos": ("coral red", "warm amber yellow"),
}

EMBOSS_WORDS = {
    "mail": "a closed envelope reduced to a rounded rectangle with a V-shaped flap pressed into it",
    "files": "two rounded folder shapes with small tabs, one offset behind the other",
    "terminal": "a right-pointing chevron like a greater-than sign followed by a short "
    "horizontal cursor bar, side by side",
    "notes": "a rounded square sheet with one folded-down corner and two short raised bars",
    "photos": "a rounded frame holding a wide low hill across its bottom and a small sun circle "
    "in its upper right corner, off to the side",
}

def emboss_prompt(slug: str, style: str) -> str:
    start, end = EMBOSS_PAIR[slug]
    tail = EMBOSS_GROUND if style == "emboss" else EMBOSS_TILE
    return EMBOSS_SYMBOL.format(subject=EMBOSS_WORDS[slug]) + tail.format(start=start, end=end)
