use std::fs::File;
use std::io;
use std::io::{BufRead};
use std::path::Path;

fn main() {
    // let lock = io::stdin().lock();
    // let lines: Vec<_> = lock.lines().collect::<Result<_, _>>().expect("Error reading lines");
    let lines: Vec<String> = read_lines("input").expect("Error reading file").collect::<Result<_, _>>().expect("Error reading lines");
    let x: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            let chars: Vec<char> = line.chars()
                .map(|c| c.clone())
                .collect::<Vec<char>>();
            let numbers: Vec<&char> = chars
                .iter()
                .filter(|c| c.is_numeric())
                .collect();

            dbg!(line, numbers.clone());

            match numbers.as_slice() {
                &[a] => {
                    let res = Some([a, a]);
                    res
                },
                &[a, .., b] => {
                    let res = Some([a, b]);
                    res
                },
                _ => None,
            }.map(| pair| String::from_iter(pair.map(|c| c.to_string())))
        })
        .collect();

    let sum: u64 = x.iter().map(|l| l.parse::<u64>().expect("Could not parse int")).sum();
    println!("sum: {}", sum);
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
