use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::path::Path;
use execute::Execute;
use std::str;

#[derive(Debug)]
enum Command {
    VERSION,
    APPLICABLE,
    RUN
}

struct AnalyzerArgs {
    dir: String,
    commit: String,
    command: Command,
}

fn main() {
    let args = parse_args();

    match args.command {
        Command::VERSION => { version() }
        Command::APPLICABLE => { applicable(); }
        Command::RUN => {
            let response = run(&args);
            println!("{}", serde_json::to_string(&response).unwrap());
        }
    }
}

fn parse_args() -> AnalyzerArgs {
    assert_eq!(std::env::args().len(), 4);

    AnalyzerArgs {
        dir: std::env::args().nth(1).expect("Missing directory"),
        commit: std::env::args().nth(2).expect("Missing commit hash"),
        command: validate_command()
    }
}

fn validate_command() -> Command {
    let command = std::env::args().nth(3).expect("Missing command");

    match command.as_ref() {
        "version" => Command::VERSION,
        "applicable" => Command::APPLICABLE,
        "run" => Command::RUN,
        _ => panic!("Unknown command type")
    }
}

#[derive(Serialize)]
struct MuseResponse {
    #[serde(rename = "type")]
    type_of: String,
    message: String,
    file: String,
    line: u32
}

#[derive(Deserialize)]
struct Outdated {
    crate_name: String,
    dependencies: Vec<Dependency>
}

#[derive(Deserialize)]
struct Dependency {
    name: String,
    project: String,
    compat: String,
    latest: String,
    kind: DependencyKind
}

#[derive(Deserialize, PartialEq, Debug)]
enum DependencyKind {
    Normal, Development, Build
}

fn version() {
    println!("1")
}

fn applicable() {
    if Path::new("./Cargo.lock").exists() {
        println!("true");
    } else {
        println!("false");
    }
}

fn run(args: &AnalyzerArgs) -> Vec<MuseResponse> {
    let output = execute_outdated_command(&args);

    if !output.1.is_empty() {
        return vec![
                MuseResponse {
                    type_of: "stderr".to_string(),
                    message: str::from_utf8(&output.1).unwrap().to_string(),
                    file: "N/A".to_string(),
                    line: 0
                }
        ];
    }

    let response: Outdated = serde_json::from_str(str::from_utf8(&output.0).unwrap()).unwrap();
    build_muse_response(response)
}

fn execute_outdated_command(_args: &&AnalyzerArgs) -> (Vec<u8>, Vec<u8>) {
    let mut command = std::process::Command::new("/root/.cargo/bin/cargo");
    command.arg("outdated");
    // command.arg("-m");
    // command.arg(&args.dir);
    command.arg("-R");
    command.arg("--format");
    command.arg("json");

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    return match command.execute_output() {
        Ok(output) => { (output.stdout, output.stderr) }
        Err(e) => { (e.to_string().into_bytes(), Vec::new()) }
    }
}

fn build_muse_response(outdated: Outdated) -> Vec<MuseResponse> {

    let response = outdated.dependencies.iter().map(|dependency| {
        MuseResponse {
            type_of: "Out of date".to_string(),
            message: build_muse_message(dependency),
            file: "Cargo.toml".to_string(),
            line: 0
        }
    }).collect();

    response
}

fn build_muse_message(dependency: &Dependency) -> String {
    format!("### {}\nVersion is at {} but could be upgrade to {}",
                      dependency.name, dependency.project, dependency.latest)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_with_no_dependencies() {
        let expected = r###"
        {
            "crate_name": "foobar",
            "dependencies": []

        }"###;

        let actual: Outdated = serde_json::from_str(expected).unwrap();

        assert_eq!(actual.crate_name, "foobar");
        assert_eq!(actual.dependencies.len(), 0);
    }

    #[test]
    fn can_parse_with_one_dependency() {
        let expected = r###"
        {
            "crate_name": "foobar",
            "dependencies": [
                {
                    "name": "baz",
                    "project": "1.2.3",
                    "compat": "---",
                    "latest": "3.0.0",
                    "kind": "Development",
                    "platform": "null"
                }
            ]
        }
        "###;

        let actual: Outdated = serde_json::from_str(expected).unwrap();

        assert_eq!(actual.dependencies.len(), 1);
        assert_eq!(actual.dependencies[0].name, "baz");
        assert_eq!(actual.dependencies[0].project, "1.2.3");
        assert_eq!(actual.dependencies[0].compat, "---");
        assert_eq!(actual.dependencies[0].latest, "3.0.0");
        assert_eq!(actual.dependencies[0].kind, DependencyKind::Development);
    }

    #[test]
    fn can_parse_multiple_dependencies() {
        let expected = r###"
        {
           "crate_name":"muse-rust-analyzer",
           "dependencies":[
              {
                 "name":"execute-command-macro-impl->syn",
                 "project":"1.0.68",
                 "compat":"1.0.69",
                 "latest":"1.0.69",
                 "kind":"Normal",
                 "platform":null
              },
              {
                 "name":"serde_derive->syn",
                 "project":"1.0.68",
                 "compat":"1.0.69",
                 "latest":"1.0.69",
                 "kind":"Normal",
                 "platform":null
              }
           ]
        }
        "###;

        let actual: Outdated = serde_json::from_str(expected).unwrap();

        assert_eq!(actual.crate_name, "muse-rust-analyzer");
        assert_eq!(actual.dependencies.len(), 2);
    }
}
