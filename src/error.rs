use owo_colors::{OwoColorize, colors::css::MediumPurple};
use resext::resext;
use strip_ansi_escapes::strip_str;
use time::macros::format_description;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, Stdout, stdout},
    sync::{Mutex, OnceCell},
};

#[resext(
    delimiter = " -> ",
    source_prefix = "Cause: ",
    include_variant = true,
    alloc = true
)]
pub enum CrawnError {
    IoError(std::io::Error),
    NetworkError(reqwest::Error),
    UrlParseError(url::ParseError),
    ScrapeError(scraper::error::SelectorErrorKind<'static>),
    ConcurrentTaskError(tokio::task::JoinError),
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
                let res = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)
                    .await;

                match res {
                    Ok(file) => Logger::File(Mutex::new(file)),
                    Err(err) => {
                        println!(
                            "{} Failed to open log file: {}\nCause: {}",
                            "[WARN]".fg::<MediumPurple>(),
                            path.to_string_lossy().red().bold(),
                            err
                        );

                        Logger::Stdout(Mutex::new(stdout()))
                    }
                }
            } else {
                Logger::Stdout(Mutex::new(stdout()))
            }
        })
        .await
}

pub trait Log<T> {
    async fn log(self, level: &'static str) -> Res<Option<T>>;
}

pub const LOG_TIMESTAMP_FORMAT: &[time::format_description::BorrowedFormatItem] = format_description!(
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
                    .map_err(|_| String::from("Format Failure"))
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
                            .with_context(format_args!("Failed to write log at: {}", timestamp))?;

                        Ok(None)
                    }

                    Logger::Stdout(mutex_stdout) => {
                        let mut stdout = mutex_stdout.lock().await;

                        let log = format!(
                            "{} {}:\n{}\n\n",
                            timestamp.yellow(),
                            level.fg::<MediumPurple>(),
                            err
                        );

                        stdout
                            .write_all(log.as_bytes())
                            .await
                            .with_context(format_args!("Failed to write log at: {}", timestamp))?;

                        Ok(None)
                    }
                }
            }
        }
    }
}

impl Log<()> for String {
    async fn log(self, level: &'static str) -> Res<Option<()>> {
        let timestamp: String = time::OffsetDateTime::now_utc()
            .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
            .format(&LOG_TIMESTAMP_FORMAT)
            .map_err(|_| String::from("Format Failure"))
            .context("Failed to format timestamp for log")?;

        let logger = init_logger().await;

        match logger {
            Logger::File(mutex_wtr) => {
                let mut wtr = mutex_wtr.lock().await;

                let log = format!("{} {}:\n{}\n\n", timestamp, level, strip_str(self));

                wtr.write_all(log.as_bytes())
                    .await
                    .with_context(format_args!("Failed to write log at: {}", timestamp))?;

                Ok(None)
            }

            Logger::Stdout(mutex_stdout) => {
                let mut stdout = mutex_stdout.lock().await;

                let log = format!(
                    "{} {}:\n{}\n\n",
                    timestamp.yellow(),
                    level.fg::<MediumPurple>(),
                    self
                );

                stdout
                    .write_all(log.as_bytes())
                    .await
                    .with_context(format_args!("Failed to write log at: {}", timestamp))?;

                Ok(None)
            }
        }
    }
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
