use std::any::Any;

use is_terminal::IsTerminal;
use wasi_common::{WasiFile, file::FileType, Error};

pub struct ExtensionStdout {

}

#[async_trait::async_trait]
impl WasiFile for ExtensionStdout {
    fn as_any(&self) ->  &dyn Any {
        self
    }

    // #[cfg(unix)]
    // fn pollable(&self) -> Option<Borrowed

    async fn get_filetype(&self) -> Result<FileType, Error> {
        if self.isatty() {
            Ok(FileType::CharacterDevice)
        } else {
            Ok(FileType::Unknown)
        }
    }

    fn isatty(&self) -> bool {
        std::io::stdout().is_terminal()
    }
}