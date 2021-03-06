mod editor;
mod terminal;

use std::env;
use std::process;

use editor::Editor;

fn main() {
    let filenames: Vec<String> = env::args().collect();

    let mut editor = if filenames.len() == 1 {
        Editor::new_empty()
    } else if filenames.len() == 2 {
        match Editor::new(&filenames[1]) {
            Ok(e) => e,
            Err(err) => {
                println!("{}", err);
                process::exit(1);
            }
        }
    } else {
        println!("Usage: poe [FILENAME]");
        process::exit(1);
    };

    editor.run();
}
