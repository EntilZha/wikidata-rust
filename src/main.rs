extern crate clap;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::{BufReader, BufRead};
use std::vec::Vec;
use std::iter;
use std::iter::Iterator;
use clap::{Arg, App};


use serde_json::Value;
use serde_json::map::{Map, Values};

#[derive(Deserialize)]
struct Item {
    id: String,
    #[serde(rename="type")]
    wd_type: String,
    claims: Map<String, Value>,
    sitelinks: Value
}

#[derive(Debug)]
struct Relation {
    subject: String,
    property: String,
    value: String
}


fn extract_claims(subject_id: &String, claim_snaks: Values) -> Vec<Relation> {
    claim_snaks.flat_map(|raw_snak_values| {
        let snak_values = raw_snak_values.as_array().unwrap();
        snak_values.iter().flat_map(|snak| {
            let mainsnak = &snak["mainsnak"];
            let subject = subject_id.clone();
            let datatype = mainsnak["datatype"].as_str().expect("Expected datatype to exist");
            let property = mainsnak["property"]
                .as_str()
                .expect("Expected property to exist")
                .to_string();
            match datatype {
                "monolingualtext" => {
                    let value = mainsnak["datavalue"]["value"]["text"]
                        .as_str()
                        .expect("Expected text value to exist")
                        .to_string();
                    Some(Relation {subject, property, value})
                },
                "wikibase-item" | "wikibase-property" => {
                    match mainsnak["datavalue"]["value"]["id"].as_str() {
                        Some(str_value) => {
                            let value = str_value.to_string();
                            Some(Relation {subject, property, value})
                        },
                        None => None
                    }
                }
                "external-id" | "url" | "globe-coordinate" | "quantity"
                    | "string" | "commonsMedia" | "time" | "math" => None,
                _ => {
                    println!("\n!!!!Skipping: {}", datatype);
                    println!("!!!!Claim: {}", mainsnak);
                    None
                }
            }
        })
    }).collect()
}


fn main() {
    let matches = App::new("wikidata")
                          .version("1.0")
                          .author("Pedro Rodriguez")
                          .about("Parses wikidata.org json dump")
                          .arg(
                              Arg::with_name("INPUT")
                              .help("Location of wikidata json dump")
                              .required(true)
                              .index(1))
                          .get_matches();
    let input_filename = matches.value_of("INPUT").unwrap();
    let mut file = File::open(input_filename).expect("file not found");
    for raw_line in BufReader::new(file).lines() {
        match raw_line {
            Ok(line) => {
                if line.starts_with("[") || line.starts_with("]"){
                    println!("Skipping header");
                } else {
                    let length = line.len();
                    let (valid_json, _) = line.split_at(length - 1);
                    let wd_item: Item = serde_json::from_str(valid_json).expect("Valid json not found");
                    let id = &wd_item.id;
                    println!("{:?}", extract_claims(id, wd_item.claims.values()));

                    println!("{}", id);
                }
            },
            _ => ()
        }
    }
}
