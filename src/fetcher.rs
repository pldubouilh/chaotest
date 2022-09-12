use anyhow::{bail, Context, Result};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub fn fetcher(src: &str, dest: &mut Vec<u8>) -> Result<()> {
    let err = "cant fetch initial src specified";

    match StringType::get(src)? {
        StringType::Url(url) => {
            ureq::get(url.as_str())
                .call()
                .context(err)?
                .into_reader()
                .read_to_end(dest)
                .context(err)?;
        }
        StringType::FilePath(path) => {
            std::fs::File::open(path)?.read_to_end(dest)?;
        }
    };

    Ok(())
}

enum StringType {
    Url(url::Url),
    FilePath(PathBuf),
}

impl StringType {
    fn get(str: &str) -> Result<Self> {
        match url::Url::parse(str) {
            Ok(url) => Ok(Self::Url(url)),
            // ... or it is a file
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                Ok(Self::FilePath(Path::new(str).to_path_buf()))
            }
            Err(e) => {
                bail!(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() -> Result<()> {
        let str = "https://www.google.com/";
        let string_type = StringType::get(str)?;
        match string_type {
            StringType::Url(url) => {
                assert_eq!(url.as_str(), str);
            }
            _ => bail!("Wrong Type"),
        }
        Ok(())
    }

    #[test]
    fn test_bad_url() -> Result<()> {
        let str = "http://1273.23.12.12";
        let string_type = StringType::get(str);
        match string_type {
            Err(e) => {
                let real_error: url::ParseError = e.downcast()?;
                if real_error == url::ParseError::InvalidIpv4Address {
                    return Ok(());
                }
            }
            _ => bail!("Wrong Type"),
        }
        bail!("Test Failed")
    }

    #[test]
    fn test_file() -> Result<()> {
        let str = "test-file";
        let string_type = StringType::get(str);
        match string_type {
            Ok(StringType::FilePath(path)) => {
                let path = path.to_str().unwrap();
                assert_eq!(str, path);
            }
            _ => bail!("Wrong Type"),
        }
        Ok(())
    }
}
