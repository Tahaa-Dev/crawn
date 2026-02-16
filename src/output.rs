use owo_colors::OwoColorize;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
    sync::{Mutex, OnceCell},
};

use crate::error::{Res, ResExt};

static WRITER: OnceCell<Mutex<BufWriter<File>>> = OnceCell::const_new();

async fn init_writer() -> &'static Mutex<BufWriter<File>> {
    WRITER
        .get_or_init(async || {
            let args = &*crate::ARGS;
            let path = &args.output;

            let ext = path.extension().unwrap_or_else(|| std::ffi::OsStr::new(""));

            if ext != "ndjson" {
                eprintln!(
                    "{} Output file extension: {}{}{} is not: {}",
                    "[FATAL]".red().bold(),
                    "[".purple(),
                    ext.display().purple(),
                    "]".purple(),
                    "[ndjson]".purple()
                );
                std::process::exit(1);
            }

            let res = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .await;

            let file = match res {
                Ok(file) => file,
                Err(err) => {
                    eprintln!(
                        "{} Failed to open output file: {}\nCause: {}",
                        "[FATAL]".red().bold(),
                        path.to_string_lossy().red().bold(),
                        err
                    );

                    std::process::exit(1);
                }
            };

            let buf_cap = if args.include_content {
                1024 * 16
            } else if args.include_text {
                1024 * 4
            } else {
                256
            };

            Mutex::new(BufWriter::with_capacity(buf_cap, file))
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
        .context("Failed to flush writer into output file")
}

pub async fn write_output(
    url: String,
    title: String,
    links: usize,
    text: Option<String>,
    content: Option<String>,
) -> Res<()> {
    let line = tokio::task::spawn_blocking(move || {
        let mut buf = Vec::with_capacity(256);
        let mut line = Vec::with_capacity(text.as_ref().map_or(1024, |t| t.len() + 512));

        line.extend_from_slice(b"{\"URL\": \"");
        escape_json(&*url, &mut buf);
        line.extend_from_slice(&buf);

        line.extend_from_slice(b"\", \"Title\": \"");
        escape_json(title, &mut buf);
        line.extend_from_slice(&buf);

        line.extend_from_slice(b"\", \"Links\": ");
        line.extend_from_slice(links.to_string().as_bytes());

        if let Some(t) = text {
            line.extend_from_slice(b", \"Text\": \"");
            escape_json(t, &mut buf);
            line.extend_from_slice(&buf);
            line.extend_from_slice(b"\"}\n");
        } else if let Some(c) = content {
            line.extend_from_slice(b", \"Content\": \"");
            escape_json(c, &mut buf);
            line.extend_from_slice(&buf);
            line.extend_from_slice(b"\"}\n");
        } else {
            line.extend_from_slice(b"}\n");
        }

        line
    })
    .await
    .context("Failed to escape output concurrently")?;

    init_writer()
        .await
        .lock()
        .await
        .write_all(&line)
        .await
        .context("Failed to write output entry into output file")?;

    Ok(())
}

#[inline(always)]
fn escape_json<S: AsRef<str>>(s: S, buf: &mut Vec<u8>) {
    buf.clear();

    for byte in s.as_ref().bytes() {
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

        escape_json(s, &mut buf);

        assert_eq!(
            &buf,
            b"escape\\t string\\r\\nfor \\b \\\\ testing \\f\\\"escape\\\" function"
        );
    }
}
