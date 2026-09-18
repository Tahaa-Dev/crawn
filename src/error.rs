use std::io::IsTerminal;

use owo_colors::OwoColorize;
use resext::ctx;
use resext::resext;
use time::macros::format_description;
use tokio::io::Stderr;
use tokio::io::stderr;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    sync::{Mutex, OnceCell},
};

#[resext(
    delimiter = " -> ",
    source_prefix = "Cause: ",
    include_variant = true,
    alloc = true,
    buf_size = 76
)]
pub enum CrawnError {
    IoError(std::io::Error),
    NetworkError(reqwest::Error),
    UrlParseError(url::ParseError),
    ScrapeError(scraper::error::SelectorErrorKind<'static>),
    ConcurrentTaskFailure(tokio::task::JoinError),
    FmtError(String),
}

unsafe impl Send for CrawnError {}
unsafe impl Sync for CrawnError {}
unsafe impl Send for ResErr {}
unsafe impl Sync for ResErr {}

static LOGGER: OnceCell<(Mutex<BufWriter<Stderr>>, bool)> = OnceCell::const_new();

async fn init_logger() -> &'static (Mutex<BufWriter<Stderr>>, bool) {
    LOGGER
        .get_or_init(async || {
            (
                Mutex::new(BufWriter::with_capacity(1024 * 64, stderr())),
                std::io::stderr().is_terminal(),
            )
        })
        .await
}

pub trait Log<T> {
    async fn log(self) -> Res<Option<T>>;
}

pub const LOG_TIMESTAMP_FORMAT: &[time::format_description::BorrowedFormatItem] = format_description!(
    "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second].[subsecond digits:3]"
);

impl<T> Log<T> for Res<T> {
    async fn log(self) -> Res<Option<T>> {
        match self {
            Ok(ok) => Ok(Some(ok)),
            Err(err) => {
                let timestamp: String = time::OffsetDateTime::now_utc()
                    .to_offset(
                        time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC),
                    )
                    .format(&LOG_TIMESTAMP_FORMAT)
                    .map_err(|_| String::from("Format Failure"))
                    .context("Failed to format timestamp for log")?;

                let (logger, is_terminal) = init_logger().await;

                {
                    let wtr: &mut BufWriter<Stderr> = &mut *logger.lock().await;

                    if *is_terminal {
                        wtr.write_all(timestamp.yellow().to_string().as_bytes())
                            .await
                            .context(ctx!("Failed to write log at: {}", timestamp))?;

                        wtr.write_all(" [WARN]: ".purple().to_string().as_bytes())
                            .await
                            .context(ctx!("Failed to write log at: {}", timestamp))?;
                    } else {
                        wtr.write_all(timestamp.as_bytes())
                            .await
                            .context(ctx!("Failed to write log at: {}", timestamp))?;

                        wtr.write_all(b" [WARN]: ")
                            .await
                            .context(ctx!("Failed to write log at: {}", timestamp))?;
                    }

                    wtr.write_all(err.to_string().as_bytes())
                        .await
                        .context(ctx!("Failed to write log at: {}", timestamp))?;

                    wtr.write_all(b"\n")
                        .await
                        .context(ctx!("Failed to write log at: {}", timestamp))?;
                }

                Ok(None)
            }
        }
    }
}

impl Log<()> for String {
    async fn log(self) -> Res<Option<()>> {
        let timestamp: String = time::OffsetDateTime::now_utc()
            .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
            .format(&LOG_TIMESTAMP_FORMAT)
            .map_err(|_| String::from("Format Failure"))
            .context("Failed to format timestamp for log")?;

        let (logger, is_terminal) = init_logger().await;

        {
            let wtr: &mut BufWriter<Stderr> = &mut *logger.lock().await;

            if *is_terminal {
                wtr.write_all(timestamp.yellow().to_string().as_bytes())
                    .await
                    .context(ctx!("Failed to write log at: {}", timestamp))?;

                wtr.write_all(" [INFO]: ".blue().to_string().as_bytes())
                    .await
                    .context(ctx!("Failed to write log at: {}", timestamp))?;
            } else {
                wtr.write_all(timestamp.as_bytes())
                    .await
                    .context(ctx!("Failed to write log at: {}", timestamp))?;

                wtr.write_all(b" [INFO]: ")
                    .await
                    .context(ctx!("Failed to write log at: {}", timestamp))?;
            }

            wtr.write_all(self.as_bytes())
                .await
                .context(ctx!("Failed to write log at: {}", timestamp))?;

            wtr.write_all(b"\n")
                .await
                .context(ctx!("Failed to write log at: {}", timestamp))?;
        }

        Ok(None)
    }
}

pub async fn flush_logger() -> Res<()> {
    let (logger, _) = init_logger().await;
    let mut logger = logger.lock().await;

    logger.flush().await.context("Failed to flush logger")
}

#[macro_export]
macro_rules! match_option {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => continue,
        }
    };
}
