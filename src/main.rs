use std::env;
use std::thread::sleep;
use std::time::Duration;

extern crate getopts;
use getopts::Options;
extern crate houselights;
use houselights::houselights::{RGB,Zone,Dmx,render};

const MAX_BRIGHTNESS: u8 = 255;

struct Params { sleep: Duration }

fn build_params () -> Params {
    let mut params = Params { sleep: Duration::new(0, 200_000_000) };

    // parse command line args and adjust params accordingly
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("s", "sleep", "sleep interval in seconds, default 0.5", "SECONDS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("s") {
        // take float seconds
        // convert to int seconds and nanoseconds to make Duration happy
        let seconds: f32 = matches.opt_str("s").unwrap().parse::<f32>().unwrap();
        let whole_seconds: u64 = seconds as u64;
        let nano_seconds: u32 = ((seconds - whole_seconds as f32) * 1_000_000_000_f32) as u32;
        params.sleep = Duration::new(whole_seconds, nano_seconds);
    }

    return params;
}

fn build_rainbow (zones: &[Zone]) -> Vec<RGB> {
    let mut live: u16 = 0;
    for zone in zones {
        live += zone.body as u16;
    }
    let float_per_trans: f32 = 255_f32/((live as f32)/6_f32);
    let per_trans = float_per_trans.round() as u8;
    let mut lights: Vec<RGB> = vec![];
    let mut red: u8 = MAX_BRIGHTNESS;
    let mut green: u8 = 0;
    let mut blue: u8 = 0;
    // red at max, ramp up green
    for _x in 0..live/6 {
        lights.push(RGB { red: red, green: green, blue: blue });
        green += per_trans;
    }
    green = MAX_BRIGHTNESS; // in case of rounding errors...
    // green at max, ramp down red
    for _x in 0..live/6 {
        lights.push(RGB { red: red, green: green, blue: blue });
        red -= per_trans;
    }
    red = 0; // rounding errors
    // green at max, ramp up blue
    for _x in 0..live/6 {
        lights.push(RGB { red: red, green: green, blue: blue });
        blue += per_trans;
    }
    blue = MAX_BRIGHTNESS; // rounding errors
    // blue at max, ramp down green
    for _x in 0..live/6 {
        lights.push(RGB { red: red, green: green, blue: blue });
        green -= per_trans;
    }
    green = 0;  // rounding errors
    // blue at max, ramp up red
    for _x in 0..live/6 {
        lights.push(RGB { red: red, green: green, blue: blue });
        red += per_trans;
    }
    red = MAX_BRIGHTNESS; // rounding errors
    // red at max, ramp down blue
    for _x in 0..live/6 {
        lights.push(RGB { red: red, green: green, blue: blue });
        blue -= per_trans;
    }

    return lights;
}

fn main() {
    let params = build_params();

    let dmx = Dmx::new();

    let zones: [Zone; 6] = [
        Zone { head: 0, body: 44, tail: 3, name: "10".to_string() },
        Zone { head: 2, body: 91, tail: 3, name: "11a".to_string() },
        Zone { head: 2, body: 92, tail: 2, name: "11b".to_string() },
        Zone { head: 2, body: 90, tail: 3, name: "12a".to_string() },
        Zone { head: 2, body: 91, tail: 3, name: "12b".to_string() },
        Zone { head: 2, body: 43, tail: 0, name: "13".to_string() }
    ];

    let mut lights = build_rainbow(&zones);
    loop {
        render(&lights, &zones, &dmx);
        // take last trailing pixel and move to front of vector
        let pix: RGB = lights.pop().unwrap();
        lights.insert(0, pix);
        sleep(params.sleep);
    }
}
