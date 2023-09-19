use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use xml::reader::{XmlEvent, EventReader};
use csv::WriterBuilder;

struct Arguments {
    library_xml: Option<String>,
}

impl Arguments
{
    fn from_iter<T>(args: T) -> Self
        where T: Iterator<Item = String>
    {
        let mut xml = Option::None;
        let mut pos: usize = 0;
        for arg in args {
            pos+=1;
            if pos == 1 { continue }
            xml = Some(arg);
        }
        Self {
            library_xml: xml
        }
    }
}



fn main() {
    let args = Arguments::from_iter(env::args().into_iter());
    if let Some(ref path) = args.library_xml {
        // println!("Analyzing library at {}", path)
    } else {
        eprintln!("No library specified");
        exit(1);
    }

    go(args.library_xml.unwrap().as_str()).expect("TODO: panic message");
}

const PLIST: &str = "plist";
const DICT: &str = "dict";
const KEY: &str = "key";
const TRACKS: &str = "Tracks";
#[derive(Debug)]
enum Fsm {
    Start,
    LookingForTracksDict,
    InTrackList,

    InTrack,
    ReadingTrackKey,
    ReadingTrackValueForKey(String),
    Done
}

fn go(xml: &str) -> std::io::Result<()> {
    let file = File::open(xml)?;
    let file = BufReader::new(file); // Buffering is important for performance

    let parser = EventReader::new(file);

    let mut fsm = Fsm::Start;
    let mut c = 0;

    let mut value_type = String::default();

    let mut headers: HashSet<String> = HashSet::new();

    let mut building_track: HashMap<String, String> = HashMap::new();
    let mut all_tracks: Vec<HashMap<String, String>> = Vec::new();

    for e in parser {

        let e = match(e) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{err}");
                panic!("exiting")
            }
        };

        match (&mut fsm, e) {
            (Fsm::Start, XmlEvent::Characters(tracks)) if tracks == TRACKS => {
                fsm = Fsm::LookingForTracksDict;
            }
            (Fsm::LookingForTracksDict, XmlEvent::StartElement{ name, .. }) if name.local_name == DICT => {
                fsm = Fsm::InTrackList;
            }

            (Fsm::InTrackList, XmlEvent::StartElement{ name, .. }) if name.local_name == DICT => {
                c +=1;
                fsm = Fsm::InTrack;
            }
            (Fsm::InTrack, XmlEvent::StartElement{ name, .. }) if name.local_name == KEY => {
                fsm = Fsm::ReadingTrackKey;
            }
            (Fsm::ReadingTrackKey, XmlEvent::Characters( data )) => {
                headers.insert(data.clone());
                fsm = Fsm::ReadingTrackValueForKey(data);
            }
            //
            (Fsm::ReadingTrackValueForKey(key_name), XmlEvent::StartElement{ name: vt, .. }) => {
                match vt.local_name.as_str() {
                    "true" => {
                        // print!("true,");
                        building_track.insert(key_name.clone(), String::from("true"));
                        // println!("{key_name} is type boolean with value true");
                        fsm = Fsm::InTrack;
                        value_type = String::default();
                    },
                    "false" => {
                        building_track.insert(key_name.clone(), String::from("false"));
                        // print!("false,");
                        // println!("{key_name} is type boolean with value false");
                        fsm = Fsm::InTrack;
                        value_type = String::default();
                    },
                    vt => {
                        value_type = vt.to_string();
                    }
                }
            }
            (Fsm::ReadingTrackValueForKey(key_name), XmlEvent::Characters( data )) => {
                // println!("{key_name} is type {value_type} with value {data}");
                // print!("{data},");
                building_track.insert(key_name.clone(), data);
                fsm = Fsm::InTrack;
                value_type = String::default();
            }
            (Fsm::InTrack, XmlEvent::EndElement {name, ..}) if name.local_name == DICT => {
                all_tracks.push(building_track.clone());
                building_track = HashMap::new();
                fsm = Fsm::InTrackList
            }
            (Fsm::InTrackList, XmlEvent::EndElement {name , ..}) if name.local_name == DICT => {
                fsm = Fsm::Done
            }
            (fsm_state, XmlEvent::Characters(data)) => {
                // println!("{:?} Characters->> {:?}", fsm_state, data)
            }
            (fsm_state, xml_event) => {
                // println!("{:?} {:?}", fsm_state, xml_event)
            }
        }
    }
    let mut wtr = WriterBuilder::new().from_path("data.csv")?;
    let headers: Vec<String> = headers.into_iter().collect();
    wtr.write_record(&headers).expect("Error writing header");

    let empty = String::from("");


    let total = all_tracks.len();
    let mut cnt = 0;
    for t in all_tracks.iter() {
        cnt+=1;
        let track_record: Vec<String> = headers.iter().map(|h| {
            let val = t.get(h).unwrap_or(&empty).clone();
            val
        }).collect();
        println!("{}/{}", cnt, total);

        wtr.write_record(track_record).expect("Writing track record failed");
    }


    Ok(())
}
