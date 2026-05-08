pub mod echo;

pub enum Command<'a> {
    Exit,
    Echo(Vec<&'a str>),
    Type(Vec<&'a str>),
    Unkown,
}

impl<'a> Command<'a> {
    pub fn from_raw(input: &'a str) -> Command<'a> {
        let args: Vec<&'a str> = input.trim().split_whitespace().collect();

        match args[0] {
            "exit" => Command::Exit,

            "echo" => Command::Echo(args[1..].to_vec()),

            "type" => Command::Type(args[1..].to_vec()),

            _ => Command::Unkown,
        }
    }
}
