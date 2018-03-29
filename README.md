# wikidata-rust

## What does this do?
This code parses wikidata.org JSON dumps. The input should be a file from here: https://dumps.wikimedia.org/wikidatawiki/entities/.
The code will parse all the Subject-Relation-Objects and emits them to a the stdout as json objects, one per line.

## Usage

Below is an example of the first few lines of output:

```bash
$ cargo run --release -- wikidata-dump.json
{"subject":"Q12234604","property":"P105","value":"Q7432","title":null}
{"subject":"Q12234604","property":"P171","value":"Q146434","title":null}
{"subject":"Q12234604","property":"P31","value":"Q16521","title":null}
{"subject":"Q12234608","property":"P105","value":"Q7432","title":null}

```
The command above accepts a list of json files and it will attempt to read each one in parallel. This is helpful since the
wikidata dump is about 100GB it may be faster to split the large file into batches with a command like `split -l 100000 wikidata-20170306-all.json wikidata-part-`

The command to run would change to something like this then:
`cargo run --release -- $(find /scratch0/wikidata-parts/)`

Lastly, if you want the output saved to a file the easiest way is to redirect the stdout to a file like `cargo run --release -- $(find /scratch0/wikidata-parts/ -type f) > relations.jsonl`

## Installation

You will need to install Rust and Cargo which is easily done using rustup: https://www.rustup.rs/. Following this the commands above should just work
