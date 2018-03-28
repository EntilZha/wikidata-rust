extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate rayon;

#[macro_use]
extern crate serde_derive;

use std::fmt::format;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::vec::Vec;
use std::iter::Iterator;
use std::path::Path;
use std::io::prelude::*;
use clap::{Arg, App};
use rayon::prelude::*;


use serde_json::Value;
use serde_json::map::{Map, Values};

#[derive(Deserialize)]
struct Item {
    id: String,
    claims: Map<String, Value>,
    #[serde(default)]
    sitelinks: Value
}

#[derive(Debug, Serialize)]
struct Relation {
    subject: String,
    property: String,
    value: String,
    title: Option<String>
}


fn parse_value(datatype: &str, mainsnak: &Value) -> Option<String> {
    match datatype {
        "monolingualtext" => {
            mainsnak["datavalue"]["value"]["text"].as_str().map(|s| s.to_string())
        },
        "wikibase-item" | "wikibase-property" => {
            mainsnak["datavalue"]["value"]["id"].as_str().map(|s| s.to_string())
        }
        "external-id" | "url" | "globe-coordinate" | "quantity"
            | "string" | "commonsMedia" | "time" | "math" => None,
        _ => {
            panic!("Oops, unhandled data type")
        }
    }
}


fn extract_claims(subject_id: &String, maybe_title: Option<&str>, claim_snaks: Values) -> Vec<Relation> {
    claim_snaks.flat_map(|raw_snak_values| {
        let snak_values = raw_snak_values.as_array().unwrap();
        snak_values.iter().flat_map(|snak| {
            let mainsnak = &snak["mainsnak"];
            let subject = subject_id.clone();
            match mainsnak["datatype"].as_str() {
                Some(datatype) => {
                    let property = mainsnak["property"]
                        .as_str()
                        .expect("Expected property to exist")
                        .to_string();
                    let maybe_value = parse_value(datatype, mainsnak);
                    let title = maybe_title.map(|s| s.to_string());
                    match maybe_value {
                        Some(value) => Some(Relation {subject, property, value, title}),
                        None => None
                    }
                },
                None => None
            }
        })
    }).collect()
}

fn line_to_relations(line: &str) -> Vec<Relation> {
    if line.starts_with("[") || line.starts_with("]") {
        Vec::new()
    } else {
        let valid_json = match line.chars().last() {
            Some(',') => {
                let (valid_json, _) = line.split_at(line.len() - 1);
                valid_json
            },
            _ => line
        };
        let wd_item: Item = serde_json::from_str(valid_json)
            .expect(format!("Valid json not found: {}", valid_json).as_str());
        let id = &wd_item.id;
        let title = wd_item.sitelinks["enwiki"]["title"].as_str();
        extract_claims(id, title, wd_item.claims.values())
    }
}

fn filename_to_relations(filename: &str) -> Vec<Relation> {
    let file = File::open(filename).expect("file not found");
    BufReader::new(file).lines().flat_map(|l| {
        match l {
            Ok(l) => line_to_relations(&l),
            Err(_) => Vec::new()
        }
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
                              .index(1)
                              .multiple(true))
                          .get_matches();
    let input_files: Vec<&str> = matches.values_of("INPUT").unwrap().collect();
    input_files
        .par_iter()
        .flat_map(|f| filename_to_relations(f))
        .for_each(|r| {
            println!("{}", serde_json::to_string(&r).unwrap());
        })
}
