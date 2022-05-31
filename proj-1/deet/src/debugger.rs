use crate::debugger_command::DebuggerCommand;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use crate::inferior::Inferior;
use nix::sys::ptrace;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: Vec<usize>,
}

fn parse_address(addr: &str) -> Option<usize> {
    let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
        &addr[2..]
    } else {
        &addr
    };
    usize::from_str_radix(addr_without_0x, 16).ok()
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        debug_data.print();  // for debug

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints: vec![],
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    self.kill_inferior_if_exists().unwrap();
                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.breakpoints) {
                        // Create the inferior
                        self.inferior = Some(inferior);

                        let inferior = self.inferior.as_mut().unwrap();
                        match inferior.continue_run(None) {
                            Ok(status) => match status {
                                crate::inferior::Status::Stopped(signal, rip) => {
                                    println!("Child stopped (signal {:?})", signal);
                                    let _line = self.debug_data.get_line_from_addr(rip);
                                    let _func = self.debug_data.get_function_from_addr(rip);
                                    if _line.is_some() && _func.is_some() {
                                        println!(
                                            "Stopped at {} {}",
                                            _func.unwrap(),
                                            _line.unwrap()
                                        );
                                    }
                                }
                                crate::inferior::Status::Exited(code) => {
                                    println!("Child exited (status {})", code)
                                }
                                crate::inferior::Status::Signaled(_) => todo!(),
                            },
                            Err(e) => {
                                let regs = ptrace::getregs(inferior.pid()).unwrap();

                                let _line = self.debug_data.get_line_from_addr(regs.rip as usize);
                                let _func =
                                    self.debug_data.get_function_from_addr(regs.rip as usize);
                            }
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Continue => {
                    let inferior = self.inferior.as_mut().unwrap();
                    inferior.continue_run(None).unwrap();
                }
                DebuggerCommand::BackTrace => {
                    if let Some(inferior) = self.inferior.as_mut() {
                        inferior.print_backtrace(&self.debug_data).unwrap();
                    } else {
                        println!("There isn't any child process")
                    }
                }
                DebuggerCommand::Quit => {
                    self.kill_inferior_if_exists().unwrap();
                    return;
                }
                DebuggerCommand::Breakpoint(addr) => {
                    // let inferior = self.inferior.as_mut().unwrap();
                    if addr.starts_with("*") {
                        match parse_address(&addr[1..]) {
                            Some(addr) => self.breakpoints.push(addr),
                            None => {
                                println!("Invalid breakpoints: {}", addr)
                            }
                        }
                    }
                }
            }
        }
    }

    fn kill_inferior_if_exists(&mut self) -> Result<(), std::io::Error> {
        if let Some(inferior) = self.inferior.as_mut() {
            println!("Killing running inferior (pid {})", inferior.pid());
            inferior.kill()?;
        }
        Ok(())
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
