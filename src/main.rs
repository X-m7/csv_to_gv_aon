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
    println!("{}", gen_gv(calc_stats(activities)));
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

fn calc_stats(acts: (Vec<Activity>, HashMap<String, ActivityStats>)) -> (Vec<Activity>, HashMap<String, ActivityStats>) {
	let (activities, mut act_stats) = acts;
	for i in &activities { //forward path
		let mut max_pred_early_finish = 0;
		for j in &i.pred {
			let j_stats = &act_stats[&j.clone()];
			if j_stats.early_finish > max_pred_early_finish {
				max_pred_early_finish = j_stats.early_finish;
			}
		}
		let mut stats = act_stats.get_mut(&i.id).unwrap();
		stats.early_start = max_pred_early_finish;
		stats.early_finish = stats.early_start + i.dur;
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
	fn get_output(&self, stats: &ActivityStats) -> String {
		format!("{}", html!(
			table(border="0", cellborder="1", cellspacing="0") {
				tr {
					td: stats.early_start;
					td: self.id.clone();
					td: stats.early_finish;
				}
				tr {
					td: stats.slack;
					td(colspan="2"): self.desc.clone();
				}
				tr {
					td: stats.late_start;
					td: self.dur;
					td: stats.late_finish;
				}
			}
		))
	}
}

fn gen_gv(acts: (Vec<Activity>, HashMap<String, ActivityStats>)) -> String {
	let (activities, act_stats) = acts;
	let mut output = GV_BEGIN.to_string();
	let mut edges: Vec<(String, String)> = Vec::new();
	for i in activities {
		output.push_str(&format!("{} [label = <{}>];\n", i.id, i.get_output(&act_stats[&i.id])));
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