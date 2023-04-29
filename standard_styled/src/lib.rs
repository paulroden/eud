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


pub struct CommandParts
{
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone)]
enum Message {
    StdOut(String),
    StdErr(String),
    None,
}


struct View {
    message: Arc<Mutex<Message>>,
    ticker: Arc<AtomicUsize>,
    style: Style,
}

impl View {
    fn with_style(style: Style) -> Self {
        Self {
            message: Arc::new(Mutex::new(Message::None)),
            ticker: Arc::new(AtomicUsize::new(0)),
            style,
        }
    }
    
    fn tick(&self) -> usize {
        self.ticker.fetch_add(1, Ordering::SeqCst)
    }

    fn update_message(&mut self, msg: &Message) {
        let mut prev = self.message.lock().unwrap();
        *prev = msg.clone();
    }

    fn get_frame(&self) -> String {
        let count = self.ticker.load(Ordering::SeqCst);
        self.style.spinner_frame(count)
    }
    
    fn display(&self) -> String {
        let styled_msg = match self.message.lock().unwrap().deref() {
            Message::StdOut(msg) => (self.style.stdout_style)(msg),
            Message::StdErr(msg) => (self.style.stderr_style)(msg),
            Message::None => "".into(),
        };
        format!(
            "{} {:>10}",
            &self.get_frame(),
            styled_msg,
        )
    }
    fn print(&self) {
        print!("\r{}", self.display());
        std::io::stdout().flush().expect("cannot flush stdout");
    }
}

// TODO: no pub here. maybe try builder ptn
pub struct Style {
    pub spinner: Vec<&'static str>,
    pub stdout_style: Box<dyn Fn(&str) -> ColoredString>,
    pub stderr_style: Box<dyn Fn(&str) -> ColoredString>,    
}

impl Style {
    pub fn new(
        spinner: Vec<&'static str>,
        stdout_style: Box<dyn Fn(&str) -> ColoredString>,
        stderr_style: Box<dyn Fn(&str) -> ColoredString>,
    ) -> Self {
        Self {
            spinner,
            stdout_style,
            stderr_style,
        }
    }

    pub fn spinner_frame(&self, k: usize) -> String {
        let n = self.spinner.len();
        let frame = k % n;
        // unwrap safe because % ensures `frame` is bounded at `n`
        self.spinner.get(frame).unwrap().to_string()
    }
}


pub async fn standard_styled(
    command: CommandParts,
    style: Style
) -> std::io::Result<()> {
    
    let mut view = View::with_style(style);

    let mut child = Command::new(command.program)
        .args(command.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().expect("no stdout!"); // TODO, create or don't based on take() -> Some() | None
    let child_stderr = child.stderr.take().expect("no stderr!"); // ibid.
    
    let mut stderr_reader = BufReader::new(child_stderr).lines();
    let mut stdout_reader = BufReader::new(child_stdout).lines();

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
    
    Ok(())
}


#[cfg(test)]
mod tests {
    

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
