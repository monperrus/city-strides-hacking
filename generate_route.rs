extern crate city_strides_utils;

use city_strides_utils::cs;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::vec::Vec;

fn path_bfs(
    starting_path: &[i64],
    steps: i32,
    adj_list_all: &HashMap<i64, HashSet<i64>>,
    nodes_all: &HashMap<i64, (f64, f64)>,
    streets: &HashMap<String, Vec<Vec<i64>>>,
) -> Vec<i64> {
    let mut best_path = Vec::new();
    let mut best_completed;
    let mut best_score = 0.0;
    let mut queue = VecDeque::new();
    let done_at_start = cs::streets_completed(starting_path, streets);

    queue.push_back((starting_path.to_vec(), 0.0, steps));

    while let Some((mut path, dist, steps_left)) = queue.pop_front() {
        let done = cs::streets_completed(&path, streets);
        let done_delta = done - done_at_start;
        let score = done_delta as f64 / dist;

        if steps_left == 0 {
            if score >= best_score {
                best_completed = done;
                best_score = score;
                best_path = path.clone();

                if best_score > 0.0 {
                    println!(
                        "{:.2} {} {} / {:.2}",
                        best_score, best_completed, done_delta, dist
                    );
                }
            }
        } else if best_score == 0.0 || best_score - score < 1.0 {
            let curr = path.last().cloned().unwrap_or_default();
            let prev = path.iter().rev().nth(1).cloned().unwrap_or_default();
            let mut choices: Vec<i64> = adj_list_all[&curr].iter().cloned().collect();
            let mut removed = false;

            if choices.contains(&prev) && choices.len() > 1 {
                if let Some(pos) = choices.iter().position(|&x| x == prev) {
                    choices.remove(pos);
                }
                removed = true;
            }

            let mut dist_to_next = 0.0;

            for next in choices.iter() {
                path.push(*next);
                dist_to_next = cs::dist(curr, *next, nodes_all);
                queue.push_back((
                    path.clone(),
                    dist + dist_to_next,
                    steps_left - (choices.len() > 1) as i32,
                ));
                path.pop();
            }

            if removed && choices.len() == 1 && dist_to_next > 0.02 {
                path.push(prev);
                queue.push_back((
                    path.clone(),
                    dist + cs::dist(curr, prev, nodes_all),
                    steps_left - 1,
                ));
            }
        }
    }

    if best_score == 0.0 {
        println!("increasing steps to {}", steps + 2);
        best_path = path_bfs(starting_path, steps + 2, adj_list_all, nodes_all, streets);
    }

    best_path
}

fn node_list_for_csv(path: &[i64], nodes: &HashMap<i64, (f64, f64)>) -> Vec<Vec<String>> {
    path.iter()
        .enumerate()
        .map(|(i, id)| {
            let (lat, lon) = nodes[id];
            vec![
                lat.to_string(),
                lon.to_string(),
                "2".to_string(),
                format!("\"Name: {}\"", id),
                i.to_string(),
            ]
        })
        .collect()
}

#[allow(clippy::enum_clike_unportable_variant)]
#[allow(dead_code)]
enum StartLocations {
    BangkokMarket = 4634908009,
    BangkokDT = 9038489299,
    BangkokDT2 = 3918038819,
    BangkokDT3 = 619810486,
    BangkokDT4 = 2109346563,
    BangkokMarketFar = 1692740969,
    BangkokMarketNear = 1692805767,
    Etobicoke = 21098692,
}

fn main() -> Result<(), Box<dyn Error>> {
    let city = "bangkok"; // Replace with the desired city name.

    // Load JSON data and build dictionaries
    let elements = cs::load_json(city)?;
    let streets = cs::street_dictionary(&elements);

    let elems_all = cs::load_json(format!("{}_all", city).as_str())?;
    let nodes_all = cs::node_dictionary(&elems_all);
    let alist_all = cs::adjacency_list(&cs::street_dictionary(&elems_all));

    const MAX_DISTANCE: f64 = 20.0;

    let mut total_distance = 0.0;
    let mut path = vec![StartLocations::BangkokMarketNear as i64];

    while total_distance < MAX_DISTANCE {
        // Calculate the best path using BFS
        path = path_bfs(&path, 8, &alist_all, &nodes_all, &streets);
        total_distance = cs::distance_of_path_precise(&path, &nodes_all);
        println!("\ntotal_distance = {}\n", total_distance);

        // Write nodes to CSV
        cs::write_nodes_csv(&node_list_for_csv(&path, &nodes_all))?;
    }

    Ok(())
}
