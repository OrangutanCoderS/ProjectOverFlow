#![forbid(unsafe_code)]
use memchr::memmem;

/// Hex pattern with optional wildcards ("??")
#[derive(Debug, Clone)]
pub struct Signature {
    pub raw: String,          // e.g., "48 8B ?? 89"
    pub bytes: Vec<Option<u8>>// Some(byte) or None for wildcard
}

impl Signature {
    pub fn parse(hex_with_wildcards: &str) -> Result<Self, String> {
        let mut bytes = Vec::new();
        for tok in hex_with_wildcards.split_whitespace() {
            if tok == "??" { bytes.push(None); continue; }
            if tok.len() != 2 { return Err(format!("bad token '{}'", tok)); }
            let b = u8::from_str_radix(tok, 16).map_err(|e| format!("bad hex '{}': {e}", tok))?;
            bytes.push(Some(b));
        }
        Ok(Self { raw: hex_with_wildcards.to_string(), bytes })
    }

    pub fn len(&self) -> usize { self.bytes.len() }
}

/// Find all matches of signature (wildcards supported) in haystack.
/// Optimized by anchoring with first non-wildcard byte sequence if available.
pub fn find_all(hay: &[u8], sig: &Signature) -> Vec<usize> {
    if sig.bytes.is_empty() { return vec![]; }

    // Build the longest contiguous anchor (no wildcards)
    let mut best_anchor: Option<(usize, Vec<u8>)> = None;
    let mut i = 0;
    while i < sig.bytes.len() {
        if let Some(b) = sig.bytes[i] {
            let start = i;
            let mut buf = vec![b];
            i += 1;
            while i < sig.bytes.len() {
                match sig.bytes[i] {
                    Some(x) => { buf.push(x); i += 1; }
                    None => break
                }
            }
            // choose longest
            if best_anchor.as_ref().map_or(true, |(_, a)| buf.len() > a.len()) {
                best_anchor = Some((start, buf));
            }
        } else {
            i += 1;
        }
    }

    let mut hits = Vec::new();
    if let Some((anchor_off, anchor_bytes)) = best_anchor {
        for pos in memmem::find_iter(hay, &anchor_bytes) {
            // candidate start of signature = pos - anchor_off, if not underflow
            if pos < anchor_off { continue; }
            let start = pos - anchor_off;
            if start + sig.len() > hay.len() { continue; }
            if matches_sig(&hay[start..start + sig.len()], sig) {
                hits.push(start);
            }
        }
    } else {
        // all wildcards — every position of this length matches
        if sig.len() <= hay.len() {
            for start in 0..=hay.len() - sig.len() {
                hits.push(start);
            }
        }
    }
    hits
}

fn matches_sig(win: &[u8], sig: &Signature) -> bool {
    debug_assert_eq!(win.len(), sig.len());
    for (i, sb) in sig.bytes.iter().enumerate() {
        if let Some(b) = sb {
            if *b != win[i] { return false; }
        }
    }
    true
}