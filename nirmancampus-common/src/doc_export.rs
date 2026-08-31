//! Document export helpers.
//!
//! DOCX: pandoc markdown → docx, then:
//!   1) rewrite OOXML theme fonts (Windows Mangal/Shruti/… → Noto),
//!   2) set explicit style + per-run fonts by Unicode script so LibreOffice
//!      does not keep complex-script text on Liberation/Aptos (which drops glyphs).
//! PDF: same prepared docx → LibreOffice headless, with a fontconfig alias file
//! so Mangal/etc. still resolve to Noto if anything still asks for them.

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::process::Command;
use zip::ZipArchive;
use zip::write::SimpleFileOptions;

fn work_dir() -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nirmancampus-export-{nanos}"));
    std::fs::create_dir_all(&dir).map_err(|e| format!("temp dir: {e}"))?;
    Ok(dir)
}

fn file_uri(path: &Path) -> Result<String, String> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let s = abs.to_string_lossy().replace('\\', "/");
    if s.starts_with('/') {
        Ok(format!("file://{s}"))
    } else {
        Ok(format!("file:///{s}"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    Latin,
    Devanagari,
    Gujarati,
    Telugu,
    Gurmukhi,
    Bengali,
    Tamil,
    Kannada,
    Malayalam,
    Other,
}

fn script_of(c: char) -> Option<Script> {
    // None = common (whitespace / punctuation) — inherit neighbours.
    if c.is_whitespace() {
        return None;
    }
    let u = c as u32;
    Some(match u {
        0x0900..=0x097F | 0x1CD0..=0x1CFF | 0xA8E0..=0xA8FF => Script::Devanagari,
        0x0A80..=0x0AFF => Script::Gujarati,
        0x0C00..=0x0C7F => Script::Telugu,
        0x0A00..=0x0A7F => Script::Gurmukhi,
        0x0980..=0x09FF => Script::Bengali,
        0x0B80..=0x0BFF => Script::Tamil,
        0x0C80..=0x0CFF => Script::Kannada,
        0x0D00..=0x0D7F => Script::Malayalam,
        0x0000..=0x024F | 0x1E00..=0x1EFF => Script::Latin,
        _ if c.is_ascii() => Script::Latin,
        _ => Script::Other,
    })
}

fn font_for(script: Script) -> &'static str {
    match script {
        Script::Devanagari => "Noto Sans Devanagari",
        Script::Gujarati => "Noto Sans Gujarati",
        Script::Telugu => "Noto Sans Telugu",
        Script::Gurmukhi => "Noto Sans Gurmukhi",
        Script::Bengali => "Noto Sans Bengali",
        Script::Tamil => "Noto Sans Tamil",
        Script::Kannada => "Noto Sans Kannada",
        Script::Malayalam => "Noto Sans Malayalam",
        Script::Latin | Script::Other => "Liberation Sans",
    }
}

fn split_by_script(text: &str) -> Vec<(Script, String)> {
    let chars: Vec<(char, Option<Script>)> = text.chars().map(|c| (c, script_of(c))).collect();
    if chars.is_empty() {
        return Vec::new();
    }

    // Forward-fill commons from previous; then back-fill from next.
    let mut scripts: Vec<Script> = vec![Script::Latin; chars.len()];
    let mut last = Script::Latin;
    for (i, (_, s)) in chars.iter().enumerate() {
        if let Some(s) = *s {
            last = s;
            scripts[i] = s;
        } else {
            scripts[i] = last;
        }
    }
    let mut next = Script::Latin;
    for i in (0..chars.len()).rev() {
        if chars[i].1.is_some() {
            next = scripts[i];
        } else {
            // Prefer following real script for leading commons in a segment.
            scripts[i] = next;
        }
    }

    let mut out: Vec<(Script, String)> = Vec::new();
    for ((c, _), s) in chars.into_iter().zip(scripts) {
        if let Some((_, buf)) = out.last_mut().filter(|(ps, _)| *ps == s) {
            buf.push(c);
        } else {
            out.push((s, c.to_string()));
        }
    }
    out
}

fn xml_escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn rfonts_xml(font: &str) -> String {
    format!(
        "<w:rFonts w:ascii=\"{font}\" w:hAnsi=\"{font}\" w:cs=\"{font}\" w:eastAsia=\"{font}\"/>"
    )
}

/// Rewrite a single `<w:r>…</w:r>` so mixed-script text becomes one run per script
/// with an explicit font LibreOffice will actually use.
fn rewrite_run_xml(run: &str) -> String {
    let t_start = match run.find("<w:t") {
        Some(i) => i,
        None => return run.to_string(),
    };
    let t_tag_end = match run[t_start..].find('>') {
        Some(i) => t_start + i + 1,
        None => return run.to_string(),
    };
    let t_close = match run[t_tag_end..].find("</w:t>") {
        Some(i) => t_tag_end + i,
        None => return run.to_string(),
    };
    let text = &run[t_tag_end..t_close];
    // Decode minimal entities pandoc emits.
    let decoded = text
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    let parts = split_by_script(&decoded);
    if parts.len() <= 1 {
        let font = font_for(parts.first().map(|(s, _)| *s).unwrap_or(Script::Latin));
        return ensure_run_font(run, font);
    }

    // Preserve rPr (bold/italic/…) without old rFonts.
    let rpr = if let Some(start) = run.find("<w:rPr>") {
        if let Some(end) = run.find("</w:rPr>") {
            let inner = &run[start + "<w:rPr>".len()..end];
            let cleaned = strip_rfonts(inner);
            Some(cleaned)
        } else {
            None
        }
    } else {
        None
    };

    let space_attr = if run[t_start..t_tag_end].contains("xml:space") {
        " xml:space=\"preserve\""
    } else {
        ""
    };

    let mut out = String::new();
    for (script, chunk) in parts {
        if chunk.is_empty() {
            continue;
        }
        let font = font_for(script);
        out.push_str("<w:r>");
        out.push_str("<w:rPr>");
        out.push_str(&rfonts_xml(font));
        if let Some(ref extra) = rpr {
            out.push_str(extra);
            // Complex-script bold/italic twins when western ones are present.
            if extra.contains("<w:b") && !extra.contains("<w:bCs") {
                out.push_str("<w:bCs/>");
            }
            if extra.contains("<w:i") && !extra.contains("<w:iCs") {
                out.push_str("<w:iCs/>");
            }
        }
        out.push_str("</w:rPr>");
        out.push_str("<w:t");
        out.push_str(space_attr);
        out.push('>');
        out.push_str(&xml_escape_text(&chunk));
        out.push_str("</w:t></w:r>");
    }
    out
}

fn strip_rfonts(rpr_inner: &str) -> String {
    let mut out = String::with_capacity(rpr_inner.len());
    let mut rest = rpr_inner;
    while let Some(start) = rest.find("<w:rFonts") {
        out.push_str(&rest[..start]);
        if let Some(end) = rest[start..].find("/>") {
            rest = &rest[start + end + 2..];
        } else if let Some(end) = rest[start..].find("</w:rFonts>") {
            rest = &rest[start + end + "</w:rFonts>".len()..];
        } else {
            break;
        }
    }
    out.push_str(rest);
    out
}

fn ensure_run_font(run: &str, font: &str) -> String {
    let fonts = rfonts_xml(font);
    if let Some(start) = run.find("<w:rPr>") {
        let after = start + "<w:rPr>".len();
        if let Some(rel_end) = run[after..].find("</w:rPr>") {
            let end = after + rel_end;
            let inner = strip_rfonts(&run[after..end]);
            let mut out = String::new();
            out.push_str(&run[..after]);
            out.push_str(&fonts);
            out.push_str(&inner);
            if inner.contains("<w:b") && !inner.contains("<w:bCs") {
                out.push_str("<w:bCs/>");
            }
            if inner.contains("<w:i") && !inner.contains("<w:iCs") {
                out.push_str("<w:iCs/>");
            }
            out.push_str(&run[end..]);
            return out;
        }
    }
    // Insert rPr after <w:r>
    if let Some(pos) = run.find("<w:r>") {
        let at = pos + "<w:r>".len();
        format!(
            "{}<w:rPr>{}</w:rPr>{}",
            &run[..at],
            fonts,
            &run[at..]
        )
    } else if let Some(pos) = run.find("<w:r ") {
        if let Some(end) = run[pos..].find('>') {
            let at = pos + end + 1;
            format!(
                "{}<w:rPr>{}</w:rPr>{}",
                &run[..at],
                fonts,
                &run[at..]
            )
        } else {
            run.to_string()
        }
    } else {
        run.to_string()
    }
}

fn rewrite_document_fonts(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len() + 64);
    let mut rest = xml;
    const RUN_PREFIX: &str = "<w:r";
    while let Some(start) = rest.find(RUN_PREFIX) {
        // Avoid matching <w:rPr>, <w:rFonts>, <w:rStyle>, …
        let after = &rest[start + RUN_PREFIX.len()..];
        let ok = after.starts_with('>') || after.starts_with(' ') || after.starts_with('\n');
        if !ok {
            out.push_str(&rest[..start + RUN_PREFIX.len()]);
            rest = &rest[start + RUN_PREFIX.len()..];
            continue;
        }
        out.push_str(&rest[..start]);
        let body = &rest[start..];
        let end = match find_run_end(body) {
            Some(e) => e,
            None => {
                out.push_str(body);
                return out;
            }
        };
        let run = &body[..end];
        out.push_str(&rewrite_run_xml(run));
        rest = &body[end..];
    }
    out.push_str(rest);
    out
}

fn find_run_end(run_start: &str) -> Option<usize> {
    // run_start begins with <w:r …> or <w:r>
    let open_end = run_start.find('>')? + 1;
    let mut depth = 1usize;
    let mut i = open_end;
    let bytes = run_start.as_bytes();
    while i < run_start.len() {
        if run_start[i..].starts_with("</w:r>") {
            depth -= 1;
            if depth == 0 {
                return Some(i + "</w:r>".len());
            }
            i += "</w:r>".len();
            continue;
        }
        if run_start[i..].starts_with("<w:r>") || run_start[i..].starts_with("<w:r ") {
            // nested run — uncommon; still track
            depth += 1;
            i += 4;
            continue;
        }
        // advance one char safely
        let next = run_start[i..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let _ = bytes; // silence
        i += next;
    }
    None
}

fn rewrite_theme_fonts(theme_xml: &str) -> String {
    const REPLACEMENTS: &[(&str, &str)] = &[
        ("Mangal", "Noto Sans Devanagari"),
        ("Shruti", "Noto Sans Gujarati"),
        ("Gautami", "Noto Sans Telugu"),
        ("Raavi", "Noto Sans Gurmukhi"),
        ("Vrinda", "Noto Sans Bengali"),
        ("Latha", "Noto Sans Tamil"),
        ("Tunga", "Noto Sans Kannada"),
        ("Kartika", "Noto Sans Malayalam"),
        ("Kalinga", "Noto Sans Oriya"),
        ("Iskoola Pota", "Noto Sans Sinhala"),
        ("Nirmala UI", "Noto Sans"),
        ("Leelawadee UI", "Noto Sans"),
        ("Ebrima", "Noto Sans"),
        ("Nyala", "Noto Sans Ethiopic"),
        ("Myanmar Text", "Noto Sans Myanmar"),
        ("Microsoft Himalaya", "Noto Serif Tibetan"),
        ("Angsana New", "Noto Sans Thai"),
        ("Cordia New", "Noto Sans Thai"),
        ("DokChampa", "Noto Sans Lao"),
        ("DaunPenh", "Noto Sans Khmer"),
        ("MoolBoran", "Noto Sans Khmer"),
        ("Estrangelo Edessa", "Noto Sans Syriac"),
        ("MV Boli", "Noto Sans Thaana"),
        ("Plantagenet Cherokee", "Noto Sans Cherokee"),
        ("Euphemia", "Noto Sans Canadian Aboriginal"),
        ("Microsoft Yi Baiti", "Noto Sans Yi"),
        ("Mongolian Baiti", "Noto Sans Mongolian"),
        ("Microsoft Uighur", "Noto Sans"),
        ("Microsoft Tai Le", "Noto Sans Tai Le"),
        ("Microsoft New Tai Lue", "Noto Sans Tai Lue"),
        ("Phagspa", "Noto Sans"),
        ("Javanese Text", "Noto Sans Javanese"),
        ("Sylfaen", "Noto Sans"),
        ("Aptos Display", "Liberation Sans"),
        ("Aptos", "Liberation Sans"),
    ];

    let mut out = theme_xml.to_string();
    for (from, to) in REPLACEMENTS {
        out = out.replace(&format!("typeface=\"{from}\""), &format!("typeface=\"{to}\""));
    }
    out = out.replace(
        "<a:cs typeface=\"\"/>",
        "<a:cs typeface=\"Noto Sans Devanagari\"/>",
    );
    out
}

fn rewrite_styles_fonts(styles_xml: &str) -> String {
    // Default western face Liberation Sans; complex-script face Noto Sans Devanagari.
    let replacement = "<w:rFonts w:ascii=\"Liberation Sans\" w:hAnsi=\"Liberation Sans\" w:cs=\"Noto Sans Devanagari\" w:eastAsia=\"Noto Sans Devanagari\"/>";
    let mut out = String::new();
    let mut rest = styles_xml;
    while let Some(start) = rest.find("<w:rFonts") {
        out.push_str(&rest[..start]);
        if let Some(end) = rest[start..].find("/>") {
            out.push_str(replacement);
            rest = &rest[start + end + 2..];
        } else {
            out.push_str(&rest[start..]);
            return out.replace("w:bidi=\"ar-SA\"", "w:bidi=\"hi-IN\"");
        }
    }
    out.push_str(rest);
    out.replace("w:bidi=\"ar-SA\"", "w:bidi=\"hi-IN\"")
}

fn prepare_docx(docx: &[u8]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(docx);
    let mut archive =
        ZipArchive::new(cursor).map_err(|e| format!("docx zip open: {e}"))?;

    let mut out_buf = Cursor::new(Vec::with_capacity(docx.len().saturating_mul(2)));
    {
        let mut writer = zip::ZipWriter::new(&mut out_buf);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("docx zip entry: {e}"))?;
            let name = file.name().to_string();

            if file.is_dir() {
                writer
                    .add_directory(&name, options)
                    .map_err(|e| format!("docx zip dir: {e}"))?;
                continue;
            }

            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| format!("docx zip read {name}: {e}"))?;

            if name == "word/theme/theme1.xml" {
                let xml = String::from_utf8_lossy(&data);
                data = rewrite_theme_fonts(&xml).into_bytes();
            } else if name == "word/styles.xml" {
                let xml = String::from_utf8_lossy(&data);
                data = rewrite_styles_fonts(&xml).into_bytes();
            } else if name == "word/document.xml"
                || name == "word/footnotes.xml"
                || name == "word/comments.xml"
            {
                let xml = String::from_utf8_lossy(&data);
                data = rewrite_document_fonts(&xml).into_bytes();
            }

            writer
                .start_file(&name, options)
                .map_err(|e| format!("docx zip start {name}: {e}"))?;
            writer
                .write_all(&data)
                .map_err(|e| format!("docx zip write {name}: {e}"))?;
        }

        writer
            .finish()
            .map_err(|e| format!("docx zip finish: {e}"))?;
    }

    Ok(out_buf.into_inner())
}

fn write_fontconfig_aliases(dir: &Path) -> Result<PathBuf, String> {
    let path = dir.join("fonts.conf");
    // Strong aliases so any leftover Windows theme names still resolve to Noto.
    let conf = r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <include ignore_missing="yes">/etc/fonts/fonts.conf</include>
  <alias binding="strong"><family>Mangal</family><prefer><family>Noto Sans Devanagari</family></prefer></alias>
  <alias binding="strong"><family>Shruti</family><prefer><family>Noto Sans Gujarati</family></prefer></alias>
  <alias binding="strong"><family>Gautami</family><prefer><family>Noto Sans Telugu</family></prefer></alias>
  <alias binding="strong"><family>Raavi</family><prefer><family>Noto Sans Gurmukhi</family></prefer></alias>
  <alias binding="strong"><family>Vrinda</family><prefer><family>Noto Sans Bengali</family></prefer></alias>
  <alias binding="strong"><family>Latha</family><prefer><family>Noto Sans Tamil</family></prefer></alias>
  <alias binding="strong"><family>Tunga</family><prefer><family>Noto Sans Kannada</family></prefer></alias>
  <alias binding="strong"><family>Kartika</family><prefer><family>Noto Sans Malayalam</family></prefer></alias>
  <alias binding="strong"><family>Aptos</family><prefer><family>Liberation Sans</family></prefer></alias>
  <alias binding="strong"><family>Aptos Display</family><prefer><family>Liberation Sans</family></prefer></alias>
</fontconfig>
"#;
    std::fs::write(&path, conf).map_err(|e| format!("write fonts.conf: {e}"))?;
    Ok(path)
}

async fn pandoc_docx(markdown: &str) -> Result<Vec<u8>, String> {
    let mut child = Command::new("pandoc")
        .args(["-s", "-f", "markdown", "-t", "docx", "-o", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn pandoc: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(markdown.as_bytes())
            .await
            .map_err(|e| format!("pandoc stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("pandoc wait: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pandoc failed: {stderr}"));
    }
    prepare_docx(&output.stdout)
}

async fn soffice_docx_to_pdf(docx: &[u8]) -> Result<Vec<u8>, String> {
    let dir = work_dir()?;
    let docx_path = dir.join("document.docx");
    let pdf_path = dir.join("document.pdf");
    let profile = dir.join("lo-profile");
    std::fs::write(&docx_path, docx).map_err(|e| format!("write docx: {e}"))?;
    let fc_path = write_fontconfig_aliases(&dir)?;

    let profile_uri = file_uri(&profile)?;
    let output = Command::new("soffice")
        .env("FONTCONFIG_FILE", &fc_path)
        .arg(format!("-env:UserInstallation={profile_uri}"))
        .args([
            "--headless",
            "--norestore",
            "--convert-to",
            "pdf",
            "--outdir",
        ])
        .arg(&dir)
        .arg(&docx_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            format!("failed to spawn soffice (is LibreOffice installed?): {e}")
        })?;

    if !output.status.success() || !pdf_path.is_file() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let _ = std::fs::remove_dir_all(&dir);
        return Err(format!(
            "LibreOffice PDF conversion failed: {stderr}{stdout}"
        ));
    }

    let bytes = std::fs::read(&pdf_path).map_err(|e| format!("read pdf: {e}"))?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(bytes)
}

pub async fn export_docx(markdown: &str) -> Result<Vec<u8>, String> {
    pandoc_docx(markdown).await
}

pub async fn export_pdf(markdown: &str) -> Result<Vec<u8>, String> {
    let docx = pandoc_docx(markdown).await?;
    soffice_docx_to_pdf(&docx).await
}

pub fn attachment_filename(base: &str, ext: &str) -> String {
    let safe: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let name = if safe.is_empty() { "export".into() } else { safe };
    format!("{name}.{ext}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn split_mixed_line_by_script() {
        let parts = split_by_script("Hello नमस्ते વિશ્વ");
        assert!(parts.len() >= 3);
        assert_eq!(parts[0].0, Script::Latin);
        assert!(parts.iter().any(|(s, t)| *s == Script::Devanagari && t.contains("नमस्ते")));
        assert!(parts.iter().any(|(s, t)| *s == Script::Gujarati && t.contains("વિશ્વ")));
    }

    #[test]
    fn rewrite_theme_replaces_mangal() {
        let xml = r#"<a:font script="Deva" typeface="Mangal"/><a:cs typeface=""/>"#;
        let out = rewrite_theme_fonts(xml);
        assert!(out.contains("Noto Sans Devanagari"));
        assert!(!out.contains("Mangal"));
    }

    #[test]
    fn rewrite_run_splits_and_sets_fonts() {
        let run = "<w:r><w:t>Hi नमस्ते</w:t></w:r>";
        let out = rewrite_run_xml(run);
        assert!(out.contains("Noto Sans Devanagari"));
        assert!(out.contains("Liberation Sans"));
        assert!(out.matches("<w:r>").count() >= 2);
    }

    #[tokio::test]
    async fn prepared_docx_marks_devanagari_runs() {
        if std::process::Command::new("pandoc")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let bytes = pandoc_docx("# Test\n\nHello नमस्ते વિશ્વ\n")
            .await
            .expect("pandoc+prepare");
        let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("zip");
        {
            let mut doc = archive.by_name("word/document.xml").expect("document");
            let mut xml = String::new();
            doc.read_to_string(&mut xml).unwrap();
            assert!(
                xml.contains("Noto Sans Devanagari"),
                "document.xml should name Noto Sans Devanagari on runs"
            );
            assert!(
                !xml.contains("Mangal"),
                "document should not reference Mangal"
            );
        }
        {
            let mut theme = archive.by_name("word/theme/theme1.xml").expect("theme");
            let mut theme_xml = String::new();
            theme.read_to_string(&mut theme_xml).unwrap();
            assert!(!theme_xml.contains("typeface=\"Mangal\""));
            assert!(theme_xml.contains("Noto Sans Devanagari"));
        }

        if std::process::Command::new("soffice")
            .arg("--version")
            .output()
            .is_ok()
        {
            let pdf = export_pdf("# Test\n\nHello नमस्ते વિશ્વ\n")
                .await
                .expect("pdf export");
            let has_noto = pdf
                .windows(b"NotoSansDevanagari".len())
                .any(|w| w == b"NotoSansDevanagari");
            assert!(has_noto, "PDF should embed Noto Sans Devanagari");
        }
    }
}
