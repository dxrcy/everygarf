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

pub const QUERY_NONE: u8 = 0;
pub const QUERY_SOME: u8 = 10;
