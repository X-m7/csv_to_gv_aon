#[macro_use]
extern crate horrorshow;

use std::collections::HashMap;
use std::env;
use std::fs::File;

extern crate csv;

///The beginning part of the graphviz (dot format) output
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
    let file_path = args[1].to_string();
    let args = args.iter().skip(2);
    let mut double_slack = false;
    let mut explicit_stats = false;
    for arg in args {
        match arg.as_str() {
            "--dslack" => double_slack = true,
            "--estats" => explicit_stats = true,
            _ => (),
        }
    }

    let activities = get_activities_from_csv(file_path, explicit_stats);
    if explicit_stats {
        println!("{}", gen_gv(activities, double_slack));
    } else {
        println!("{}", gen_gv(calc_stats(activities), double_slack));
    }
}

/// Expected input file format:
/// - CSV file
/// - First row as headers
/// - Actual data from second row onwards with 4 columns (activity ID, description, duration, predecessors, in that order)
/// - Predecessors as a string of activity IDs separated by comma (in one cell)
/// - Activities are listed in such a way that the predecessors of each activity are located above it in the file
/// - If get_explicit_stats is true then 2 more columns are required, which are early start and late finish, also in that order
fn get_activities_from_csv(
    input_filename: String,
    get_explicit_stats: bool,
) -> (Vec<Activity>, HashMap<String, ActivityStats>) {
    let file = File::open(input_filename);
    if file.is_err() {
        panic!("Failed to read file");
    }
    let mut rdr = csv::Reader::from_reader(file.unwrap());
    let mut activities: Vec<Activity> = Vec::new();
    let mut act_stats: HashMap<String, ActivityStats> = HashMap::new();
    for result in rdr.records() {
        //for each row of the CSV
        let record = result.unwrap();
        let dur = record.get(2).unwrap().to_string().parse::<u32>().unwrap();
        let activity = Activity {
            id: {
                let id = record.get(0).unwrap().to_string();
                act_stats.insert(id.clone(), {
                    if !get_explicit_stats {
                        ActivityStats {
                            early_start: 0,
                            late_start: 0,
                            early_finish: 0,
                            late_finish: 0,
                            slack: 0,
                            next: Vec::new(),
                        }
                    } else {
                        let early_start =
                            record.get(4).unwrap().to_string().parse::<u32>().unwrap();
                        let late_finish =
                            record.get(5).unwrap().to_string().parse::<u32>().unwrap();
                        let late_start = late_finish - dur;
                        let early_finish = early_start + dur;
                        let slack = late_finish - early_finish;
                        ActivityStats {
                            early_start,
                            late_start,
                            early_finish,
                            late_finish,
                            slack,
                            next: Vec::new(),
                        }
                    }
                });
                id
            },
            desc: record.get(1).unwrap().to_string(),
            dur,
            pred: {
                let mut vec: Vec<String> = Vec::new();
                let preds = record.get(3).unwrap().to_string();
                let id = record.get(0).unwrap().to_string();
                if !preds.is_empty() {
                    //if it is we will end up with an empty string as a predecessor, need to ignore that
                    for i in preds.split(',') {
                        //for each listed predecessor
                        let i_str = i.to_string();
                        act_stats.get_mut(&i_str).unwrap().next.push(id.clone()); //store the current Activity being created as the successor of i
                        vec.push(i_str);
                    }
                }
                vec
            },
        };
        activities.push(activity);
    }
    (activities, act_stats)
}

/// Calculates all the values of the ActivityStats
fn calc_stats(
    acts: (Vec<Activity>, HashMap<String, ActivityStats>),
) -> (Vec<Activity>, HashMap<String, ActivityStats>) {
    let (activities, mut act_stats) = acts;
    for i in &activities {
        //forward pass
        let mut max_pred_early_finish = 0;
        for j in &i.pred {
            //need to find the largest early finish value of the predecessors
            let j_stats = &act_stats[&j.clone()];
            if j_stats.early_finish > max_pred_early_finish {
                max_pred_early_finish = j_stats.early_finish;
            }
        }
        let mut stats = act_stats.get_mut(&i.id).unwrap();
        stats.early_start = max_pred_early_finish;
        stats.early_finish = stats.early_start + i.dur;
    }
    for i in &activities {
        //find all activities with no successors
        let mut stats = act_stats.get_mut(&i.id).unwrap();
        if stats.next.is_empty() {
            stats.late_finish = stats.early_finish; //do this as preparation for the backward pass
        }
    }
    for i in activities.iter().rev() {
        //backward pass
        let stats = &act_stats[&i.id];
        let mut min_next_late_start = u32::max_value();
        if !stats.next.is_empty() {
            //if there are successors find the minimum late start of them
            for j in &stats.next {
                let j_stats = &act_stats[&j.clone()];
                if j_stats.late_start < min_next_late_start {
                    min_next_late_start = j_stats.late_start;
                }
            }
        } else {
            //else (no successors) just use its early finish value
            min_next_late_start = stats.early_finish;
        }
        let mut stats = act_stats.get_mut(&i.id).unwrap();
        stats.late_finish = min_next_late_start;
        stats.late_start = stats.late_finish - i.dur;
    }
    for i in &activities {
        //calculate slack for all activities
        let mut stats = act_stats.get_mut(&i.id).unwrap();
        stats.slack = stats.late_start - stats.early_start;
    }
    (activities, act_stats)
}

///Stores data for an Activity that will be calculated
struct ActivityStats {
    early_start: u32,
    late_start: u32,
    early_finish: u32,
    late_finish: u32,
    slack: u32,
    next: Vec<String>, //successors of the activity (kept here to avoid ownership issues with the main Activity struct, also only needed when calculating stats anyway)
}

///Stores data for an activity that is derived directly from the input file
struct Activity {
    id: String,
    desc: String,      //description
    dur: u32,          //duration
    pred: Vec<String>, //predecessors
}

impl Activity {
    ///Returns an HTML-style output for use by graphviz (dot format)
    fn get_output(&self, stats: &ActivityStats, double_slack: bool) -> String {
        if self.dur != 0 {
            //if it is a normal activity
            format!(
                "{}",
                html!(
                    table(border="0", cellborder="1", cellspacing="0") {
                        tr {
                            td: stats.early_start;
                            td: self.id.clone();
                            td: stats.early_finish;
                        }
                        tr {
                            td: stats.slack;
                            @ if double_slack {
                                td: self.desc.clone();
                                td: stats.slack;
                            } else {
                                td(colspan="2"): self.desc.clone();
                            }
                        }
                        tr {
                            td: stats.late_start;
                            td: self.dur;
                            td: stats.late_finish;
                        }
                    }
                )
            )
        } else {
            //else assume that it is not meant to be an actual activity (just the project itself, or something else)
            format!(
                "{}",
                html!(
                    table(border="0", cellborder="1", cellspacing="0") {
                        tr {
                            td: self.id.clone();
                        }
                        tr {
                            td: self.desc.clone();
                        }
                    }
                )
            )
        }
    }
}

///Returns a String for consumption by graphviz (dot format) representing the activities as an Activity-on-Node diagram
fn gen_gv(acts: (Vec<Activity>, HashMap<String, ActivityStats>), double_slack: bool) -> String {
    let (activities, act_stats) = acts;
    let mut output = GV_BEGIN.to_string();
    let mut edges: Vec<(String, String)> = Vec::new();
    for i in activities {
        //create the node definition for each activity
        output.push_str(&format!(
            "{} [label = <{}>];\n",
            i.id,
            i.get_output(&act_stats[&i.id], double_slack)
        ));
        for j in i.pred {
            //store the edges to be created later
            edges.push((j, i.id.clone()));
        }
    }
    output.push('\n'); //newline to separate the edge definitions from the nodes, just to make it easier to read
    for i in edges {
        //create all the edges
        output.push_str(&format!("{} -> {}\n", i.0, i.1))
    }
    output.push_str("}"); //the end of the output
    output
}
