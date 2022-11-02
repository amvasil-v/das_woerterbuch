use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct GameReader {
    reader: Editor<()>,
}

impl GameReader {
    pub fn new() -> Self {
        GameReader {
            reader: Editor::<()>::new().unwrap(),
        }
    }

    pub fn read_line(&mut self) -> Option<String> {
        let res = self.reader.readline(">> ");
        match res {
            Ok(s) => {
                let answer = s.trim().to_lowercase();
                if answer == "exit" || answer == "quit" {
                    return None;
                }
                Some(answer)
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                None
            }
            _ => {
                println!("Readline error");
                None
            }
        }
    }
}
