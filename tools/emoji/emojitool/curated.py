"""The curated set: the emoji a user can pick for their picture, plus the reactions.

Each row is (slug, codepoint as the source names it, name, and an optional rest frame). The slug is the file stem and the
serde name of `ds::EmojiId`; keep the two lists in the same order.
"""

from __future__ import annotations

from typing import NamedTuple


class Pick(NamedTuple):
    slug: str
    codepoint: str
    name: str
    # The source frame to rest on, when the closest match to the still is the wrong pose (a
    # wink whose closest frame has both eyes open): chosen by eye from a contact sheet.
    rest: int | None = None


SET: list[Pick] = [
    Pick("grinning", "1f600", "Grinning face"),
    Pick("grinning-eyes", "1f604", "Grinning face with smiling eyes"),
    Pick("beaming", "1f601", "Beaming face with smiling eyes"),
    Pick("laughing", "1f606", "Grinning squinting face"),
    Pick("grin-sweat", "1f605", "Grinning face with sweat"),
    Pick("joy", "1f602", "Face with tears of joy"),
    Pick("rofl", "1f923", "Rolling on the floor laughing"),
    Pick("slight-smile", "1f642", "Slightly smiling face"),
    Pick("upside-down", "1f643", "Upside-down face"),
    Pick("wink", "1f609", "Winking face", rest=34),
    Pick("blush", "1f60a", "Smiling face with smiling eyes"),
    Pick("halo", "1f607", "Smiling face with halo"),
    Pick("hearts", "1f970", "Smiling face with hearts"),
    Pick("heart-eyes", "1f60d", "Smiling face with heart-eyes"),
    Pick("star-struck", "1f929", "Star-struck"),
    Pick("kissing-heart", "1f618", "Face blowing a kiss", rest=37),
    Pick("yum", "1f60b", "Face savoring food"),
    Pick("winky-tongue", "1f61c", "Winking face with tongue"),
    Pick("zany", "1f92a", "Zany face"),
    Pick("hugging", "1f917", "Smiling face with open hands"),
    Pick("chuckling", "1f92d", "Face with hand over mouth"),
    Pick("thinking", "1f914", "Thinking face"),
    Pick("raised-eyebrow", "1f928", "Face with raised eyebrow"),
    Pick("smirk", "1f60f", "Smirking face"),
    Pick("relieved", "1f60c", "Relieved face"),
    Pick("sleepy", "1f62a", "Sleepy face"),
    Pick("sleeping", "1f634", "Sleeping face"),
    Pick("sunglasses", "1f60e", "Smiling face with sunglasses"),
    Pick("nerd", "1f913", "Nerd face"),
    Pick("partying", "1f973", "Partying face"),
    Pick("cowboy", "1f920", "Cowboy hat face"),
    Pick("mind-blown", "1f92f", "Exploding head"),
    Pick("confounded", "1f616", "Confounded face"),
    Pick("persevering", "1f623", "Persevering face"),
    Pick("eyes", "1f440", "Eyes"),
    Pick("wave", "1f44b", "Waving hand"),
    Pick("thumbs-up", "1f44d", "Thumbs up"),
    Pick("raising-hands", "1f64c", "Raising hands"),
    Pick("victory", "270c_fe0f", "Victory hand"),
    Pick("ghost", "1f47b", "Ghost"),
    Pick("robot", "1f916", "Robot"),
    Pick("fox", "1f98a", "Fox"),
]
