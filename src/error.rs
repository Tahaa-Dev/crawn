use resext::ResExt;
ResExt! {
    pub(crate) enum CrawnError {
        Io(std::io::Error),
        Network(reqwest::Error),
        UrlParsing(url::ParseError),
        Scraping(scraper::error::SelectorErrorKind<'static>),
    }
}
