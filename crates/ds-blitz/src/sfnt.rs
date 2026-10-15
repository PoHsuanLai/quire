//! Reading the table directory of an sfnt font file: which tables it lists, nothing more.

/// Whether the sfnt (or collection member `index`) lists a table `tag`.
pub(crate) fn table_listed(data: &[u8], index: u32, tag: [u8; 4]) -> bool {
    let be32 = |at: usize| -> Option<usize> {
        let bytes = data.get(at..at + 4)?;
        Some(u32::from_be_bytes(bytes.try_into().ok()?) as usize)
    };
    let start = if data.starts_with(b"ttcf") {
        be32(12 + 4 * index as usize)
    } else {
        Some(0)
    };
    let Some(start) = start else { return false };
    let Some(count) = data
        .get(start + 4..start + 6)
        .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]) as usize)
    else {
        return false;
    };
    (0..count).any(|n| {
        let at = start + 12 + 16 * n;
        data.get(at..at + 4) == Some(&tag[..])
    })
}

#[cfg(test)]
mod tests {
    use super::table_listed;

    /// An sfnt header listing `tags` and nothing else.
    fn sfnt(tags: &[&[u8; 4]]) -> Vec<u8> {
        let mut data = vec![0, 1, 0, 0];
        data.extend((tags.len() as u16).to_be_bytes());
        data.extend([0; 6]);
        for tag in tags {
            data.extend(tag.iter());
            data.extend([0; 12]);
        }
        data
    }

    #[test]
    fn a_table_directory_is_searched_for_colr() {
        let colour = sfnt(&[b"cmap", b"COLR", b"glyf"]);
        assert!(table_listed(&colour, 0, *b"COLR"));
        assert!(!table_listed(&sfnt(&[b"cmap", b"CBDT"]), 0, *b"COLR"));
        assert!(!table_listed(&[0, 1], 0, *b"COLR"));
    }
}
