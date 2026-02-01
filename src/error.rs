use owo_colors::OwoColorize;
use resext::resext;
use strip_ansi_escapes::strip_str;
use time::macros::format_description;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, Stdout, stdout},
    sync::{Mutex, OnceCell},
};

#[resext(
    msg_delimiter = " • ",
    source_prefix = "Cause: ",
    include_variant = true
)]
pub(crate) enum CrawnError {
    IoError(std::io::Error),
    NetworkError(reqwest::Error),
    UrlParseError(url::ParseError),
    ScrapeError(scraper::error::SelectorErrorKind<'static>),
    ConcurrentTaskFailure(tokio::task::JoinError),
    Custom(String),
}

unsafe impl Send for CrawnError {}
unsafe impl Sync for CrawnError {}
unsafe impl Send for ResErr {}
unsafe impl Sync for ResErr {}

enum Logger {
    Stdout(Mutex<Stdout>),
    File(Mutex<File>),
}

static LOGGER: OnceCell<Logger> = OnceCell::const_new();

async fn init_logger() -> &'static Logger {
    LOGGER
        .get_or_init(async || {
            let args = &*crate::ARGS;
            if let Some(path) = &args.log_file {
                let file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)
                    .await
                    .better_expect(
                        || {
                            format!(
                                "{} Failed to open log file: {}",
                                "FATAL:".red().bold(),
                                path.to_string_lossy().red().bold()
                            )
                        },
                        1,
                    );

                Logger::File(Mutex::new(file))
            } else {
                Logger::Stdout(Mutex::new(stdout()))
            }
        })
        .await
}

pub(crate) trait Log<T> {
    async fn log(self, level: &'static str) -> Res<Option<T>>;
}

const LOG_TIMESTAMP_FORMAT: &[time::format_description::BorrowedFormatItem] = format_description!(
    "[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second].[subsecond digits:3]"
);

impl<T> Log<T> for Res<T> {
    async fn log(self, level: &'static str) -> Res<Option<T>> {
        match self {
            Ok(ok) => Ok(Some(ok)),
            Err(err) => {
                let timestamp: String = time::OffsetDateTime::now_utc()
                    .to_offset(
                        time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC),
                    )
                    .format(&LOG_TIMESTAMP_FORMAT)
                    .map_err(std::io::Error::other)
                    .context("Failed to format timestamp for log")?;

                let logger = init_logger().await;

                match logger {
                    Logger::File(mutex_wtr) => {
                        let mut wtr = mutex_wtr.lock().await;

                        let log = format!(
                            "{} {}:\n{}\n\n",
                            timestamp,
                            level,
                            strip_str(err.to_string())
                        );

                        wtr.write_all(log.as_bytes())
                            .await
                            .with_context(|| format!("Failed to write log at: {}", timestamp))?;

                        Ok(None)
                    }

                    Logger::Stdout(mutex_stdout) => {
                        let mut stdout = mutex_stdout.lock().await;

                        let log = format!("{} {}:\n{}\n\n", timestamp, level, err);

                        stdout
                            .write_all(log.as_bytes())
                            .await
                            .with_context(|| format!("Failed to write log at: {}", timestamp))?;

                        Ok(None)
                    }
                }
            }
        }
    }
}
impl Log<String> for String {
    async fn log(self, level: &'static str) -> Res<Option<String>> {
        let timestamp: String = time::OffsetDateTime::now_utc()
            .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
            .format(&LOG_TIMESTAMP_FORMAT)
            .map_err(std::io::Error::other)
            .context("Failed to format timestamp for log")?;

        let logger = init_logger().await;

        match logger {
            Logger::File(mutex_wtr) => {
                let mut wtr = mutex_wtr.lock().await;

                let log = format!("{} {}:\n{}\n\n", timestamp, level, strip_str(self));

                wtr.write_all(log.as_bytes())
                    .await
                    .with_context(|| format!("Failed to write log at: {}", timestamp))?;

                Ok(None)
            }

            Logger::Stdout(mutex_stdout) => {
                let mut stdout = mutex_stdout.lock().await;

                let log = format!("{} {}:\n{}\n\n", timestamp, level, self);

                stdout
                    .write_all(log.as_bytes())
                    .await
                    .with_context(|| format!("Failed to write log at: {}", timestamp))?;

                Ok(None)
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! match_option {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => continue,
        }
    };
}
