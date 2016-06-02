/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate time;
use std::process::Command;
use std::collections::BTreeMap;
use time::{strptime, Tm, Duration, now};
use std::env;

const DATE_FORMAT: &'static str = "%Y-%m-%d";

struct Bounds {
    upper: Tm,
    lower_relative: Option<i64>
}

impl Bounds {
    fn add_to_command(&self, command: &mut Command) {
        // If there was no lower_relative bound, ignore this
        if self.lower_relative.is_none() {
            return;
        }
        // Add the days to the cut-off point
        let after_tm = self.upper + Duration::days(self.lower_relative.unwrap());
        // Convert it to a string
        let after_tm_str = after_tm.strftime(DATE_FORMAT).unwrap();
        // Convert the cut-off point to a string
        let upper_str = self.upper.strftime(DATE_FORMAT).unwrap();
        // Format the before and after arguments
        let after = format!("--after={}", if after_tm < self.upper { &after_tm_str } else { &upper_str });
        let before = format!("--before={}", if after_tm < self.upper { &upper_str } else { &after_tm_str });
        // Add the arguments
        command.arg(after).arg(before);
    }

    fn filter_dates(&self, dates: Vec<&str>) -> Vec<Tm> {
        dates.iter()
        // Filter out any empty lines
        .filter(|&it| it.len() > 0)
        // Get the first space-delimited part
        .map(|it| it.split(" ").collect::<Vec<_>>()[0])
        // Convert it to an optional Tm
        .map(|it| strptime(it, DATE_FORMAT))
        // Only take successful conversions
        .filter(|&it| it.is_ok())
        // Unwrap the guaranteed-okay Option<Tm>
        .map(|it| it.unwrap())
        // Filter out any out-of-range dates
        .filter(|&it| self.lower_relative.is_none() || self.upper - it <= Duration::days(self.lower_relative.unwrap().abs()))
        // Collect into a Vec<Tm>
        .collect::<Vec<_>>()
    }
}

fn get_bounds() -> Bounds {
    let mut bounds = Bounds { upper: now(), lower_relative: None };
    // Collect the arguments passed to the process
    let args = env::args().collect::<Vec<_>>();
    // If there is more than one argument
    if args.len() > 2 {
        // Set the new cut-off point to the first argument
        bounds.upper = strptime(&args[1], DATE_FORMAT).unwrap_or_else(|e| panic!("invalid date in argument: {}", e));
        // Set the new number of days to add to the cut-off point to the second argument
        let mut days = args[2].parse::<i64>().unwrap_or_else(|e| panic!("invalid number in argument: {}", e));
        // Move the days one towards zero.
        days += if days < 0 { 1 } else { -1 };
        bounds.lower_relative = Some(days);
    }
    bounds
}

fn get_command(bounds: &Bounds) -> Command {
    let mut command = Command::new("git");
    // Add basic arguments
    command
    .arg("--no-pager")
    .arg("log")
    .arg("--pretty=format:%ai");
    // Add the bounds to the command
    bounds.add_to_command(&mut command);
    command
}

fn get_command_output(command: &mut Command) -> String {
    // Get the raw output of the command
    let raw_output = command
    .output()
    .unwrap_or_else(|e| { panic!("failed to execute git: {}", e) })
    .stdout;
    // Convert the raw output to a string
    String::from_utf8_lossy(&raw_output).into_owned()
}

fn run_command(bounds: &Bounds) -> String {
    // Get the git command
    let mut command = get_command(&bounds);
    // Convert the raw output to a string
    get_command_output(&mut command)
}

fn fill_gaps(map: &BTreeMap<Tm, i32>) -> BTreeMap<Tm, i32> {
    // Clone the original map
    let mut new_dates = map.clone();
    // Get windows of two from the map
    for win in map.iter().collect::<Vec<_>>().windows(2) {
        let &one_date = win[0].0;
        let &two_date = win[1].0;
        // Get the difference (in days) between the two dates
        let diff = (two_date.to_timespec() - one_date.to_timespec()).num_days();
        // Only act if the difference is more than one day
        if diff <= 1 {
            continue;
        }
        // Fill gaps between one and two with zero
        for i in 1..diff {
            new_dates.entry(one_date + Duration::days(i)).or_insert(0);
        }
    }
    // Return the cloned, filled map
    new_dates
}

fn format_date_map(map: &BTreeMap<Tm, i32>) -> String {
    map.iter().map(|it| format!("{} ", it.1)).collect::<String>().trim().to_string()
}

fn get_frequencies() -> String {
    // Get the bounds
    let bounds = get_bounds();
    // Run the git command
    let output = run_command(&bounds);
    // Get a list of dates
    let dates_list = bounds.filter_dates(output.split("\n").collect());
    // Create a sorted map
    let mut dates_map: BTreeMap<Tm, i32> = BTreeMap::new();
    // Count the commits for each day
    for date in dates_list {
        *dates_map.entry(date).or_insert(0) += 1;
    }
    // Add bounds if necessary
    if bounds.lower_relative.is_some() && dates_map.len() < bounds.lower_relative.unwrap().abs() as usize {
        dates_map.entry(bounds.upper).or_insert(0);
        dates_map.entry(bounds.upper + Duration::days(bounds.lower_relative.unwrap())).or_insert(0);
    }
    // Fill in the gaps with zeroes
    let new_dates = fill_gaps(&dates_map);
    // Join all the counts by spaces
    format_date_map(&new_dates)
}

fn main() {
    // Print out the commit numbers
    print!("{}", get_frequencies());
}
