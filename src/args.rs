use crate::VERSION;
use console::style;

pub struct ArgsStruct {
    pub help: bool,
    pub url: Option<String>,
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
    pub database: String,
}

pub fn get_arguments() -> ArgsStruct {
    let arguments = std::env::args();
    let arguments = arguments::parse(arguments).unwrap();

    ArgsStruct {
        help: arguments.get::<bool>("help").unwrap_or(false),
        url: arguments.get::<String>("url"),
        host: arguments
            .get::<String>("host")
            .unwrap_or(String::from("localhost")),
        port: arguments.get::<u32>("port").unwrap_or(5432),
        user: arguments
            .get::<String>("user")
            .unwrap_or(String::from("postgres")),
        password: arguments
            .get::<String>("password")
            .unwrap_or(String::from("postgres")),
        database: arguments
            .get::<String>("database")
            .unwrap_or(String::from("postgres")),
    }
}

pub fn print_help() {
    println!();
    println!(
        "{} {} - PostgreSQL performance tuning tool.",
        style("Ellie").magenta(),
        style(VERSION).magenta()
    );
    println!();
    println!("Parameters:");
    println!();
    println!(
        "--url <{}>            PostgreSQL connection URL. When provided, overrides all other connection parameters.",
        style("url").cyan(),
    );
    println!();
    println!(
        "--host <{}>          PostgreSQL server hostname or IP address. Default is {}.",
        style("host").cyan(),
        style("localhost").green()
    );
    println!(
        "--port <{}>          PostgreSQL server port number. Default is {}.",
        style("port").cyan(),
        style("5432").green()
    );
    println!(
        "--user <{}>          Username for the database. Default is {}.",
        style("user").cyan(),
        style("postgres").green()
    );
    println!(
        "--password <{}>  Password for the database. Default is {}.",
        style("password").cyan(),
        style("postgres").green()
    );
    println!(
        "--database <{}>  Database name. Default is {}.",
        style("database").cyan(),
        style("postgres").green()
    );
    println!();
    println!("Examples:");
    println!();
    println!(
        "ellie --url {}",
        style("postgresql://postgres:postgres@localhost:5432/postgres").green(),
    );
    println!();
    println!(
        "ellie --host {} --port {} --user {} --password {} --database {}",
        style("localhost").green(),
        style("5432").green(),
        style("postgres").green(),
        style("postgres").green(),
        style("postgres").green()
    );
    println!();
}
