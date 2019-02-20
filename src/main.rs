extern crate clap;
extern crate reqwest;
extern crate serde;
use clap::{App, Arg, SubCommand};
use serde::Deserialize;
use serde_json::Value;
use std::cmp::Ordering;
use std::error;
use std::fmt;
use std::process;

// TODO break into module(s)

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

#[derive(Debug, Clone)]
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

    fn dist_from(&self, from: Coordinate) -> f64 {
        if self.latitude == None || self.longitude == None {
            return std::f64::MAX;
        }

        let here = Coordinate::new(self.latitude.unwrap(), self.longitude.unwrap());

        here.geo_dist(Coordinate::new(from.latitude, from.longitude))
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

#[derive(Debug, Copy, Clone)]
struct Coordinate {
    latitude: f64,
    longitude: f64,
}

impl Coordinate {
    fn new(latitude: f64, longitude: f64) -> Coordinate {
        Coordinate {
            latitude,
            longitude,
        }
    }

    fn geo_dist(self, to: Coordinate) -> f64 {
        let earth_radius = f64::from(6371);

        let self_lat_rad = Coordinate::deg_to_rad(self.latitude);
        let self_long_rad = Coordinate::deg_to_rad(self.longitude);

        let to_lat_rad = Coordinate::deg_to_rad(to.latitude);
        let to_long_rad = Coordinate::deg_to_rad(to.longitude);

        let dlong = (self_long_rad - to_long_rad).abs();

        let delta = ((self_lat_rad.sin() * to_lat_rad.sin())
            + (self_lat_rad.cos() * to_lat_rad.cos() * dlong.cos()))
        .acos();

        earth_radius * delta
    }

    fn deg_to_rad(v: f64) -> f64 {
        v * std::f64::consts::PI / 180.0
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
                    Arg::with_name("latitude")
                        .allow_hyphen_values(true)
                        .required(true)
                        .takes_value(true)
                        .help("latitude"),
                )
                .arg(
                    Arg::with_name("longitude")
                        .allow_hyphen_values(true)
                        .required(true)
                        .help("longitude"),
                ),
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

        println!("[DEBUG]: getting JSON data from API");

        let api_result: Vec<ApiState> = match get_json_data() {
            Ok(resp) => resp,
            Err(reason) => {
                eprintln!("get json data failed: {}", reason);
                process::exit(1);
            }
        };

        println!("[DEBUG]: retreived results: {}", api_result.len());

        let origin = Coordinate::new(latitude, longitude);

        // TODO make safe, implmenet ordering for Coordinate?
        let filtered_states: Vec<ApiState> = api_result
            .clone()
            .into_iter()
            .filter(|state| state.latitude != None && state.longitude != None)
            .collect();

        // TODO fix
        let mut sorted_states = filtered_states.clone();
        sorted_states.sort_by(|a, b| {
            let adist = origin.geo_dist(Coordinate::new(a.latitude.unwrap(), a.longitude.unwrap()));
            let bdist = origin.geo_dist(Coordinate::new(b.latitude.unwrap(), b.longitude.unwrap()));

            if adist.is_nan() {
                Ordering::Less
            } else if bdist.is_nan() {
                Ordering::Greater
            } else if adist < bdist {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        let distances: Vec<(ApiState, f64)> = sorted_states
            .clone()
            .into_iter()
            .map(|state| (state.clone(), state.dist_from(origin)))
            .filter(|state_tup| !state_tup.1.is_nan())
            .collect();

        let nearests = &distances[0..count];

        println!(
            "{0: <13} | {1: <13} | {2: <13} | {3: <13} | {4: <13} | {5: <13} | {6: <13}",
            "distance", "callsign", "latitude", "longitude", "altitude", "origin", "ICAO24"
        );

        for state_tup in nearests {
            let state = state_tup.0.clone();
            let dist = state_tup.1;
            // print!("{0: <10.8}", dist);

            println!(
                "{0: <13.10} | {1: <13} | {2: <13} | {3: <13} | {4: <13} | {5: <13} | {6: <13}",
                dist,
                match state.callsign {
                    Some(callsign) => callsign,
                    None => String::from("null"),
                },
                state.latitude.unwrap(),
                state.longitude.unwrap(),
                match state.geo_altitude {
                    Some(alt) => alt.to_string(),
                    None => String::from("null"),
                },
                state.origin_country,
                state.icao24
            );
        }
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
        // thank you clippy for this next line
        // absurd that I have to do this though!
        for (i, elem) in state.iter().enumerate().take(17) {
            match i {
                0 => item.icao24 = elem.to_string(),
                1 => {
                    if !elem.is_null() {
                        item.callsign = Some(String::from(elem.as_str().unwrap()));
                    }
                }
                2 => item.origin_country = String::from(elem.as_str().unwrap()),
                3 => {
                    if !elem.is_null() {
                        item.time_position = Some(elem.as_i64().unwrap());
                    }
                }
                4 => item.last_contact = elem.as_u64().unwrap(),
                5 => item.longitude = elem.as_f64(),
                6 => item.latitude = elem.as_f64(),
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
