use owo_colors::OwoColorize;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
    sync::{Mutex, OnceCell},
};

use crate::error::{Res, ResExt};

pub struct CrawledPage {
    pub url: String,
    pub title: String,
    pub links: usize,
    pub text: Option<String>,
    pub content: Option<String>,
}

static WRITER: OnceCell<Mutex<BufWriter<File>>> = OnceCell::const_new();

async fn init_logger() -> &'static Mutex<BufWriter<File>> {
    WRITER
        .get_or_init(async || {
            let args = &*crate::ARGS;
            let path = &args.output;
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
                            "FATAL:".red().bold(),
                            path.to_string_lossy().red().bold()
                        )
                    },
                    1,
                );

            Mutex::new(BufWriter::with_capacity(16 * 1024, file))
        })
        .await
}

pub(crate) async fn write_output(page: CrawledPage) -> Res<()> {
    let mut wtr = init_logger().await.lock().await;

    let mut esc_buf = Vec::new();

    wtr.write_all(b"{\"URL\": \"").await?;
    escape_json(&page.url, &mut esc_buf);
    wtr.write_all(esc_buf.as_slice()).await?;

    wtr.write_all(b"\", \"Title\": \"").await?;
    escape_json(&page.title, &mut esc_buf);
    wtr.write_all(esc_buf.as_slice()).await?;

    wtr.write_all(b"\", \"Links\": ").await?;
    wtr.write_all(page.links.to_string().as_bytes()).await?;

    if let Some(text) = page.text {
        wtr.write_all(b", \"Text\": \"").await?;
        escape_json(&text, &mut esc_buf);
        wtr.write_all(esc_buf.as_slice()).await?;
        wtr.write_all(b"\"").await?;
    }

    if let Some(content) = page.content {
        wtr.write_all(b", \"Content\": \"").await?;
        escape_json(&content, &mut esc_buf);
        wtr.write_all(esc_buf.as_slice()).await?;
        wtr.write_all(b"\"}\n").await?;
    } else {
        wtr.write_all(b"}\n").await?;
    }

    Ok(())
}

pub(crate) async fn flush_output() -> Res<()> {
    let mut wtr = init_logger().await.lock().await;

    wtr.flush().await?;

    Ok(())
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
