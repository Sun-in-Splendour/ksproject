use clap::{Command, arg};
use kslang::compiler::lexer::{Lexer, Source, TokenJson};
use std::{
    io::{BufWriter, Read, Write},
    path::PathBuf,
};

pub fn command() -> Command {
    Command::new("lexer")
        .about("词法分析 (默认从标准输入获取源代码， <Crtl-D> 退出)")
        .arg(arg!(-f --file <FILE> "从文件获取源代码"))
        .arg(arg!(-F --format <FORMAT> "指定源代码格式，默认是 json。可选： json | text | debug"))
        .arg(arg!(-o --output <OUTPUT> "输出位置，默认输出到标准输出。可选： stdout | stderr | 文件名"))
        .arg(arg!(-s --string <STR> "从终端参数获取源代码"))
}

enum Output {
    Stdout,
    Stderr,
    File(PathBuf),
}

#[derive(Clone, Copy)]
enum Format {
    Json,
    Text,
    Debug,
}

pub fn match_command(matches: &clap::ArgMatches, verbose: bool) -> anyhow::Result<()> {
    let source = if let Some(path) = matches.get_one("file") {
        let path: &String = path;
        let mut file = std::fs::File::open(path)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;

        let path = std::path::PathBuf::from(path);
        let contents = source.clone();
        Source::File { path, contents }
    } else if let Some(s) = matches.get_one("string") {
        let s: &String = s;
        Source::String(s.clone())
    } else {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source)?;
        Source::Stdin(source)
    };

    let out_to = if let Some(path) = matches.get_one("output") {
        let path: &String = path;

        match path.as_str() {
            "stdout" => Output::Stdout,
            "stderr" => Output::Stderr,
            _ => Output::File(PathBuf::from(path)),
        }
    } else {
        Output::Stdout
    };

    let format = if let Some(format) = matches.get_one("format") {
        let format: &String = format;
        match format.as_str() {
            "json" => Format::Json,
            "text" => Format::Text,
            "debug" => Format::Debug,
            _ => anyhow::bail!("无效的格式参数: {}", format),
        }
    } else {
        Format::Json
    };

    let mut writer: BufWriter<Box<dyn Write>> = match out_to {
        Output::Stdout => BufWriter::new(Box::new(std::io::stdout())),
        Output::Stderr => BufWriter::new(Box::new(std::io::stderr())),
        Output::File(path) => BufWriter::new(Box::new(std::fs::File::create(path)?)),
    };

    let iter = Lexer::new(&source);
    let mut err_cnt = 0;
    if matches!(format, Format::Debug | Format::Text) {
        for token in iter {
            let span = match token {
                Ok(token) => {
                    let s = if let Format::Debug = format {
                        format!("{:?}", token)
                    } else if let Format::Text = format {
                        format!("{}", token)
                    } else {
                        unreachable!()
                    };

                    writer.write_all(s.as_bytes())?;
                    writer.write_all(b"\n")?;
                    writer.flush()?;
                    token.span
                }
                Err(span) => {
                    err_cnt += 1;
                    eprintln!("错误：{}@{}", span, source);
                    span
                }
            };

            if verbose {
                eprintln!("\t{}: {:?}", source, &source.text()[span.start..span.end])
            }
        }
    } else {
        let mut tokens: Vec<TokenJson> = Vec::new();
        for token in iter {
            match token {
                Ok(token) => tokens.push(token.into()),
                Err(span) => {
                    err_cnt += 1;
                    eprintln!("错误：{}", span);
                    if verbose {
                        eprintln!("\t{}: {:?}", source, &source.text()[span.start..span.end])
                    }
                }
            }
        }

        let string = serde_json::to_string(&tokens)?;
        writer.write_all(string.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    writer.flush()?;
    if err_cnt > 0 {
        if verbose {
            eprintln!("发生 {} 个错误", err_cnt);
        }

        anyhow::bail!("词法分析过程中发生错误")
    }

    Ok(())
}
