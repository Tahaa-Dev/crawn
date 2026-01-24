use owo_colors::OwoColorize;
use resext::panic_if;
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

            panic_if!(
                ext != "ndjson",
                || format!(
                    "{} Output file extension: {}{}{} is not: {}",
                    "[FATAL]".red().bold(),
                    "[".purple(),
                    ext.display().purple(),
                    "]".purple(),
                    "[ndjson]".purple()
                ),
                1
            );

            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .await
                .better_expect(
                    || {
                        format!(
                            "{} Failed to open output file: {}",
                            "[FATAL]".red().bold(),
                            path.to_string_lossy().red().bold()
                        )
                    },
                    1,
                );

            Mutex::new(BufWriter::with_capacity(512, file))
        })
        .await
}

pub(crate) async fn write_output(
    url: &str,
    title: &str,
    links: usize,
    text: Option<&str>,
    content: Option<&str>,
) -> Res<()> {
    let mut wtr = init_writer().await.lock().await;

    let mut esc_buf = Vec::new();

    wtr.write_all(b"{\"URL\": \"").await?;
    escape_json(url, &mut esc_buf);
    wtr.write_all(esc_buf.as_slice()).await?;

    wtr.write_all(b"\", \"Title\": \"").await?;
    escape_json(title, &mut esc_buf);
    wtr.write_all(esc_buf.as_slice()).await?;

    wtr.write_all(b"\", \"Links\": ").await?;
    wtr.write_all(links.to_string().as_bytes()).await?;

    if let Some(text) = text {
        wtr.write_all(b", \"Text\": \"").await?;
        escape_json(text, &mut esc_buf);
        wtr.write_all(esc_buf.as_slice()).await?;
        wtr.write_all(b"\"").await?;
    }

    if let Some(content) = content {
        wtr.write_all(b", \"Content\": \"").await?;
        escape_json(content, &mut esc_buf);
        wtr.write_all(esc_buf.as_slice()).await?;
        wtr.write_all(b"\"}\n").await?;
    } else {
        wtr.write_all(b"}\n").await?;
    }

    wtr.flush()
        .await
        .context("Failed to flush writer into output file")
}

fn escape_json(s: &str, buf: &mut Vec<u8>) {
    buf.clear();

    for byte in s.bytes() {
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

    #[test]
    fn test_escaping() {
        let mut buf = Vec::new();

        let s = "escape\t string\r\nfor \x08 \\ testing \x0C\"escape\" function";

        escape_json(s, &mut buf);

        assert_eq!(buf, b"escape\\t string\\r\\nfor \\b \\\\ testing \\f\\\"escape\\\" function");
    }
}
