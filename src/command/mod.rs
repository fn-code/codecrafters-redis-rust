use ping::Ping;
use echo::Echo;
use crate::command::info::Info;
use crate::command::get::Get;
use crate::command::replconf::Replconf;
use crate::command::set::Set;
use crate::command::psync::Psync;
use crate::value::Value;

pub mod ping;
pub mod echo;
pub mod get;
pub mod set;
pub mod info;
pub mod replconf;
pub mod psync;

pub(crate) enum Command {
    Ping(Ping),
    Echo(Echo),
    Get(Get),
    Set(Set),
    Info(Info),
    Replconf(Replconf),
    Psync(Psync),
    Unknown
}

impl Command {
    pub(crate) fn parse(value: Value) -> Self {
        let (cmd, args) = Value::parse_command(value).unwrap_or_default();

        match cmd.as_str() {
            "ping" => Command::Ping(Ping::new()),
            "echo" => Command::Echo(Echo::new(args)),
            "get" => Get::new(args).map_or(Command::Unknown, Command::Get),
            "set" => Set::new(args).map_or(Command::Unknown, Command::Set),
            "info" => Command::Info(Info::new()),
            "replconf" =>  Command::Replconf(Replconf::new(args)),
            "psync" => Command::Psync(Psync::new(args)),
            _ => Command::Unknown
        }
    }

}