mod app;
mod args;
mod database;
mod palette;
mod tabs;
mod widgets;

use app::App;
use console::style;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        original_hook(panic_info);
    }));

    if let Err(err) = App::new().init().run() {
        if is_connection_refused(&*err) {
            eprintln!("\n\n{}\n", style("Connection refused.").red());
            args::print_help();
        } else {
            eprintln!("Error: {err}");
            let mut source = err.source();
            while let Some(e) = source {
                eprintln!("  caused by: {e}");
                source = e.source();
            }
        }
        std::process::exit(1);
    }
}

fn is_connection_refused(err: &(dyn std::error::Error + 'static)) -> bool {
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        return io_err.kind() == std::io::ErrorKind::ConnectionRefused;
    }
    err.source().is_some_and(is_connection_refused)
}
