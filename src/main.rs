
use std::env::VarError;
use std::fmt::{Display, Debug};
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

use anyhow::Result;
use serde_json::Value;
use clap::{command, Command, arg, ArgAction};
use openprovider::{Client, Builder};
use openprovider::json::ValueExt;
use openprovider::io_result_ext::IOResultExt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Credentials {
    username: String,
    password: String,
    token: Option<String>,
}

fn get_env_string<S: AsRef<str>>(name: S) -> Option<String> {
    let name_ref = name.as_ref();
    match std::env::var(name_ref) {
        Ok(string) => Some(string),
        Err(VarError::NotUnicode(_)) => {
            log::warn!("environment variable {} was set but could not be decoded, so it is ignored.", name_ref);
            None
        },
        Err(VarError::NotPresent) => None,
    }
}

fn output<T: Debug + ?Sized + Serialize>(value: &T, format: Format) {
    match format {
        Format::JSON => println!("{}", serde_json::to_string_pretty(value).unwrap()),
        Format::HumanReadable => println!("{:#?}", value),
    }
}

enum Format {
    HumanReadable,
    JSON,
}

#[tokio::main]
async fn main() -> Result<()> {

    env_logger::init();

    let matches = command!()
        .arg(arg!(--json "Output data as JSON").action(ArgAction::SetTrue))
        .arg(arg!(--human "Output data as human-readable text").action(ArgAction::SetTrue))
        .subcommand(
            Command::new("login")
                .about("authenticate with OpenProvider")
                .arg(arg!(<username> "The username you use to login"))
                .arg(arg!(<password> "The password you use to login"))
                .arg(arg!(--save "Save credentials in $HOME for later use").action(ArgAction::SetTrue))
        )
        .subcommand(
            Command::new("zone")
                .about("manage DNS zones")
                .subcommand(
                    Command::new("list")
                        .about("list all DNS zones")
                )
                .subcommand(
                    Command::new("info")
                        .about("get specific information about a DNS zone")
                        .arg(arg!(<name> "The name of the zone"))
                )
        )
        .subcommand(
            Command::new("record")
                .about("manage DNS records")
                .subcommand(
                    Command::new("list")
                        .about("list all DNS records of a specific zone")
                        .arg(arg!(<name> "ID of the DNS zone that should be inspected"))
                )
                .subcommand(
                    Command::new("set")
                        .about("update a DNS record")
                        .arg(arg!(<zone_id> "ID of the DNS zone the record belongs to"))
                )
        )
        .get_matches();

    let mut output_format = Format::HumanReadable;

    let config_dir = dirs::config_dir().unwrap_or(std::env::current_dir().unwrap()).join("openprovider-cli");

    let mut username = std::env::var("OPENPROVIDER_USERNAME").ok();
    let mut password = std::env::var("OPENPROVIDER_PASSWORD").ok();
    let mut token = std::env::var("OPENPROVIDER_TOKEN").ok();

    let mut max_retries: u32 = 5;

    let credentials_file = config_dir.join("credentials.toml");
    match std::fs::read_to_string(&credentials_file).ok_not_found()? {
        None => {},
        Some(string) => {
            let creds: Credentials = serde_json::from_str(&string)?;
            username = Some(creds.username);
            password = Some(creds.password);
            token = creds.token;
        },
    }

    let settings_file = config_dir.join("config.toml");
    match std::fs::read_to_string(settings_file).ok_not_found()? {
        None => {},
        Some(string) => {
            let settings: Value = serde_json::from_str(&string)?;
            match settings.get("max_retries") {
                None => {},
                Some(value) => { max_retries = value.as_u32_ok().unwrap() },
            }
        },
    }

    username = username.or(get_env_string("OPENPROVIDER_USERNAME"));
    password = password.or(get_env_string("OPENPROVIDER_PASSWORD"));
    token = token.or(get_env_string("OPENPROVIDER_TOKEN"));

    if matches.get_flag("json") {
        output_format = Format::JSON;
    }
    if matches.get_flag("human") {
        output_format = Format::HumanReadable;
    }

    let mut client = Builder::new()
        .max_retries(max_retries)
        .token(token)
        .build();

    match matches.subcommand() {
        Some(("login", matches)) => {
            let username = matches.get_one::<String>("username").unwrap();
            let password = matches.get_one::<String>("password").unwrap();
            let save = matches.get_flag("save");
            match client.login(username, password).await {
                Ok(token) => {
                    println!("Authentication successful!");
                    if save {
                        std::fs::create_dir_all(config_dir)?;
                        let writer = std::fs::File::create(&credentials_file)?;
                        serde_json::to_writer_pretty(writer, &Credentials {
                            username: username.to_string(),
                            password: password.to_string(),
                            token: Some(token),
                        })?;
                        std::fs::set_permissions(&credentials_file, Permissions::from_mode(0o600))?;
                        println!("Credentials written to {:?}", credentials_file);
                    }
                },
                Err(why) => eprintln!("Authentication failed: {}", why)
            }
        },
        Some(("zone", matches)) => match matches.subcommand() {
            Some(("list", _)) => {
                eprintln!("{:#?}", client.list_zones().await.unwrap());
            },
            Some(("info", matches)) => {
                let name = matches.get_one::<String>("name").unwrap();
                output(&client.get_zone(name, false).await.unwrap(), output_format);
            },
            None => eprintln!("Please provide a subcommand."),
            _ => eprintln!("Unrecognised subcommand. Please check your spelling."),
        },
        Some(("record", matches)) => match matches.subcommand() {
            Some(("list", matches)) => {
                let name = matches.get_one::<String>("name").unwrap();
                output(&client.get_zone(name, true).await.unwrap().records.unwrap(), output_format);
            },
            Some(("set", _matches)) => unimplemented!(),
            None => eprintln!("Please provide a subcommand."),
            _ => eprintln!("Unrecognised subcommand. Please check your spelling."),
        },
        None => eprintln!("Please provide a command."),
        _ => eprintln!("Unrecognised command. Please check your spelling."),
    }

    Ok(())

}
