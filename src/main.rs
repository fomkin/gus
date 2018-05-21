#[macro_use]
extern crate serde_derive;

extern crate clap;

use clap::{Arg, App, SubCommand};
use std::process::{Command, Stdio};

mod config;
mod common;

const COMMAND_LIST: &'static str = "list";
const COMMAND_NEW: &'static str = "new";
const COMMAND_REMOVE: &'static str = "remove";
const COMMAND_MODIFY: &'static str = "modify";
const COMMAND_LOCAL: &'static str = "local";
const COMMAND_GLOBAL: &'static str = "global";

const ARG_ALIAS: &'static str = "alias";
const ARG_NAME: &'static str = "name";
const ARG_EMAIL: &'static str = "email";
const ARG_SIGNINGKEY: &'static str = "signingkey";

fn main() {

    let alias = Arg::with_name(ARG_ALIAS)
        .help("Gus user alias")
        .required(true);

    let clap_config = App::new("Gus - Git User Switcher")
        .version("0.1.0")
        .author("Aleksey Fomkin <aleksey.fomkin@gmail.com>")
        .about("Switch between GIT identities")
        .arg(Arg::with_name("config")
            .long("config")
            .value_name("FILE")
            .takes_value(true)
            .help("Path to Gus config file")
        )
        .subcommand(
            SubCommand::with_name(COMMAND_NEW)
                .about("Creates new Git user")
                .arg(Arg::with_name(ARG_NAME).help("user's name (example: John Doe)").required(true).index(2))
                .arg(Arg::with_name(ARG_EMAIL).help("user's email (example: john.doe@exmaple.com)").required(true).index(3))
                .arg(Arg::with_name(ARG_SIGNINGKEY).help("user's GPG signing key").required(false).index(4))
                .arg(alias.clone().index(1))
        )
        .subcommand(SubCommand::with_name(COMMAND_MODIFY)
            .about("Update user in Gus config")
            .arg(Arg::with_name(ARG_NAME).long(ARG_NAME).help("user's name (example: John Doe)").takes_value(true))
            .arg(Arg::with_name(ARG_EMAIL).long(ARG_EMAIL).help("user's email (example: john.doe@exmaple.com)").takes_value(true))
            .arg(Arg::with_name(ARG_SIGNINGKEY).long(ARG_SIGNINGKEY).help("user's GPG signing key").takes_value(true))
            .arg(alias.clone())
        )
        .subcommand(SubCommand::with_name(COMMAND_LIST)
            .about("Shows users managed by Gus")
        )
        .subcommand(SubCommand::with_name(COMMAND_REMOVE)
            .about("Removes user from Gus config")
            .arg(alias.clone())
        )
        .subcommand(SubCommand::with_name(COMMAND_LOCAL)
            .about("Sets user to the repository config")
            .arg(alias.clone())
        )
        .subcommand(SubCommand::with_name(COMMAND_GLOBAL)
            .about("Sets user to global GIT config")
            .arg(alias.clone())
        );

    let matches = clap_config.clone().get_matches();

    //println!("config.path: {:?}, config.data: {}", _config_path.as_path(), _config_data);
    // matches.value_of("config")

    let config_path = config::path(matches.value_of("config"));
    let config_data = config::read(config_path.clone());
    let mut _config = config::parse(config_data);

    fn print_help(mut clap_config: clap::App) {
        clap_config
            .print_help()
            .expect("");
        println!()
    }

    fn set_git_config(user: &common::User, global: bool) {
        user.to_cmd().iter().for_each(|xs|{
            let mut args = vec!["config".to_string()];
            if global {
                args.push("--global".to_string())
            }
            xs.iter().for_each(|x|args.push(x.to_string()));
            Command::new("git")
                .args(args)
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stdin(Stdio::null())
                .output()
                .expect("failed to execute process");
        });
    }

    match matches.subcommand {
        Some(subcommand) => {
            let name = subcommand.name.clone();
            let matches = subcommand.matches;
            match name.as_ref() {
                COMMAND_NEW => {
                    _config.insert(
                        matches
                            .value_of(ARG_ALIAS)
                            .unwrap()
                            .to_string(),
                        common::User {
                            name: matches
                                .value_of(ARG_NAME)
                                .unwrap()
                                .to_string(),
                            email: matches
                                .value_of(ARG_EMAIL)
                                .unwrap()
                                .to_string(),
                            signingkey:matches
                                .value_of(ARG_SIGNINGKEY)
                                .map(|x| x.to_string())
                        }
                    );
                    config::save(config_path, _config)
                },
                COMMAND_MODIFY => {
                    let alias = matches.value_of(ARG_ALIAS).unwrap().to_string();
                    let maybe_user = _config.get(&alias).map(|user| {
                        let mut mutable_user = user.clone();
                        mutable_user.change(
                            matches.value_of(ARG_NAME).map(|s| s.to_string()),
                            matches.value_of(ARG_EMAIL).map(|s| s.to_string()),
                            matches.value_of(ARG_SIGNINGKEY).map(|s| s.to_string())
                        );
                        mutable_user
                    });
                    match maybe_user {
                        Some(user) => {
                            _config.insert(alias.clone(), user);
                            config::save(config_path, _config);
                        }
                        None =>
                            println!("{} doesn't exist", alias)
                    }
                },
                COMMAND_LIST => {
                    _config.iter().for_each(|(alias, user)| {
                        println!("{}: {} <{}>{}", alias, user.name, user.email,
                                 user.signingkey
                                     .clone()
                                     .map(|s| format!(", GPG signing key: {}", s))
                                     .unwrap_or("".to_string())
                        )
                    })
                },
                COMMAND_REMOVE => {
                    let alias = matches.value_of(ARG_ALIAS).unwrap().to_string();
                    if _config.contains_key(&alias) {
                        _config.remove(&alias);
                        config::save(config_path, _config)
                    } else {
                        println!("{} doesn't exist", alias)
                    }
                },
                COMMAND_LOCAL => {
                    let alias = matches.value_of(ARG_ALIAS).unwrap().to_string();
                    match _config.get(&alias) {
                        Some(user) => set_git_config(user, false),
                        None => println!("{} doesn't exist", alias)
                    }
                }
                COMMAND_GLOBAL => {
                    let alias = matches.value_of(ARG_ALIAS).unwrap().to_string();
                    match _config.get(&alias) {
                        Some(user) => set_git_config(user, true),
                        None => println!("{} doesn't exist", alias)
                    }
                }
                _ => {
                    print_help(clap_config.clone());
                }
            }
        },
        None => {
            print_help(clap_config.clone());
        }
    }
}
