#![forbid(unsafe_code)]

use std::convert::Infallible;
use std::error;
use std::io::{self, Write};
use std::process::exit;

use ren::prelude::*;

fn main() {
    exit({
        let code = match try_main() {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("error: {}", err);
                1
            }
        };
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
        code
    });
}

fn try_main() -> Result<(), Box<dyn error::Error>> {
    ren::run!(MyApp)
}

struct MyApp;

impl<'gl> App<'gl> for MyApp {
    type Err = Infallible;

    fn init(ctx: &mut RenderingContext<'gl>) -> Result<Self, Self::Err> {
        ctx.set_clear_color((1.0, 0.0, 1.0, 1.0));

        Ok(Self)
    }

    fn draw(&mut self, ctx: &mut RenderingContext<'gl>, _wnd: &Window) {
        ctx.clear_color_buffer();
    }
}
