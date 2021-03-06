use std::env;
use std::thread::sleep;

extern crate getopts;
use getopts::Options;
extern crate houselights;
use houselights::houselights::{RGB,Zone,Dmx,render};
extern crate time;
use time::Duration;

const MAX_BRIGHTNESS: i16 = 150;

struct Params {
    runfor: i64,
    sleep: std::time::Duration
}

fn build_params () -> Params {
    let mut params = Params {
        runfor: 5,
        sleep: Duration::nanoseconds(200_000_000).to_std().unwrap()
    };

    // parse command line args and adjust params accordingly
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("r", "runfor", "number of minutes to run, default 5", "MINUTES");
    opts.optopt("s", "sleep", "sleep interval in seconds, default 0.5", "SECONDS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("r") {
        params.runfor = matches.opt_str("r").unwrap().parse::<i64>().unwrap();
    }
    if matches.opt_present("s") {
        // take float seconds
        // convert to int seconds and nanoseconds to make Duration happy
        let seconds: f32 = matches.opt_str("s").unwrap().parse::<f32>().unwrap();
        let whole_seconds: i64 = seconds as i64;
        let nano_seconds: i64 = ((seconds - whole_seconds as f32) * 1_000_000_000_f32) as i64;
        params.sleep = (Duration::seconds(whole_seconds) + Duration::nanoseconds(nano_seconds)).to_std().unwrap();
    }

    return params;
}

fn build_rainbow (zones: &[Zone]) -> Vec<RGB> {
    let mut live: u16 = 0;
    for zone in zones {
        live += zone.body as u16;
    }
    let float_per_trans: f32 = 255_f32/((live as f32)/6_f32);
    let per_trans = float_per_trans.round() as i16;
    let mut lights: Vec<RGB> = vec![];
    let mut red: i16 = MAX_BRIGHTNESS;
    let mut green: i16 = 0;
    let mut blue: i16 = 0;
    // red at max, ramp up green
    for _x in 0..live/6 {
        lights.push(RGB { red: red as u8, green: green as u8, blue: blue as u8 });
        green += per_trans;
        if green > MAX_BRIGHTNESS {
            green = MAX_BRIGHTNESS;
        }
    }
    green = MAX_BRIGHTNESS; // in case of rounding errors...
    // green at max, ramp down red
    for _x in 0..live/6 {
        lights.push(RGB { red: red as u8, green: green as u8, blue: blue as u8 });
        red -= per_trans;
        if red < 0 {
            red = 0;
        }
    }
    red = 0; // rounding errors
    // green at max, ramp up blue
    for _x in 0..live/6 {
        lights.push(RGB { red: red as u8, green: green as u8, blue: blue as u8 });
        blue += per_trans;
        if blue > MAX_BRIGHTNESS {
            blue = MAX_BRIGHTNESS;
        }
    }
    blue = MAX_BRIGHTNESS; // rounding errors
    // blue at max, ramp down green
    for _x in 0..live/6 {
        lights.push(RGB { red: red as u8, green: green as u8, blue: blue as u8 });
        green -= per_trans;
        if green < 0 {
            green = 0;
        }
    }
    green = 0;  // rounding errors
    // blue at max, ramp up red
    for _x in 0..live/6 {
        lights.push(RGB { red: red as u8, green: green as u8, blue: blue as u8 });
        red += per_trans;
        if red > MAX_BRIGHTNESS {
            red = MAX_BRIGHTNESS;
        }
    }
    red = MAX_BRIGHTNESS; // rounding errors
    // red at max, ramp down blue
    for _x in 0..live/6 {
        lights.push(RGB { red: red as u8, green: green as u8, blue: blue as u8 });
        blue -= per_trans;
        if blue < 0 {
            blue = 0;
        }
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
    let finish = time::get_time() + Duration::minutes(params.runfor);
    loop {
        render(&lights, &zones, &dmx);
        if time::get_time() > finish {
            break;
        }
        // take last trailing pixel and move to front of vector
        let pix: RGB = lights.pop().unwrap();
        lights.insert(0, pix);
        sleep(params.sleep);
    }
}
