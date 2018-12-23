extern crate clap;
extern crate hyper;
extern crate ini;
extern crate shellexpand;

use clap::{App, Arg};
use hyper::{Body, Request, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use ini::Ini;

#[derive(Debug)]
struct Configuration {
    config_file_name: Option<String>,
    tile_base: Option<String>,
    gpx_base: Option<String>
}

const PHRASE: &str = "Hello, World!";

fn hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from(PHRASE))
}

fn expand_env(option: Option<&String>) -> Option<String> {
    option.map(|value| shellexpand::env(value).unwrap().into_owned())
}

fn parse_config_file(config_file_name: &Option<String>) -> Configuration {
    match config_file_name {
        None => {
            Configuration{ config_file_name: None, tile_base: None, gpx_base: None }
        },
        Some(file_name) => {
            let config = Ini::load_from_file(file_name)
                .expect("Config file name could not be found");
            let section: Option<String> = None;
            let section = config.section(section).unwrap();
            let tile_base: Option<&String> = section.get("TileBase");
            let gpx_base: Option<&String> = section.get("GpxBase");
            Configuration {
                config_file_name: None,
                tile_base: expand_env(tile_base),
                gpx_base: expand_env(gpx_base)
            }
        }
    }
}

fn parse_command_line() -> Configuration {

    let matches = App::new("My Test Program")
        .version("")
        .author("")
        .about("")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .takes_value(true)
             .help("File name of config file"))
        .arg(Arg::with_name("tile-base")
             .short("t")
             .long("tile-base")
             .takes_value(true)
             .help("Base directory of tile cache"))
        .arg(Arg::with_name("gpx-base")
             .short("g")
             .long("gpx-base")
             .takes_value(true)
             .help("Base directory of GPX file store"))
        .get_matches();

    let config_file_name = matches.value_of("config").map(|value| String::from(value));
    let tile_base = matches.value_of("tile-base").map(|value| String::from(value));
    let gpx_base = matches.value_of("gpx-base").map(|value| String::from(value));
    return Configuration{config_file_name, tile_base, gpx_base};
}

impl Configuration {
    // merge two configurations; self takes precedence
    fn merge(&self, other: &Configuration) -> Configuration {
        Configuration{
            config_file_name: match self.config_file_name {
                Some(_) => self.config_file_name.clone(),
                None => other.config_file_name.clone()
            },
            tile_base: match self.tile_base {
                Some(_) => self.tile_base.clone(),
                None => other.tile_base.clone()
            },
            gpx_base: match self.gpx_base {
                Some(_) => self.gpx_base.clone(),
                None => other.gpx_base.clone()
            }
        }
    }
}


fn main() {

    let config_cli = parse_command_line();
    let config_file = parse_config_file(&config_cli.config_file_name);
    let config = config_cli.merge(&config_file);

    println!("{:?}", config);

    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`
        service_fn_ok(hello_world)
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    print!("server started at http://localhost:3000");

    // Run this server for... forever!
    hyper::rt::run(server);

}
