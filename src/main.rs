mod editor;
mod terminal;

use std::env;
use std::process;

use editor::Editor;

fn main() {
    let filenames: Vec<String> = env::args().collect();

    if filenames.len() != 2 {
        println!("Usage: poe FILENAME");
        process::exit(1);
    }

    let mut editor = match Editor::new(&filenames[1]) {
        Ok(e) => e,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };
    editor.run();
}
