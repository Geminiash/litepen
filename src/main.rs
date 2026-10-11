//! 轻笔 · LitePen — 入口。
//!
//! 一份 Markdown 进，一封像信的 HTML 出。单二进制，无运行时依赖。

mod cli;
mod render;
mod serve;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("litepen: {e}");
            ExitCode::FAILURE
        }
    }
}
