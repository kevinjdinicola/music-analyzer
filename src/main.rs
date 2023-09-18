use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use xml::reader::{XmlEvent, EventReader};

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
        println!("Analyzing library at {}", path)
    } else {
        eprintln!("No library specified");
        exit(1);
    }

    go(args.library_xml.unwrap().as_str()).expect("TODO: panic message");
}

fn go(xml: &str) -> std::io::Result<()> {
    let mut counter = 0;
    let file = File::open(xml)?;
    let file = BufReader::new(file); // Buffering is important for performance

    let parser = EventReader::new(file);
    let mut depth = 0;
    for e in parser {
        counter += 1;
        if counter > 100 {
            break;
        }
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                println!("{:spaces$}+{name}", "", spaces = depth * 2);
                depth += 1;
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
                println!("{:spaces$}-{name}", "", spaces = depth * 2);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            // There's more: https://docs.rs/xml-rs/latest/xml/reader/enum.XmlEvent.html
            _ => {}
        }
    }

    Ok(())
}