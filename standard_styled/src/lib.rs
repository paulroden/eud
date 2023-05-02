// Derived from https://stackoverflow.com/a/55565595
use std::{
    io::Write,
    ops::Deref,
    process::Stdio,
    sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex},
};
use tokio::{
    io::{BufReader, AsyncBufReadExt},
    process::Command,
};
use colored::ColoredString;
pub use colored::Colorize; // re-exported for creation of styles (TODO: develop reasonable semantics for this)


#[derive(Clone)]
pub struct CommandParts
{
    program: String,
    args: Vec<String>,
}

impl CommandParts {
    pub fn new(program: &String, args: &Vec<String>) -> Self {
        Self {
            program: program.into(),
            args: args.clone(),
        }
    }

    pub fn program(&self) -> String {
        self.program.clone()
    }
    pub fn args(&self) -> std::slice::Iter<'_, String> {
        self.args.iter()
    }
}


// TODO: no pub here. maybe try builder ptn
pub struct Style {
    pub spinner: Vec<&'static str>,
    pub stdout_style: Box<dyn Fn(&str) -> ColoredString>,
    pub stderr_style: Box<dyn Fn(&str) -> ColoredString>,
    pub message_style: Box<dyn Fn(&str) -> ColoredString>,
    pub end_message: Option<String>,
}

impl Style {
    pub fn new(
        spinner: Vec<&'static str>,
        stdout_style: Box<dyn Fn(&str) -> ColoredString>,
        stderr_style: Box<dyn Fn(&str) -> ColoredString>,
        message_style: Box<dyn Fn(&str) -> ColoredString>,
        end_message: Option<String>,
    ) -> Self {
        Self {
            spinner,
            stdout_style,
            stderr_style,
            message_style,
            end_message,
        }
    }
    
    pub fn spinner_frame(&self, k: usize) -> String {
        let n = self.spinner.len();
        let frame = k % n;
        // unwrap safe because % ensures `frame` is bounded at `n`
        self.spinner.get(frame).unwrap().to_string()
    }
}




#[derive(Clone)]
enum Message {
    StdOut(String),
    StdErr(String),
    None,
}


struct View<'s> {
    message: Arc<Mutex<Message>>,
    ticker: Arc<AtomicUsize>,
    style: &'s Style,
}

impl<'s> View<'s> {
    fn display(&self) -> String {
        let styled_msg = match self.message.lock().unwrap().deref() {
            Message::StdOut(msg) => (self.style.stdout_style)(msg),
            Message::StdErr(msg) => (self.style.stderr_style)(msg),
            Message::None => "".into(),
        };
        format!(
            "{} {}",
            &self.get_frame(),
            styled_msg,
        )
    }
    
    fn get_frame(&self) -> String {
        let count = self.ticker.load(Ordering::SeqCst);
        self.style.spinner_frame(count)
    }

    fn print(&self) {
        self.clear();
        print!("\r{}", self.display());
        std::io::stdout().flush().expect("cannot flush stdout");
    }

    fn clear(&self) {
        print!("\x1B[2K");
        print!("\r");
        std::io::stdout().flush().expect("cannot flush stdout");
    }
    
    fn show_end_message(&self) {
        match &self.style.end_message {
            None => (),
            Some(message) => {
                self.clear();
                println!("{}", (self.style.message_style)(message) );
            }
        }
    }
    
    fn tick(&self) -> usize {
        self.ticker.fetch_add(1, Ordering::SeqCst)
    }
    fn update_message(&mut self, msg: &Message) {
        let mut prev = self.message.lock().unwrap();
        *prev = msg.clone();
    }
    fn with_style(style: &'s Style) -> Self {
        Self {
            message: Arc::new(Mutex::new(Message::None)),
            ticker: Arc::new(AtomicUsize::new(0)),
            style,
        }
    }
}

pub async fn standard_styled(
    command: CommandParts,
    style: &Style
) -> std::io::Result<()> {
    
    let mut child = Command::new(command.program)
        .args(command.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().expect("no stdout!"); // TODO, create or don't based on take() -> Some() | None
    let child_stderr = child.stderr.take().expect("no stderr!"); // ibid.
    
    let mut stderr_reader = BufReader::new(child_stderr).lines();
    let mut stdout_reader = BufReader::new(child_stdout).lines();

    let mut view = View::with_style(style);
    view.print();
    
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let msg = Message::StdOut(line);
                        view.update_message(&msg);
                        view.tick();
                        view.print();
                    }
                    Ok(None) => break,
                    Err(_) => (),
                }
            },
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let msg = Message::StdErr(line);
                        view.update_message(&msg);
                        view.tick();
                        view.print();
                    }
                    Err(_) => (),
                    _ => (),
                }
            }
        }
    }

    // command has completed, play ending message (if there is one)
    view.show_end_message();
    
    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::CommandParts;



    #[test]
    fn test_stdout_out_only() {
        let cat_stdout = CommandParts::new(
            &"cat".to_string(),
            &vec!["abc".to_string()]
        );
        println!("{:?}", cat_stdout.program());
    }
}
