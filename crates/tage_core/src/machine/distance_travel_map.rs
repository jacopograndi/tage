use std::collections::VecDeque;

use crate::player::PlayerId;

use super::*;

/// Distance fields, tiles store the position to the nearest <field name>
/// O(2n) -> Needs to scan the board and propagation should take at most another scan
#[derive(Clone, Debug)]
pub struct DistanceTravelMap {
    pub hostile_unit: Grid<i32>,
    pub hostile_building: Grid<i32>,
    pub friendly_unit: Grid<i32>,
    pub friendly_building: Grid<i32>,
    pub friendly_church: Grid<i32>,
    pub relic: Grid<i32>,
    pub resource: Grid<i32>,
    pub unclaimed_resource: Grid<i32>,
    pub good_towncenter_spot: Grid<i32>,
}

impl DistanceTravelMap {
    pub fn from_board(bp: &Blueprints, board: &Board, player_id: &PlayerId) -> Self {
        let grid = Grid::fill(board.grid.size, MAX_DISTANCE_NEG);
        let mut map = DistanceTravelMap {
            hostile_unit: grid.clone(),
            hostile_building: grid.clone(),
            friendly_unit: grid.clone(),
            friendly_building: grid.clone(),
            friendly_church: grid.clone(),
            relic: grid.clone(),
            resource: grid.clone(),
            unclaimed_resource: grid.clone(),
            good_towncenter_spot: grid.clone(),
        };

        let player = board.get_player(player_id);

        let mut move_cost = grid.clone();
        for xy in iter_area(board.grid.size) {
            let tile = board.grid.get_at(&xy);
            move_cost.set_at(&xy, tile.get_movement_cost(bp))
        }

        let town_center_bp = bp
            .get_unit_from_name("Town Center")
            .map(|id| bp.get_unit(&id));
        let church_bp = bp.get_unit_from_name("Church").map(|id| bp.get_unit(&id));

        let mut hostile_unit_sources = vec![];
        let mut hostile_building_sources = vec![];
        let mut friendly_unit_sources = vec![];
        let mut friendly_building_sources = vec![];
        let mut friendly_church_sources = vec![];
        let mut unclaimed_resources_sources = vec![];
        let mut resources_sources = vec![];
        let mut relic_sources = vec![];
        let mut tc_spot = vec![];

        for xy in iter_area(board.grid.size) {
            let tile = board.grid.get_at(&xy);
            if let Some(unit) = &tile.unit {
                if player.is_hostile(board.get_player(&unit.owner)) {
                    map.hostile_unit.set_at(&xy, 0);
                    hostile_unit_sources.push(xy);
                } else {
                    map.friendly_unit.set_at(&xy, 0);
                    friendly_unit_sources.push(xy);
                }
            }
            if let Some(building) = &tile.building {
                if player.is_hostile(board.get_player(&building.owner)) {
                    map.hostile_building.set_at(&xy, 0);
                    hostile_building_sources.push(xy);
                } else {
                    map.friendly_building.set_at(&xy, 0);
                    friendly_building_sources.push(xy);
                    if let Some(church) = church_bp {
                        if building.blueprint_id == church.header.id {
                            map.friendly_church.set_at(&xy, 0);
                            friendly_church_sources.push(xy);
                        }
                    }
                }
            }
            if let Some(_) = &tile.terrain.resource {
                map.resource.set_at(&xy, 0);
                resources_sources.push(xy);
                if tile.building.is_none() {
                    map.unclaimed_resource.set_at(&xy, 0);
                    unclaimed_resources_sources.push(xy);
                }
            }
            if let Some(Collectable::Relic) = &tile.terrain.collectable {
                map.relic.set_at(&xy, 0);
                relic_sources.push(xy);
            }
            if let Some(tc) = town_center_bp {
                let allowed_terrain =
                    tc.build_constraints
                        .iter()
                        .any(|constraint| match constraint {
                            BuildConstraint::OnTerrain(terrain_id) => {
                                &tile.terrain.blueprint_id == terrain_id.terrain()
                            }
                            _ => false,
                        });
                if allowed_terrain || tile.terrain.resource.is_some() {
                    map.good_towncenter_spot.set_at(&xy, 0);
                    tc_spot.push(xy);
                }
            }
        }

        propagate_distance_sources(&mut map.hostile_unit, &move_cost, hostile_unit_sources);
        propagate_distance_sources(
            &mut map.hostile_building,
            &move_cost,
            hostile_building_sources,
        );
        propagate_distance_sources(&mut map.friendly_unit, &move_cost, friendly_unit_sources);
        propagate_distance_sources(
            &mut map.friendly_building,
            &move_cost,
            friendly_building_sources,
        );
        propagate_distance_sources(
            &mut map.friendly_church,
            &move_cost,
            friendly_church_sources,
        );
        propagate_distance_sources(&mut map.resource, &move_cost, resources_sources);
        propagate_distance_sources(
            &mut map.unclaimed_resource,
            &move_cost,
            unclaimed_resources_sources,
        );
        propagate_distance_sources(&mut map.relic, &move_cost, relic_sources);
        propagate_distance_sources(&mut map.good_towncenter_spot, &move_cost, tc_spot);

        map
    }
}

const MAX_DISTANCE_NEG: i32 = -10000;

fn propagate_distance_sources(grid: &mut Grid<i32>, cost: &Grid<i32>, sources: Vec<IVec2>) {
    const MAX_SOURCES_PROPAGATION: usize = 1000000;

    if sources.len() == 0 {
        *grid = Grid::fill(grid.size, 0);
    }

    let mut frontier: VecDeque<IVec2> = sources.into();
    for _iter in 0..MAX_SOURCES_PROPAGATION {
        if let Some(pos) = frontier.pop_front() {
            let distance_tile = *grid.get_at(&pos);
            let mut propagations = vec![];
            for (dir, oth) in grid.get_adjacent(&pos) {
                let target = pos + *dir;
                let cost = cost.get_at(&target);
                if distance_tile >= oth + cost + 1 {
                    propagations.push((target, distance_tile - cost));
                    frontier.push_back(target);
                }
            }
            for (xy, propagation) in propagations {
                grid.set_at(&xy, propagation);
            }
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod test_propagation {
    use crate::{propagate_case, v};
    use std::{collections::HashMap, i32};

    use super::*;

    fn generate_uniform_expected(size: IVec2, sources: &Vec<IVec2>) -> Grid<i32> {
        let mut out = Grid::fill(size, -MAX_DISTANCE_NEG);
        for xy in iter_area(size) {
            out.set_at(
                &xy,
                sources
                    .iter()
                    .map(|source| (xy - *source).length())
                    .min()
                    .unwrap_or(-MAX_DISTANCE_NEG),
            );
        }
        out
    }

    fn uniform_propagation(size: IVec2, sources: Vec<IVec2>) {
        let cost = Grid::fill(size, 1);
        let mut grid = Grid::fill(size, MAX_DISTANCE_NEG);

        let expected = generate_uniform_expected(size, &sources);

        for xy in &sources {
            grid.set_at(&xy, 0);
        }

        propagate_distance_sources(&mut grid, &cost, sources);
        for xy in iter_area(grid.size) {
            *grid.get_at_mut(&xy) *= -1;
        }
        for xy in iter_area(size) {
            assert_eq!(grid.get_at(&xy), expected.get_at(&xy), " at {}", xy);
        }
    }

    #[test]
    fn uniform_cost_one_source() {
        uniform_propagation(IVec2::new(3, 3), vec![IVec2::new(1, 1)]);
    }

    #[test]
    fn uniform_cost_two_sources() {
        uniform_propagation(IVec2::new(3, 3), vec![IVec2::new(1, 2), IVec2::new(0, 0)]);
    }

    #[test]
    fn uniform_cost_all_sources() {
        uniform_propagation(IVec2::new(3, 3), iter_area(IVec2::new(3, 3)).collect());
    }

    #[test]
    fn uniform_cost_no_sources() {
        let size = v!(3, 3);
        let sources = vec![];
        let cost = Grid::fill(size, 1);
        let mut grid = Grid::fill(size, MAX_DISTANCE_NEG);
        let expected = Grid::fill(size, 0);
        for xy in &sources {
            grid.set_at(&xy, 0);
        }

        propagate_distance_sources(&mut grid, &cost, sources);
        for xy in iter_area(grid.size) {
            *grid.get_at_mut(&xy) *= -1;
        }
        for xy in iter_area(size) {
            assert_eq!(grid.get_at(&xy), expected.get_at(&xy), " at {}", xy);
        }
    }

    #[test]
    fn varying_cost_one_source() {
        let size = IVec2::new(4, 1);
        let sources = vec![IVec2::new(0, 0)];
        let mut grid = Grid::fill(size, MAX_DISTANCE_NEG);

        let mut cost = Grid::fill(size, 1);
        cost.set_at(&IVec2::new(0, 0), 2);
        cost.set_at(&IVec2::new(1, 0), 2);
        cost.set_at(&IVec2::new(2, 0), 2);
        cost.set_at(&IVec2::new(3, 0), 2);

        let mut expected = grid.clone();
        expected.set_at(&IVec2::new(0, 0), 0);
        expected.set_at(&IVec2::new(1, 0), 2);
        expected.set_at(&IVec2::new(2, 0), 4);
        expected.set_at(&IVec2::new(3, 0), 6);

        for xy in &sources {
            grid.set_at(&xy, 0);
        }

        propagate_distance_sources(&mut grid, &cost, sources);
        for xy in iter_area(grid.size) {
            *grid.get_at_mut(&xy) *= -1;
        }
        for xy in iter_area(size) {
            assert_eq!(grid.get_at(&xy), expected.get_at(&xy), " at {}", xy);
        }
    }

    propagate_case!(
        drop_walled_wide,
        r"
        1 1 4 1 1
        1 4 1 4 1
        4 1 1 1 4
        1 4 1 4 1
        1 1 4 1 1",
        r"
        7 6 5 6 7
        6 5 1 5 6
        5 1 0 1 5
        6 5 1 5 6
        7 6 5 6 7
        ",
        vec![(IVec2::new(2, 2))]
    );

    propagate_case!(
        drop_walled,
        r"
        1 1 1 1 1
        1 1 4 1 1
        1 4 1 4 1
        1 1 4 1 1
        1 1 1 1 1",
        r"
        7 6 5 6 7
        6 5 4 5 6
        5 4 0 4 5
        6 5 4 5 6
        7 6 5 6 7
        ",
        vec![(IVec2::new(2, 2))]
    );

    propagate_case!(
        drop,
        r"
        1 1 1 1 1
        1 1 1 1 1
        1 1 1 1 1
        1 1 4 1 1
        1 1 0 1 1",
        r"
        4 3 2 3 4
        3 2 1 2 3
        2 1 0 1 2
        3 2 4 2 3
        4 3 3 3 4
        ",
        vec![(IVec2::new(2, 2))]
    );

    propagate_case!(
        wall,
        r"
        2 2 100 2
        2 2 101 2",
        r"
        0 2 102 104
        2 4 105 106",
        vec![(IVec2::new(0, 0))]
    );

    #[macro_export]
    macro_rules! propagate_case {
        ($name: ident, $cost: literal, $expected: literal, $sources: expr) => {
            #[test]
            fn $name() {
                let cost = str_to_i32_grid($cost);
                let expected = str_to_i32_grid($expected);
                assert_eq!(cost.size, expected.size);

                let size = cost.size;
                let mut grid = Grid::fill(size, MAX_DISTANCE_NEG);

                for xy in $sources {
                    grid.set_at(&xy, 0);
                }

                propagate_distance_sources(&mut grid, &cost, $sources);
                for xy in iter_area(grid.size) {
                    *grid.get_at_mut(&xy) *= -1;
                }
                for xy in iter_area(size) {
                    assert_eq!(
                        grid.get_at(&xy),
                        expected.get_at(&xy),
                        " at {}\n cost:\n{} grid:\n{} expected:\n{}",
                        xy,
                        i32_grid_to_str(&cost),
                        i32_grid_to_str(&grid),
                        i32_grid_to_str(&expected)
                    );
                }
            }
        };
    }

    fn i32_grid_to_str(grid: &Grid<i32>) -> String {
        let mut s = String::new();
        for y in 0..grid.size.y {
            for x in 0..grid.size.x {
                s += &(grid.get_at(&IVec2::new(x, y)).to_string() + " ");
            }
            s += "\n"
        }
        s
    }

    fn str_to_i32_grid(s: &str) -> Grid<i32> {
        let mut map = HashMap::<IVec2, i32>::new();
        let mut max = IVec2::ZERO;
        for (y, line) in s
            .split("\n")
            .filter(|l| !l.is_empty())
            .map(|l| l.trim())
            .enumerate()
        {
            for (x, s) in line
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|s| s.trim())
                .enumerate()
            {
                let pos = IVec2::new(x as i32, y as i32);
                max = max.max(pos);
                map.insert(pos, s.parse().unwrap());
            }
        }
        let mut cost = Grid::fill(max + IVec2::ONE, 0);
        for (pos, value) in map {
            cost.set_at(&pos, value)
        }
        cost
    }
}
