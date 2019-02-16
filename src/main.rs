extern crate clap;
extern crate reqwest;
extern crate serde;
use clap::{App, Arg, SubCommand};
use serde::Deserialize;
use serde_json::Value;
use std::error;
use std::fmt;
use std::process;

struct ApiRequest {
    lamin: f64,
    lomin: f64,
    lamax: f64,
    lomax: f64,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    states: Vec<Vec<ApiResponseField>>,
}

#[derive(Debug, Deserialize)]
enum ApiResponseField {
    Str(String),
    Int(u64),
    Float(f64),
    Bool(bool),
    Ints(Vec<u64>),
}

#[derive(Debug)]
struct ApiState {
    icao24: String,
    callsign: String,
    origin_country: String,
    time_position: Option<u64>,
    last_contact: u64,
    longitude: Option<f64>,
    latitude: Option<f64>,
    baro_altitude: Option<f64>,
    on_ground: bool,
    velocity: Option<f64>,
    true_track: Option<f64>,
    vertical_rate: Option<f64>,
    sensors: Vec<u64>,
    geo_altitude: f64,
    squawk: Option<String>,
    spi: bool,
    position_source: u8,
}

#[derive(Debug)]
struct GetApiError {
    message: String,
}

impl GetApiError {
    fn new(message: String) -> GetApiError {
        GetApiError { message }
    }
}

impl fmt::Display for GetApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for GetApiError {
    fn description(&self) -> &str {
        self.message.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

fn main() {
    let app = App::new("sky-cli")
        .version("0.1")
        .author("Ryan Faulhaber <faulhaberryan@gmail.com>")
        .about("calls Open Sky API")
        .subcommand(
            SubCommand::with_name("nearest")
                .about("finds nearest plane(s)")
                .help("finds nearest plane(s)")
                .arg(
                    Arg::with_name("count")
                        .short("c")
                        .long("count")
                        .help("lists c entries")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("sort")
                        .short("s")
                        .long("sort")
                        .help("sort field")
                        .takes_value(true),
                ),
        );

    if let Some(matches) = app.get_matches().subcommand_matches("nearest") {
        let count_str = matches.value_of("count").unwrap_or("1");

        let api_result = match get_json_data() {
            Ok(resp) => resp,
            Err(reason) => {
                eprintln!("get json data failed: {}", reason);
                process::exit(1);
            }
        };

        println!("result: {:?}", api_result);
    }
}

fn get_json_data() -> Result<Vec<ApiState>, GetApiError> {
    // TODO specify GPS boundries

    let mut resp_data = match reqwest::get("https://opensky-network.org/api/states/all") {
        Ok(data) => data,
        Err(reason) => return Err(GetApiError::new(String::from("request failed"))),
    };

    let states: Value = serde_json::from_str(resp_data.text().unwrap().as_str()).unwrap();

    let json_data: ApiResponse = match serde_json::from_value(states) {
        Ok(data) => data,
        Err(reason) => {
            println!("reason: {}", reason);
            return Err(GetApiError::new(String::from("json parse failed")));
        }
    };

    println!("states: {:?}", json_data);

    // let states = json_data["states"];

    unimplemented!();

    // let responses: Vec<ApiResponse> = match serde_json::from_value(states) {
    //     Ok(data) => data,
    //     Err(reason) => {
    //         println!("reason: {}", reason);
    //         return Err(GetApiError::new(String::from("value parse failed")));
    //     }
    // };

    // Ok(responses)
}
