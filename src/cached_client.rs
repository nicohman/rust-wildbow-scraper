extern crate easy_error;
extern crate reqwest;

use easy_error::{Error, ResultExt};
use reqwest::blocking::{Client, Response};
use reqwest::Url;
use std::fs::create_dir_all;
use std::path::PathBuf;

/// Facilitates response type selection in `CachedClient`
/// by converting the cached and fetched responses to requested type.
pub trait DataType: std::convert::AsRef<[u8]> + Clone {
    fn from_response(response: Response) -> Result<Self, Error>
    where
        Self: Sized;
    fn from_bytes(contents: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
}

impl DataType for String {
    fn from_response(response: Response) -> Result<Self, Error> {
        // While reqwest supports determining response’s character encoding from Content-Type HTTP header,
        // falling back to UTF-8 when not available, web sites that use a different encoding will likely
        // specify it inside the document anyway so the header sniffing will probably not help.
        // When loading from bytes (i.e. from cache), we are already assuming UTF-8
        // so let’s do it for HTTP response as well to be consistent.
        let contents = response.bytes().context("Cannot extract text.")?;
        Ok(String::from_utf8_lossy(&contents).to_string())
    }

    fn from_bytes(contents: &[u8]) -> Result<Self, Error> {
        Ok(String::from_utf8_lossy(contents).to_string())
    }
}

impl DataType for Vec<u8> {
    fn from_response(response: Response) -> Result<Self, Error> {
        Ok(Vec::from(response.bytes().context("Cannot extract data.")?))
    }

    fn from_bytes(contents: &[u8]) -> Result<Self, Error> {
        Ok(Vec::from(contents))
    }
}

/// Wraps the response from `CachedClient` to allow the consumer
/// to find out whether the resource was fetched or obtained from cache.
#[derive(Debug)]
pub enum Resource<T: DataType> {
    Fetched(T),
    Cached(T),
}

impl<T: DataType> Resource<T> {
    pub fn contents(&self) -> &T {
        match self {
            Resource::Fetched(contents) => contents,
            Resource::Cached(contents) => contents,
        }
    }

    pub fn is_cached(&self) -> bool {
        match self {
            Resource::Fetched(_) => false,
            Resource::Cached(_) => true,
        }
    }
}

/// Wrapper around `reqwest::Client` that caches files in the provided directory.
pub struct CachedClient {
    client: Client,
    cache_dir: Option<PathBuf>,
}

impl CachedClient {
    pub fn new(cache_dir: Option<PathBuf>) -> Result<Self, Error> {
        if let Some(ref cache_path) = cache_dir {
            create_dir_all(cache_path).context(format!("Could not create cache directory {cache_path:?}"))?;
        }

        Ok(Self {
            client: Client::new(),
            cache_dir: cache_dir,
        })
    }

    pub fn fetch_uncached(&self, url: &Url) -> Result<Response, Error> {
        self.client
            .get(url.clone())
            .send()
            .context(format!("Could not retrieve page {url}"))
    }

    /// Provides the contents of given URL in the format specified by the type parameter `T`:
    ///  - For `String`, the contents will be decoded using UTF-8 encoding.
    ///  - For `Vec<u8>`, the conents will be returned as they are.
    /// When the client has a cache directory available, it will attempt to look for the URL in there.
    pub fn fetch<T: DataType>(&self, url: &Url, skip_cache: bool) -> Result<Resource<T>, Error> {
        Ok(match self.cache_dir {
            // Cache directory exists.
            Some(ref cache_path) => {
                let cached_file = cache_path.join(url.to_string().replace("/", "%2F"));
                if !skip_cache && cached_file.exists() {
                    let cached_contents =
                        std::fs::read(&cached_file).context(format!("Unable to load {cached_file:?} from cache"))?;
                    Resource::Cached(DataType::from_bytes(&cached_contents)?)
                } else {
                    let page = self.fetch_uncached(url)?;
                    let contents: T = DataType::from_response(page).context("Unable to retrieve data")?;
                    std::fs::write(cached_file, contents.clone()).context(format!("Could not cache {url}"))?;
                    Resource::Fetched(contents)
                }
            }
            // No cache directory, fetch directly.
            None => Resource::Fetched(DataType::from_response(self.fetch_uncached(url)?)?),
        })
    }
}
