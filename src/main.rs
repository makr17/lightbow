use std::thread::sleep;
use std::time::Duration;

extern crate sacn;
use sacn::DmxSource;

const MAX_BRIGHTNESS: u8 = 75;
const PIXEL_SIZE: usize = 3;
const UNIVERSE_SIZE: usize = 510;

struct Zone  { head: u8, body: u8, tail: u8, name: String }

fn build_rainbow (zones: &[Zone]) -> Vec<u8> {
    let live: u16 = zones.iter().map(|x| (x.body as u16)).sum();
    let float_per_trans: f32 = (MAX_BRIGHTNESS as f32)/((live as f32)/6_f32);
    let per_trans = float_per_trans.round() as u8;
    let mut lights: Vec<u8> = vec![];
    let mut red: u8 = MAX_BRIGHTNESS;
    let mut green: u8 = 0;
    let mut blue: u8 = 0;
    // red at max, ramp up green
    for x in 0..live/6 {
        lights.push(red);
        lights.push(green);
        lights.push(blue);
        green += per_trans;
    }
    green = MAX_BRIGHTNESS; // in case of rounding errors...
    // green at max, ramp down red
    for x in 0..live/6 {
        lights.push(red);
        lights.push(green);
        lights.push(blue);
        red -= per_trans;
    }
    red = 0; // rounding errors
    // green at max, ramp up blue
    for x in 0..live/6 {
        lights.push(red);
        lights.push(green);
        lights.push(blue);
        blue += per_trans;
    }
    blue = MAX_BRIGHTNESS; // rounding errors
    // blue at max, ramp down green
    for x in 0..live/6 {
        lights.push(red);
        lights.push(green);
        lights.push(blue);
        green -= per_trans;
    }
    green = 0;  // rounding errors
    // blue at max, ramp up red
    for x in 0..live/6 {
        lights.push(red);
        lights.push(green);
        lights.push(blue);
        red += per_trans;
    }
    red = MAX_BRIGHTNESS; // rounding errors
    // red at max, ramp down blue
    for x in 0..live/6 {
        lights.push(red);
        lights.push(green);
        lights.push(blue);
        blue -= per_trans;
    }

    return lights;
}

fn main() {
    let dmx = DmxSource::new("Controller").unwrap();

    let zones: [Zone; 6] = [
        Zone { head: 3, body: 47, tail: 0, name: "10".to_string() },
        Zone { head: 2, body: 92, tail: 2, name: "11a".to_string() },
        Zone { head: 2, body: 92, tail: 2, name: "11b".to_string() },
        Zone { head: 2, body: 90, tail: 3, name: "12a".to_string() },
        Zone { head: 2, body: 91, tail: 3, name: "12b".to_string() },
        Zone { head: 2, body: 43, tail: 0, name: "13".to_string() }
    ];

    let mut lights  = build_rainbow(&zones);
    // println!("{:?}", lights);
    let refresh = Duration::new(1, 500_000_000);
    loop {
        render(&lights, &zones, &dmx);
        // take last three elements and move to front of vector
        for i in 0..PIXEL_SIZE {
            let pix: u8 = lights.pop().unwrap();
            lights.insert(0, pix);
        }
        sleep(refresh);
    }
    dmx.terminate_stream(1);
}

// output to lighting controller via sACN
fn render( lights: &Vec<u8>, zones: &[Zone], dmx: &DmxSource ) {
    let mut out: Vec<u8> = lights.clone();
    let mut offset: usize = 0;
    for zone in zones {
        // null pixels at head
        let mut idx = offset;
        if idx > out.len() as usize {
            break;
        }
        for n in 0..(zone.head * PIXEL_SIZE as u8) {
            out.insert(idx as usize, 0);
        }
        idx += zone.head as usize * PIXEL_SIZE + zone.body as usize * PIXEL_SIZE;
        if idx > out.len() as usize {
            break;
        }
        for n in 0..(zone.tail * PIXEL_SIZE as u8) {
            out.insert(idx, 0);
        }
        offset += (zone.head as usize + zone.body as usize + zone.tail as usize) * PIXEL_SIZE;
    }
    let mut universes = Vec::new();
    while out.len() > UNIVERSE_SIZE {
        let u = out.split_off(UNIVERSE_SIZE);
        universes.push(u);
    }
    universes.push(out);
    // println!("{:?}", universes);
    let mut universe: u16 = 1;
    while let Some(u) = universes.pop() {
        dmx.send(universe, &u);
        universe += 1;
    }
}
