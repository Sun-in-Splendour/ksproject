mod lexer;

use clap::{Command, arg};

pub fn run() -> anyhow::Result<()> {
    let matches = command().get_matches();

    if match_command(&matches)? {
        Ok(())
    } else {
        anyhow::bail!("未知命令")
    }
}

fn command() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(arg!(-v --verbose "启用详细输出"))
        .subcommand(lexer::command())
}

macro_rules! match_subcommands {
    ($m:expr, $v:expr $(=> $($sub:ident),* $(,)?)?) => {
        $($(
            if let Some(sub) = $m.subcommand_matches(stringify!($sub)) {
                $sub::match_command(sub, $v)?; true
            }
        )else * else)? { false }
    };
}

fn match_command(matches: &clap::ArgMatches) -> anyhow::Result<bool> {
    let verbose = matches.get_flag("verbose");
    Ok(match_subcommands!(matches, verbose => lexer))
}
