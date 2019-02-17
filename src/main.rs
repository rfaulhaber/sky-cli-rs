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
    states: Vec<Vec<Value>>,
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
    callsign: Option<String>,
    origin_country: String,
    time_position: Option<i64>,
    last_contact: u64,
    longitude: Option<f64>,
    latitude: Option<f64>,
    baro_altitude: Option<f64>,
    on_ground: bool,
    velocity: Option<f64>,
    true_track: Option<f64>,
    vertical_rate: Option<f64>,
    sensors: Vec<u64>,
    geo_altitude: Option<f64>,
    squawk: Option<String>,
    spi: bool,
    position_source: u8,
}

impl ApiState {
    fn default() -> ApiState {
        ApiState {
            icao24: String::new(),
            callsign: None,
            origin_country: String::new(),
            time_position: None,
            last_contact: 0 as u64,
            longitude: None,
            latitude: None,
            baro_altitude: None,
            on_ground: false,
            velocity: None,
            true_track: None,
            vertical_rate: None,
            sensors: Vec::new(),
            geo_altitude: None,
            squawk: None,
            spi: false,
            position_source: 0 as u8,
        }
    }
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

type Coordinate = (f64, f64);

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
                )
                .arg(Arg::with_name("latitude").required(true))
                .arg(Arg::with_name("longitude").required(true)),
        );

    if let Some(matches) = app.get_matches().subcommand_matches("nearest") {
        let count: usize = match matches.value_of("count").unwrap_or("1").parse() {
            Ok(val) => val,
            Err(reason) => {
                eprintln!("could not parse count argument: {}", reason);
                process::exit(1);
            }
        };

        // TODO actually verify that floats are parsed so as to prevent panic!
        let latitude: f64 = matches.value_of_lossy("latitude").unwrap().parse().unwrap();
        let longitude: f64 = matches
            .value_of_lossy("longitude")
            .unwrap()
            .parse()
            .unwrap();

        let api_result = match get_json_data() {
            Ok(resp) => resp,
            Err(reason) => {
                eprintln!("get json data failed: {}", reason);
                process::exit(1);
            }
        };

        println!("lat: {}, long: {}", latitude, longitude);

        let from: Coordinate = (latitude, longitude);

        let closest_states: Vec<ApiState> = Vec::with_capacity(count);

        // println!("result: {:?}", api_result);
    }
}

fn get_json_data() -> Result<Vec<ApiState>, GetApiError> {
    // TODO specify GPS boundries

    let mut resp_data = match reqwest::get("https://opensky-network.org/api/states/all") {
        Ok(data) => data,
        Err(_) => return Err(GetApiError::new(String::from("request failed"))),
    };

    let json_data: ApiResponse = match resp_data.json() {
        Ok(data) => data,
        Err(reason) => {
            println!("reason: {}", reason);
            return Err(GetApiError::new(String::from("json parse failed")));
        }
    };

    let mut states: Vec<ApiState> = Vec::new();

    // TODO implement custom deserializer?
    for state in json_data.states {
        let mut item = ApiState::default();
        for i in 0..17 {
            let elem = &state[i];
            match i {
                0 => item.icao24 = elem.to_string(),
                2 => {
                    if !elem.is_null() {
                        item.callsign = Some(String::from(elem.as_str().unwrap()));
                    }
                }
                3 => {
                    if !elem.is_null() {
                        item.time_position = Some(elem.as_i64().unwrap());
                    }
                }
                4 => item.last_contact = elem.as_u64().unwrap(),
                5 => item.longitude = elem.as_f64(),
                6 => item.longitude = elem.as_f64(),
                7 => item.baro_altitude = elem.as_f64(),
                8 => item.on_ground = elem.as_bool().unwrap(),
                9 => item.velocity = elem.as_f64(),
                10 => item.true_track = elem.as_f64(),
                11 => item.vertical_rate = elem.as_f64(),
                // 12 => item.sensors = elem.as_array().unwrap(),
                13 => item.geo_altitude = elem.as_f64(),
                14 => {
                    if !elem.is_null() {
                        item.squawk = Some(String::from(elem.as_str().unwrap()));
                    }
                }
                15 => {
                    item.spi = elem.as_bool().unwrap();
                }
                16 => item.position_source = elem.as_u64().unwrap() as u8,
                _ => (),
            }
        }

        states.push(item);
    }

    Ok(states)
}

fn dist(from: Coordinate, to: Coordinate) -> f64 {
    unimplemented!();
}
