#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("..")]
    DownloadFail,

    #[error("..")]
    NoDir,

    #[error("..")]
    CreateDir,

    #[error("..")]
    ReadExistingDates,

    #[error("..")]
    ProxyPing,

    #[error("..")]
    CacheDownload,

    #[error("..")]
    CleanCache,

    #[error("..")]
    CacheAppendNewline,

    #[error("..")]
    BadStartDate,
}

