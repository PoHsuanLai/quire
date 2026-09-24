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
