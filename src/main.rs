//! A command line client for taiga.io, inspired by taskwarrior.

mod cli;
mod commands;
mod taiga;
mod utils;

use anyhow::Result;

use cli::{parse_args, TaigaCmd};
use taiga::{Error, Taiga};

fn main() -> Result<()> {
    let cached = Taiga::from_cache()?;

    match parse_args(cached.as_ref()) {
        TaigaCmd::Login(args) => {
            Taiga::auth(args.address)?;
            Ok(())
        }
        cmd => commands::run(cmd, cached.ok_or(Error::NotLoggedIn)?),
    }
}
