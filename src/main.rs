#[macro_use]
extern crate horrorshow;

const GV_BEGIN: &str = "
digraph aon {
rankdir=LR;
node [shape = plain];

";

fn main() {
	let act = Activity {id: "1".to_string(), desc: "Desc".to_string(), dur: 2, pred: Vec::new()};
	let act2 = Activity {id: "2".to_string(), desc: "Desc".to_string(), dur: 2, pred: vec!["1".to_string()]};
    println!("{}", gen_gv(vec![act, act2]));
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