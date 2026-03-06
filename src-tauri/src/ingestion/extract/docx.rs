use std::{fs::File, io::Read, path::Path};

use quick_xml::{events::Event, Reader};
use zip::ZipArchive;

pub async fn extract_docx_text(path: &Path) -> anyhow::Result<String> {
    let path_buf = path.to_path_buf();
    let text = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
        let file = File::open(path_buf)?;
        let mut zip = ZipArchive::new(file)?;
        let mut xml = String::new();
        zip.by_name("word/document.xml")?.read_to_string(&mut xml)?;

        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);

        let mut out = String::new();
        loop {
            match reader.read_event() {
                Ok(Event::Text(t)) => {
                    out.push_str(t.unescape()?.as_ref());
                    out.push(' ');
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }
        Ok(out)
    })
    .await??;

    Ok(text)
}
