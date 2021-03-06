pub enum DebuggerCommand {
    Quit,
    Run(Vec<String>),
    Continue,
    BackTrace,
    Breakpoint(String),
}

impl DebuggerCommand {
    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            "c" | "cont" => Some(DebuggerCommand::Continue),
            "bt" | "backtrace" => Some(DebuggerCommand::BackTrace),
            "b" | "break" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Breakpoint(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            // Default case:
            _ => None,
        }
    }
}
