use std::{any::Any, io};

use wasi_common::{
    file::{FdFlags, FileType},
    Error, ErrorExt, SystemTimeSpec, WasiFile,
};

pub struct ExtensionStdout {
    pub prefix: String,
    pub stdout: wasmtime_wasi::sync::stdio::Stdout,
}

#[async_trait::async_trait]
impl WasiFile for ExtensionStdout {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(unix)]
    fn pollable(&self) -> Option<rustix::fd::BorrowedFd> {
        self.stdout.pollable()
    }

    #[cfg(windows)]
    fn pollable(&self) -> Option<io_extras::os::windows::RawHandleOrSocket> {
        self.stdout.pollable()
    }

    async fn get_filetype(&self) -> Result<FileType, Error> {
        self.stdout.get_filetype().await
    }

    async fn get_fdflags(&self) -> Result<FdFlags, Error> {
        self.stdout.get_fdflags().await
    }

    async fn write_vectored<'a>(
        &self,
        bufs: &[io::IoSlice<'a>],
    ) -> Result<u64, Error> {
        let span =
            tracing::span!(tracing::Level::INFO, "plugin", id = self.prefix);
        let _guard = span.enter();

        let mut written = 0;
        for buf in bufs {
            // AssemblyScript prints a single newline in a completely new buffer?
            if buf.as_ref() == [10] {
                continue;
            }

            let as_str = std::str::from_utf8(buf)
                .map_err(|_| Error::invalid_argument())?;

            // Strip trailing newline because tracing also adds one.
            let stripped = as_str
                .strip_suffix('\n')
                .or(as_str.strip_suffix("\r\n"))
                .unwrap_or(as_str);

            tracing::info!("{stripped}");
            written += buf.len();
        }
        Ok(written as u64)
    }

    async fn write_vectored_at<'a>(
        &self,
        _bufs: &[io::IoSlice<'a>],
        _offset: u64,
    ) -> Result<u64, Error> {
        Err(Error::seek_pipe())
    }

    async fn seek(&self, _pos: io::SeekFrom) -> Result<u64, Error> {
        Err(Error::seek_pipe())
    }

    async fn set_times(
        &self,
        atime: Option<SystemTimeSpec>,
        mtime: Option<SystemTimeSpec>,
    ) -> Result<(), Error> {
        self.stdout.set_times(atime, mtime).await
    }

    fn isatty(&self) -> bool {
        self.stdout.isatty()
    }
}
