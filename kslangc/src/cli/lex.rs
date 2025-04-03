use super::utils::*;
use anyhow::Context;
use clap::{Command, arg};
use kslang::compiler::lexer::{Lexer, Source, SourceSequence};
use std::{
    io::{BufWriter, Read, Write},
    path::PathBuf,
};

pub fn command() -> Command {
    Command::new("lex")
        .about("词法分析 (默认从标准输入获取源代码， <Crtl-D> 退出)")
        .arg(arg!(-i --input <IN> "源代码输入 <FILE> | <STRING> | stdin（默认）"))
        .arg(arg!(-o --output <OUT> "输出到 <FILE> | stderr | stdout（默认）"))
        .arg(arg!(-f --format <FORMAT> "输出格式 text | debug | html (实验) | json（默认）"))
        .arg(arg!(-l --level <LEVEL> "终止等级 debug | warning | error | fatal（默认）"))
        .arg(arg!(-e --error <ERROR> "错误输出到 <FILE> | stdout | stderr（默认）"))
}

pub fn match_command(matches: &clap::ArgMatches, _verbose: bool) -> anyhow::Result<()> {
    let level = if let Some(level) = matches.get_one("level") {
        let level: &String = level;
        match level.as_str() {
            "debug" => Level::Debug,
            "warning" => Level::Warning,
            "error" => Level::Error,
            "fatal" => Level::Fatal,
            _ => anyhow::bail!("无效的终止等级(`{}`)", level),
        }
    } else {
        Level::default()
    };

    let input = if let Some(input) = matches.get_one("input") {
        let input: &String = input;
        if input.as_str() == "stdin" {
            Input::Stdin
        } else {
            let path = PathBuf::from(input);
            if let Ok(contents) = path.try_exists() {
                if contents {
                    Input::File(path)
                } else {
                    Input::String(input.to_string())
                }
            } else if level.warning() {
                anyhow::bail!("系统错误(try_exists)")
            } else {
                Input::String(input.to_string())
            }
        }
    } else {
        Input::default()
    };

    let output = if let Some(output) = matches.get_one("output") {
        let output: &String = output;
        if output.as_str() == "stdout" {
            Output::Stdout
        } else if output.as_str() == "stderr" {
            Output::Stderr
        } else {
            let path = PathBuf::from(output);
            if let Ok(contents) = path.try_exists() {
                if contents && level.error() {
                    anyhow::bail!("文件 `{}` 已存在", output)
                } else {
                    Output::File(path)
                }
            } else if level.warning() {
                anyhow::bail!("系统错误(try_exists)")
            } else {
                Output::File(path)
            }
        }
    } else {
        Output::default()
    };

    let format = if let Some(format) = matches.get_one("format") {
        let format: &String = format;
        match format.as_str() {
            "json" => Format::Json,
            "text" => Format::Text,
            "html" => Format::Html,
            "debug" => Format::Debug,
            _ => {
                if level.error() {
                    anyhow::bail!("无效的输出格式(`{}`)", format)
                } else {
                    Format::default()
                }
            }
        }
    } else {
        Format::default()
    };

    let error_output = if let Some(error_output) = matches.get_one("error") {
        let error_output: &String = error_output;
        if error_output.as_str() == "stdout" {
            ErrorOutput::Stdout
        } else if error_output.as_str() == "stderr" {
            ErrorOutput::Stderr
        } else {
            let path = PathBuf::from(error_output);
            if let Ok(contents) = path.try_exists() {
                if contents && level.error() {
                    anyhow::bail!("文件 `{}` 已存在", error_output)
                } else {
                    ErrorOutput::File(path)
                }
            } else if level.warning() {
                anyhow::bail!("系统错误(try_exists)")
            } else {
                ErrorOutput::File(path)
            }
        }
    } else {
        ErrorOutput::default()
    };

    let mut out: Box<dyn Write> = match output {
        Output::Stdout => Box::new(std::io::stdout()),
        Output::Stderr => Box::new(std::io::stderr()),
        Output::File(path) => Box::new(BufWriter::new(
            std::fs::File::create(path).context("创建输出文件失败")?,
        )),
    };

    let mut err: Box<dyn Write> = match error_output {
        ErrorOutput::Stdout => Box::new(std::io::stdout()),
        ErrorOutput::Stderr => Box::new(std::io::stderr()),
        ErrorOutput::File(path) => Box::new(BufWriter::new(
            std::fs::File::create(path).context("创建错误输出文件失败")?,
        )),
    };

    let src = match input {
        Input::Stdin => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .context("读取输入失败")?;
            Source::Stdin(buffer)
        }
        Input::String(string) => Source::String(string),
        Input::File(path) => {
            let mut contents = String::new();
            std::fs::File::open(&path)
                .and_then(|mut file| file.read_to_string(&mut contents))
                .context("读取输入文件失败")?;
            Source::File { path, contents }
        }
    };

    let srcs = SourceSequence { sources: vec![src] };
    let lexer = Lexer::new(0, &srcs);

    let mut tokens = Vec::new();
    let mut err_cnt = 0;
    for token in lexer {
        match token {
            Ok(token) => match format {
                Format::Json | Format::Html => tokens.push(token),
                Format::Text => println!("{}", token),
                Format::Debug => println!("{:?}", token),
            },
            Err(e) => {
                let src = &srcs.sources[0];
                let text = srcs.get_text(e);

                writeln!(err, "[Lexer:{}] {}@{}\t`{}`", err_cnt, src, e, text)
                    .context("写入错误输出失败")?;
                if level.error() {
                    anyhow::bail!("词法分析出现错误")
                }
                err_cnt += 1;
            }
        }
    }

    if matches!(format, Format::Json) {
        let json = serde_json::to_string(&tokens).context("序列化 JSON 失败")?;
        out.write_all(json.as_bytes()).context("写入输出失败")?;
    } else if matches!(format, Format::Html) {
        unimplemented!()
    }

    out.flush().context("刷新输出失败")?;
    err.flush().context("刷新错误输出失败")?;

    if err_cnt > 0 && level.error() {
        anyhow::bail!("词法分析出现错误({})", err_cnt)
    }
    Ok(())
}
