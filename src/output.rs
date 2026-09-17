use tokio::{
    io::{AsyncWriteExt, BufWriter, Stdout, stdout},
    sync::{Mutex, OnceCell},
};

use crate::error::{Res, ResExt};

static WRITER: OnceCell<Mutex<BufWriter<Stdout>>> = OnceCell::const_new();

async fn init_writer() -> &'static Mutex<BufWriter<Stdout>> {
    WRITER
        .get_or_init(async || {
            let args = &*crate::ARGS;
            let buf_cap = if args.include_content {
                1024 * 16
            } else if args.include_text {
                1024 * 4
            } else {
                256
            };

            Mutex::new(BufWriter::with_capacity(buf_cap, stdout()))
        })
        .await
}

pub async fn flush_writer() -> Res<()> {
    init_writer()
        .await
        .lock()
        .await
        .flush()
        .await
        .context("Failed to flush writer")
}

pub async fn write_output(
    url: &str,
    title: &str,
    links: usize,
    text: Option<String>,
    content: Option<String>,
) -> Res<()> {
    let mut line = Vec::with_capacity(text.as_ref().map_or(1024, |t| t.len() + 512));

    line.extend_from_slice(b"{\"URL\": \"");
    escape_json(url.bytes(), &mut line);

    line.extend_from_slice(b"\", \"Title\": \"");
    escape_json(title.bytes(), &mut line);

    if links != 0 {
        line.extend_from_slice(b"\", \"Links\": ");
        line.extend_from_slice(links.to_string().as_bytes());
    } else {
        line.push(b'"');
    }

    if let Some(t) = text {
        line = tokio::task::spawn_blocking(move || {
            line.extend_from_slice(b", \"Text\": \"");
            escape_json(t.bytes(), &mut line);
            line.extend_from_slice(b"\"}\n");
            line
        })
        .await
        .context("Failed to escape output concurrently")?;
    } else if let Some(c) = content {
        line = tokio::task::spawn_blocking(move || {
            line.extend_from_slice(b", \"Content\": \"");
            escape_json(c.bytes(), &mut line);
            line.extend_from_slice(b"\"}\n");
            line
        })
        .await
        .context("Failed to escape output concurrently")?;
    } else {
        line.extend_from_slice(b"}\n");
    }

    init_writer()
        .await
        .lock()
        .await
        .write_all(&line)
        .await
        .context("Failed to write output entry")?;

    Ok(())
}

#[inline(always)]
fn escape_json(s: core::str::Bytes<'_>, buf: &mut Vec<u8>) {
    for byte in s {
        match byte {
            b'"' => buf.extend_from_slice(b"\\\""),
            b'\\' => buf.extend_from_slice(b"\\\\"),
            b'\n' => buf.extend_from_slice(b"\\n"),
            b'\r' => buf.extend_from_slice(b"\\r"),
            b'\t' => buf.extend_from_slice(b"\\t"),
            b'\x08' => buf.extend_from_slice(b"\\b"),
            b'\x0C' => buf.extend_from_slice(b"\\f"),
            b if b < 0x20 => {
                // Control characters: \u00XX
                buf.extend_from_slice(b"\\u00");
                buf.push(b"0123456789abcdef"[(b >> 4) as usize]);
                buf.push(b"0123456789abcdef"[(b & 0x0F) as usize]);
            }
            b => buf.push(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::output::escape_json;

    #[tokio::test]
    async fn test_escaping() {
        let mut buf = Vec::new();

        let s = "escape\t string\r\nfor \x08 \\ testing \x0C\"escape\" function";

        escape_json(s.bytes(), &mut buf);

        assert_eq!(
            &buf,
            b"escape\\t string\\r\\nfor \\b \\\\ testing \\f\\\"escape\\\" function"
        );
    }
}
