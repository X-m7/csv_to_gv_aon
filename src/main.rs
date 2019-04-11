#[macro_use]
extern crate horrorshow;

use std::env;
use std::fs::File;

extern crate file_scanner;
use file_scanner::Scanner;

const GV_BEGIN: &str = "
digraph aon {
rankdir=LR;
node [shape = plain];

";

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
        panic!("Input file name required as argument");
    }

	let activities = get_activities_from_csv(args[1].to_string());
    println!("{}", gen_gv(activities));
}

fn get_activities_from_csv(input_filename: String) -> Vec<Activity> {
	let file = File::open(input_filename);
	if file.is_err() {
		panic!("Failed to read file");
	}
	let mut file = Scanner::new(file.unwrap());
	file.next_line(); //skip the first line, assumed to be column titles
	let mut activities: Vec<Activity> = Vec::new();
	let mut activity = Activity {id: "".to_string(), desc: "".to_string(), dur: 0, pred: Vec::new()};
	while let Some(input) = file.next_line() {
		let mut counter = 0;
		for i in input.split(',') {
			match counter {
				0 => {
					activity.id = i.to_string();
					counter += 1;
				},
				1 => {
					activity.desc = i.to_string();
					counter += 1;
				},
				2 => {
					activity.dur = i.to_string().parse::<u32>().unwrap();
					counter += 1;
				},
				3 => {
					if !i.is_empty() {
						for j in i.to_string().split(';') {
							activity.pred.push(j.to_string());
						}
					}
					activities.push(activity);
					activity = Activity {id: "".to_string(), desc: "".to_string(), dur: 0, pred: Vec::new()};
					counter = 0;
				},
				_ => ()
			}
		}
	}
	activities
}

struct Activity {
	id: String,
	desc: String, //description
	dur: u32, //duration
	pred: Vec<String>, //predecessors
}

impl Activity {
	fn get_output(&self) -> String {
		format!("{}", html!(
			table(border="0", cellborder="1", cellspacing="0") {
				tr {
					td: "ES";
					td: self.id.clone();
					td: "EF";
				}
				tr {
					td: "SL";
					td(colspan="2"): self.desc.clone();
				}
				tr {
					td: "LS";
					td: self.dur;
					td: "LF";
				}
			}
		))
	}
}

fn gen_gv(activities: Vec<Activity>) -> String {
	let mut output = GV_BEGIN.to_string();
	let mut edges: Vec<(String, String)> = Vec::new();
	for i in activities {
		output.push_str(&format!("{} [label = <{}>];\n", i.id, i.get_output()));
		for j in i.pred {
			edges.push((j, i.id.clone()));
		}
	}
	output.push('\n');
	for i in edges {
		output.push_str(&format!("{} -> {}\n", i.0, i.1))
	}
	output.push_str("}");
	output
}