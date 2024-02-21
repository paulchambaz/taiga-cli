use crate::taiga::taiga::Taiga;

extern crate sha1;

mod cli;
mod taiga;
mod utils;

fn main() {
    let taiga = Taiga::load();
}
