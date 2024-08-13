#!/usr/bin/env -S cargo +nightly -Zscript

use std::collections::HashMap;
use std::*;

/// map_symmetry.rs [path] [symmetry]
///
/// Reads the map in the file at [path], doubles the area and applies the symmetric transformation
///     no option specified (default): C1
///     option = C1, option C2, option C3, ...: Applies a rotation like symmetry
///     option = D1, option D2, option D3, ...: Applies a mirror like symmetry
///     the numbers specify the amount of axis.
fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        let path = &args[1];
        let symmetry: Symmetry = args[2].parse()?;
        let map = fs::read_to_string(&path).map_err(|_| format!("Failed to read"))?;
        let grid: MiniGrid = map.parse()?;
        let drawn = draw(grid, symmetry);
        let out = drawn.to_string();
        fs::write(path.clone() + ".out", out.clone()).map_err(|_| format!("Failed to write"))?;
        println!("{}", out);
    }
    Ok(())
}

fn draw(grid: MiniGrid, symmetry: Symmetry) -> MiniGrid {
    if matches!(symmetry, Symmetry::C(1)) {
        return grid;
    }
    let mut plot: HashMap<(i32, i32), String> = HashMap::new();
    let (mut minx, mut miny) = (i32::MAX, i32::MAX);
    let (mut maxx, mut maxy) = (i32::MIN, i32::MIN);
    for y in 0..grid.size.1 as i32 {
        for x in 0..grid.size.0 as i32 {
            for sample in symmetry.get_symmetric((x, y)) {
                minx = minx.min(sample.0);
                maxx = maxx.max(sample.0);
                miny = miny.min(sample.1);
                maxy = maxy.max(sample.1);
                plot.insert(sample, grid.at((x, y)).clone());
            }
        }
    }
    let (w, h) = (maxx - minx + 1, maxy - miny + 1);
    let size = (w as u32, h as u32);
    let mut out = MiniGrid::new(size);
    for y in miny..=maxy {
        for x in minx..=maxx {
            *out.at_mut((x - minx, y - miny)) = plot.remove(&(x, y)).unwrap_or("...".to_string());
        }
    }
    out
}

use std::f32::consts::TAU;
use std::str::FromStr;

/// https://en.wikipedia.org/wiki/Symmetry_group#Two_dimensions
/// https://en.wikipedia.org/wiki/Point_groups_in_two_dimensions#More_general_groups
///
/// # Examples
/// C1 is asymmetric
/// C2 is point like symmetry: "Z"
/// C3 like the flag of the ile of man
/// C4 like a swastika :o
///
/// D1 is reflection on a single axis
/// D2 is reflection on two orthogonal axis
/// D3 is reflection on the axis formed by a equilateral triangle
/// D4 square
///
/// run the view test to get an understanding
#[derive(Debug, Clone)]
pub enum Symmetry {
    C(u8),
    D(u8),
}

impl Symmetry {
    pub fn get_symmetric(&self, (x, y): (i32, i32)) -> Vec<(i32, i32)> {
        match *self {
            Symmetry::C(1) => vec![(x, y)],
            Symmetry::C(2) => vec![(x, y), (-x, -y)],
            Symmetry::C(n) => {
                let (s, t) = (x as f32, y as f32);
                let theta = TAU / n as f32;
                (0..n)
                    .map(|i| {
                        let cos_theta = f32::cos(theta * i as f32);
                        let sin_theta = f32::sin(theta * i as f32);
                        (
                            (s * cos_theta - t * sin_theta).round() as i32,
                            (s * sin_theta + t * cos_theta).round() as i32,
                        )
                    })
                    .collect()
            }
            Symmetry::D(1) => vec![(x, y), (-x, y)],
            Symmetry::D(2) => vec![(x, y), (-x, y), (x, -y), (-x, -y)],
            Symmetry::D(n) => {
                let (s, t) = (x as f32, y as f32);
                let theta = TAU / n as f32;
                let mut mirrors: Vec<(i32, i32)> = (0..n)
                    .map(|i| {
                        let cos_theta = f32::cos(theta * i as f32);
                        let sin_theta = f32::sin(theta * i as f32);
                        (
                            (s * cos_theta + t * sin_theta).round() as i32,
                            (s * sin_theta - t * cos_theta).round() as i32,
                        )
                    })
                    .collect();
                let rotations = Symmetry::C(n).get_symmetric((x, y));
                mirrors.extend(rotations);
                mirrors
            }
        }
    }
}

impl FromStr for Symmetry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s.chars().next() {
            Some('C') => {
                let num = s
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .parse()
                    .map_err(|_| format!("Failed to read axis num"))?;
                Ok(Symmetry::C(num))
            }
            Some('D') => {
                let num = s
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .parse()
                    .map_err(|_| format!("Failed to read axis num"))?;
                Ok(Symmetry::D(num))
            }
            _ => Err(format!("Unsupported symmetry")),
        }
    }
}

#[derive(Clone, Debug)]
struct MiniGrid {
    size: (u32, u32),
    tiles: Vec<String>,
}

impl MiniGrid {
    const ROW_DELIMITER: &'static str = "\n";
    const COLUMN_DELIMITER: &'static str = " ";

    fn new(size: (u32, u32)) -> Self {
        Self {
            size,
            tiles: (0..(size.0 * size.1)).map(|_| String::default()).collect(),
        }
    }

    fn to_string(&self) -> String {
        let mut s = String::new();
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                s += self.at((x as i32, y as i32));
                if x < self.size.0 - 1 {
                    s += Self::COLUMN_DELIMITER;
                }
            }
            if y < self.size.1 - 1 {
                s += Self::ROW_DELIMITER;
            }
        }
        s
    }

    fn at(&self, (x, y): (i32, i32)) -> &String {
        &self.tiles[(x + y * self.size.0 as i32) as usize]
    }

    fn at_mut(&mut self, (x, y): (i32, i32)) -> &mut String {
        &mut self.tiles[(x + y * self.size.0 as i32) as usize]
    }
}

impl FromStr for MiniGrid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let rows: Vec<String> = s
            .split(Self::ROW_DELIMITER)
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect();
        assert!(!rows.is_empty(), "The map can't be empty, silly!");
        assert!(
            rows.iter().all(|r| r.len() == rows[0].len()),
            "Yo, all rows must be the same length, dude wtf."
        );
        let grid: Vec<Vec<String>> = rows
            .iter()
            .map(|r| {
                r.split(Self::COLUMN_DELIMITER)
                    .filter(|t| !t.is_empty())
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
            })
            .collect();
        Ok(Self {
            size: (grid[0].len() as u32, grid.len() as u32),
            tiles: grid.into_iter().flatten().collect(),
        })
    }
}
