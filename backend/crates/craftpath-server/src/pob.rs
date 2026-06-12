//! PathOfBuilding-PoE2 build-code import: decodes the share code
//! (URL-safe base64 over zlib-compressed XML) and extracts the parts a chat
//! assistant needs to make crafting answers build-aware: character class,
//! ascendancy, level and the equipped item texts. The XML schema is the
//! community fork's (MIT); only stable, top-level attributes are read.

use std::io::Read;

use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use flate2::read::ZlibDecoder;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PobBuild {
    pub class_name: Option<String>,
    pub ascendancy: Option<String>,
    pub level: Option<u32>,
    /// Raw item text blocks as PoB stores them (same shape as in-game copy
    /// text plus PoB metadata lines).
    pub items: Vec<String>,
}

/// Decode a PoB share code into the underlying XML document.
pub fn decode_pob_code(code: &str) -> Result<String> {
    let normalized: String = code
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '-' => '+',
            '_' => '/',
            c => c,
        })
        .collect();
    if normalized.is_empty() {
        bail!("empty build code");
    }

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&normalized)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&normalized))
        .context("invalid base64 in build code")?;

    let mut xml = String::new();
    ZlibDecoder::new(bytes.as_slice())
        .read_to_string(&mut xml)
        .context("zlib inflate failed - is this a Path of Building share code?")?;
    Ok(xml)
}

/// Decode and parse a PoB share code into the build summary.
pub fn parse_pob_build(code: &str) -> Result<PobBuild> {
    let xml = decode_pob_code(code)?;
    parse_pob_xml(&xml)
}

fn parse_pob_xml(xml: &str) -> Result<PobBuild> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut build = PobBuild {
        class_name: None,
        ascendancy: None,
        level: None,
        items: Vec::new(),
    };
    let mut saw_root = false;

    loop {
        match reader
            .read_event()
            .map_err(|e| anyhow!("XML parse error: {e}"))?
        {
            Event::Start(e) | Event::Empty(e) => {
                let name = e.name();
                let name = name.as_ref();
                if !saw_root {
                    // root element of PoB documents is "PathOfBuilding" (the
                    // PoE2 fork keeps the name)
                    if !name.starts_with(b"PathOfBuilding") {
                        bail!("not a Path of Building document (root element mismatch)");
                    }
                    saw_root = true;
                    continue;
                }
                match name {
                    b"Build" => {
                        for attr in e.attributes().flatten() {
                            let value = attr
                                .unescape_value()
                                .map_err(|e| anyhow!("bad attribute: {e}"))?
                                .to_string();
                            match attr.key.as_ref() {
                                b"className" => build.class_name = Some(value),
                                b"ascendClassName" => build.ascendancy = Some(value),
                                b"level" => build.level = value.parse().ok(),
                                _ => {}
                            }
                        }
                    }
                    b"Item" => {
                        let text = reader
                            .read_text(e.name())
                            .map_err(|e| anyhow!("bad Item element: {e}"))?
                            .trim()
                            .to_string();
                        if !text.is_empty() {
                            build.items.push(text);
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if !saw_root {
        bail!("empty XML document");
    }
    Ok(build)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn encode(xml: &str) -> String {
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(xml.as_bytes()).unwrap();
        let bytes = enc.finish().unwrap();
        base64::engine::general_purpose::STANDARD
            .encode(bytes)
            .replace('+', "-")
            .replace('/', "_")
    }

    #[test]
    fn test_roundtrip_pob_code() -> Result<()> {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PathOfBuilding2>
  <Build level="92" className="Ranger" ascendClassName="Deadeye" mainSocketGroup="1"/>
  <Items activeItemSet="1">
    <Item id="1">Rarity: RARE
Gale Fletch
Expert Composite Bow
Item Level: 82
+25% increased Physical Damage</Item>
    <Item id="2">Rarity: MAGIC
Sapphire Ring of Success</Item>
  </Items>
</PathOfBuilding2>"#;

        let build = parse_pob_build(&encode(xml))?;
        assert_eq!(build.class_name.as_deref(), Some("Ranger"));
        assert_eq!(build.ascendancy.as_deref(), Some("Deadeye"));
        assert_eq!(build.level, Some(92));
        assert_eq!(build.items.len(), 2);
        assert!(build.items[0].contains("Expert Composite Bow"));
        Ok(())
    }

    #[test]
    fn test_rejects_garbage() {
        assert!(parse_pob_build("not-base64!!!").is_err());
        assert!(parse_pob_build(&encode("<NotPob/>")).is_err());
        assert!(parse_pob_build("").is_err());
    }
}
