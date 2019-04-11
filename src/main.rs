#[macro_use]
extern crate horrorshow;

use std::env;
use std::fs::File;
use std::collections::HashMap;

extern crate csv;

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
    println!("{}", gen_gv(activities.0));
}

fn get_activities_from_csv(input_filename: String) -> (Vec<Activity>, HashMap<String, ActivityStats>) {
	let file = File::open(input_filename);
	if file.is_err() {
		panic!("Failed to read file");
	}
	let mut rdr = csv::Reader::from_reader(file.unwrap());
	let mut activities: Vec<Activity> = Vec::new();
	let mut act_stats: HashMap<String, ActivityStats> = HashMap::new();
	for result in rdr.records() {
		let record = result.unwrap();
		let activity = Activity {id: {
			let id = record.get(0).unwrap().to_string();
			act_stats.insert(id.clone(), ActivityStats {early_start: 0, late_start: 0, early_finish: 0, late_finish: 0, slack: 0, next: Vec::new()});
			id
		}, desc: record.get(1).unwrap().to_string(), dur: record.get(2).unwrap().to_string().parse::<u32>().unwrap(), pred: {
			let mut vec: Vec<String> = Vec::new();
			let preds = record.get(3).unwrap().to_string();
			let id = record.get(0).unwrap().to_string();
			if !preds.is_empty() {
				for i in preds.split(';') {
					let i_str = i.to_string();
					act_stats.get_mut(&i_str).unwrap().next.push(id.clone());
					vec.push(i_str);
				}
			}
			vec
		}};
		activities.push(activity);
	}
	(activities, act_stats)
}

struct ActivityStats {
	early_start: u32,
	late_start: u32,
	early_finish: u32,
	late_finish: u32,
	slack: u32,
	next: Vec<String> //successors of the activity (kept here to avoid ownership issues with the main Activity struct, also only needed when calculating stats anyway)
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