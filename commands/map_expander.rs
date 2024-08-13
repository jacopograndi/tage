#!/usr/bin/env -S cargo +nightly -Zscript

use std::*;

/// expander.rs [path] [option]
///
/// Reads the map in the file at [path] and expands/compacts it based on [option]
///     no option specified (default): expands the map
///     option = compact: compacts the map
///
/// An expanded map is a map where each tile is 4 characters
/// A compacted map is a map where each tile is 1 character
/// The expansion is lossless, compaction is lossy
///
/// Example of the same map compacted and expanded:
/// - compact map:
///     .#==
///     =~m/
///
/// - expanded map
///     ... ### =-- =--
///     =-- ~~~ m() /\\
///
/// Used to write maps quicker, write the compacted form, expand and fill the details
fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    if args.len() > 2 && args[2] == "compact" {
        let expanded = fs::read_to_string(&path)?;
        let compact = expanded
            .split("\n")
            .map(|l| {
                l.split(" ")
                    .filter(|tile| !tile.is_empty())
                    .map(|tile| tile.chars().next().unwrap().to_string())
                    .fold(String::new(), |acc, s| acc + &s)
            })
            .fold(String::new(), |acc, s| acc + &s + "\n");
        fs::write(path.clone() + ".comp", compact)?;
        Ok(())
    } else {
        let compact = fs::read_to_string(&path)?;
        let expanded = compact
            .split("\n")
            .enumerate()
            .map(|(y, l)| {
                l.chars()
                    .enumerate()
                    .map(|(x, c)| match char_to_tile(c) {
                        Some(tile) => (x, tile),
                        None => panic!("Unknown char {:?} at ({}, {})", c, x, y),
                    })
                    .fold(String::new(), |acc, (x, s)| {
                        acc + s + if x < l.len() - 1 { " " } else { "" }
                    })
            })
            .fold(String::new(), |acc, s| acc + &s + "\n");
        fs::write(path.clone() + ".expa", expanded)?;
        Ok(())
    }
}

fn char_to_tile(c: char) -> Option<&'static str> {
    match c {
        '-' => Some(r"---"),
        '+' => Some(r"+--"),
        'm' => Some(r"$))"),
        'M' => Some(r"$\\"),
        '=' => Some(r"=--"),
        '.' => Some(r"..."),
        '~' => Some(r"~~~"),
        '/' => Some(r"/\\"),
        '#' => Some(r"###"),
        '&' => Some(r"&&&"),
        '(' => Some(r"())"),
        '<' => Some(r"<>>"),
        ',' => Some(r",,,"),
        _ => None,
    }
}
