use owo_colors::OwoColorize;
use resext::ResExt;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
    sync::{Mutex, OnceCell},
};

ResExt! {
    pub(crate) enum CrawnError {
        Io(std::io::Error),
        Network(reqwest::Error),
        UrlParsing(url::ParseError),
        Scraping(scraper::error::SelectorErrorKind<'static>),
    }
}

static LOGGER: OnceCell<Option<Mutex<File>>> = OnceCell::const_new();

async fn init_logger() -> &'static Option<Mutex<File>> {
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

                Some(Mutex::new(file))
            } else {
                None
            }
        })
        .await
}

pub(crate) trait Log<T> {
    async fn log_err(self) -> Res<Option<T>>;
}

// Uses Default(s) because `String::new()` is free, and it is cleaner than Option<T>
impl<T> Log<T> for Res<T> {
    async fn log_err(self) -> Res<Option<T>> {
        match self {
            Ok(ok) => Ok(Some(ok)),
            Err(err) => {
                if let Some(file) = init_logger().await {
                    let mut wtr = file.lock().await;

                    wtr.write_all(err.to_string().as_bytes())
                        .await
                        .context("Failed to write logs into log file")?;

                    wtr.write_all(b"\n\n---\n\n")
                        .await
                        .context("Failed to write delimiter between logs into log file")?;

                    Ok(None)
                } else {
                    eprintln!("{}\n\n---\n\n", err);
                    Ok(None)
                }
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
