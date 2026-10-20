"""bitmap-emoji.py OUT: a tiny bitmap-only colour emoji font, like Ubuntu's Noto Color Emoji (needs fontTools).

One glyph, `smile`, maps U+1F600 and is a CBDT format 17 PNG (8x8, solid red) at 16 ppem.
No glyf, CFF or CFF2 table: the only drawing is the bitmap.
"""
import io
import struct
import sys
import zlib

from fontTools.fontBuilder import FontBuilder

out = sys.argv[1]
PPEM, SIDE = 16, 8


def png(side, rgba):
    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    row = b"\x00" + bytes(rgba) * side
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", side, side, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(row * side))
        + chunk(b"IEND", b"")
    )


image = png(SIDE, (230, 30, 30, 255))
UPEM = 2048
fb = FontBuilder(UPEM, isTTF=True)
fb.setupGlyphOrder([".notdef", "smile"])
fb.setupCharacterMap({0x1F600: "smile"})
fb.setupHorizontalMetrics({".notdef": (UPEM, 0), "smile": (UPEM, 0)})
fb.setupHorizontalHeader(ascent=1900, descent=-500)
fb.setupNameTable({"familyName": "Bitmap Emoji Fixture", "styleName": "Regular"})
fb.setupOS2(sTypoAscender=1900, sTypoDescender=-500, usWinAscent=1900, usWinDescent=500)
fb.setupPost()
fb.setupMaxp()
font = fb.font

ttx = f"""<?xml version="1.0" encoding="UTF-8"?>
<ttFont>
  <CBDT>
    <header version="3.0"/>
    <strikedata index="0">
      <cbdt_bitmap_format_17 name="smile">
        <SmallGlyphMetrics>
          <height value="{SIDE}"/>
          <width value="{SIDE}"/>
          <BearingX value="0"/>
          <BearingY value="{SIDE}"/>
          <Advance value="{PPEM}"/>
        </SmallGlyphMetrics>
        <rawimagedata>{image.hex()}</rawimagedata>
      </cbdt_bitmap_format_17>
    </strikedata>
  </CBDT>
  <CBLC>
    <header version="3.0"/>
    <strike index="0">
      <bitmapSizeTable>
        <sbitLineMetrics direction="hori">
          <ascender value="{PPEM}"/><descender value="-4"/><widthMax value="{PPEM}"/>
          <caretSlopeNumerator value="0"/><caretSlopeDenominator value="0"/><caretOffset value="0"/>
          <minOriginSB value="0"/><minAdvanceSB value="0"/><maxBeforeBL value="{PPEM}"/>
          <minAfterBL value="-4"/><pad1 value="0"/><pad2 value="0"/>
        </sbitLineMetrics>
        <sbitLineMetrics direction="vert">
          <ascender value="{PPEM}"/><descender value="-4"/><widthMax value="{PPEM}"/>
          <caretSlopeNumerator value="0"/><caretSlopeDenominator value="0"/><caretOffset value="0"/>
          <minOriginSB value="0"/><minAdvanceSB value="0"/><maxBeforeBL value="{PPEM}"/>
          <minAfterBL value="-4"/><pad1 value="0"/><pad2 value="0"/>
        </sbitLineMetrics>
        <colorRef value="0"/>
        <startGlyphIndex value="1"/>
        <endGlyphIndex value="1"/>
        <ppemX value="{PPEM}"/>
        <ppemY value="{PPEM}"/>
        <bitDepth value="32"/>
        <flags value="1"/>
      </bitmapSizeTable>
      <eblc_index_sub_table_1 imageFormat="17" firstGlyphIndex="1" lastGlyphIndex="1">
        <glyphLoc id="1" name="smile"/>
      </eblc_index_sub_table_1>
    </strike>
  </CBLC>
</ttFont>
"""
font.importXML(io.StringIO(ttx))
font.save(out)
