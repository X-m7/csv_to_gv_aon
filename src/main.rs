#[macro_use]
extern crate horrorshow;

fn main() {
	let act = Activity {id: "1".to_string(), desc: "Desc".to_string(), dur: 2, pred: Vec::new()};
    println!("{}", act.get_output());
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
