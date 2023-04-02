use std::{any::Any, pin::Pin, future::Future};
use is_terminal::IsTerminal;

use wasi_common::file::FileType;
use wasmtime_wasi::{WasiFile, Error};

pub struct ExtensionStdout {

}

#[async_trait::async_trait]
impl WasiFile for ExtensionStdout {
    fn as_any(&self) ->  &dyn Any {
        self
    }

    async fn get_filetype(&self) -> Result<FileType, Error> {
        if self.isatty() {
            Ok(FileType::CharacterDevice)
        } else {
            Ok(FileType::Unknown)
        }
    }

    // fn get_filetype<'a, 'b>(&'a self) -> Pin<Box<dyn Future<Output = Result<FileType, Error>> + Send + 'b>>
    // where  
    //     'a: 'b,
    //     Self: 'b,
    // {
    //     Box::pin(async {
    //         if self.isatty() {
    //             Ok(FileType::CharacterDevice)
    //         } else {
    //             Ok(FileType::Unknown)
    //         }
    //     })
    // }

    fn isatty(&self) -> bool {
        std::io::stdout().is_terminal()
    }
}