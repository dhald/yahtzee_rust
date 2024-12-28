use category::Category;
use clap::Parser;
use dice::{Dice, KeepAndRoll};
use rayon::{prelude::*, ThreadPoolBuilder};
use std::time::Instant;
use turn::Turn;

mod category;
mod dice;
mod turn;

use dice::KeepAndRollData;
use strum::IntoEnumIterator;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "1")]
    threads: usize,

    #[arg(short, long, default_value = "1000")]
    chunk_size: usize,
}

fn compute_end_turn_evs(
    turn: &Turn,
    turn_evs: &Vec<f32>,
    all_rolls: &Vec<Dice>,
    roll_scratch: &mut Vec<f32>,
) {
    let Turn {
        used_categories,
        upper_category_score,
    } = turn;
    for (roll_index, roll) in all_rolls.iter().enumerate() {
        let ev = Category::iter()
            .map(|category| {
                if used_categories.is_set(&category) {
                    0.0
                } else {
                    let score = category.score(roll);
                    let used_categories = used_categories.set(&category);
                    let upper_category_score = if category.is_upper_category() {
                        upper_category_score + score
                    } else {
                        *upper_category_score
                    };
                    let new_turn = Turn::new(used_categories, upper_category_score);
                    score as f32 + turn_evs[new_turn.as_int() as usize]
                }
            })
            .reduce(|f1, f2| f1.max(f2))
            .unwrap();
        roll_scratch[roll_index] = ev;
    }
}

fn populate_keep_evs(
    keeps_and_rolls: &Vec<KeepAndRoll>,
    keep_scratch: &mut Vec<f32>,
    roll_scratch: &Vec<f32>,
) {
    keep_scratch.fill(0.0);
    for KeepAndRoll {
        keep_index,
        roll_index,
        probability,
    } in keeps_and_rolls
    {
        let added_ev = probability * roll_scratch[usize::from(*roll_index)];
        let prev_ev = keep_scratch[usize::from(*keep_index)];
        keep_scratch[usize::from(*keep_index)] = added_ev + prev_ev;
    }
}

fn populate_roll_evs(
    keeps_and_rolls: &Vec<KeepAndRoll>,
    keep_scratch: &Vec<f32>,
    roll_scratch: &mut Vec<f32>,
) {
    roll_scratch.fill(0.0);
    for KeepAndRoll {
        keep_index,
        roll_index,
        probability: _,
    } in keeps_and_rolls
    {
        let prev_ev = roll_scratch[usize::from(*roll_index)];
        let new_ev = keep_scratch[usize::from(*keep_index)];
        roll_scratch[usize::from(*roll_index)] = prev_ev.max(new_ev);
    }
}

fn compute_begin_turn_ev(roll_scratch: &Vec<f32>, all_rolls: &Vec<Dice>) -> f32 {
    all_rolls
        .iter()
        .enumerate()
        .map(|(roll_index, roll)| {
            let probability = roll.probability_to_roll();
            let ev = roll_scratch[roll_index];
            probability * ev
        })
        .sum()
}

fn compute_turn_ev(
    turn: &Turn,
    keeps_and_rolls: &Vec<KeepAndRoll>,
    keep_scratch: &mut Vec<f32>,
    roll_scratch: &mut Vec<f32>,
    all_rolls: &Vec<Dice>,
    turn_evs: &Vec<f32>,
) -> f32 {
    let Turn {
        used_categories,
        upper_category_score,
    } = turn;

    let ev = if used_categories.all_set() {
        if upper_category_score >= &63 {
            35.0
        } else {
            0.0
        }
    } else {
        compute_end_turn_evs(turn, turn_evs, all_rolls, roll_scratch);
        populate_keep_evs(keeps_and_rolls, keep_scratch, roll_scratch);
        populate_roll_evs(keeps_and_rolls, keep_scratch, roll_scratch);
        populate_keep_evs(keeps_and_rolls, keep_scratch, roll_scratch);
        populate_roll_evs(keeps_and_rolls, keep_scratch, roll_scratch);
        compute_begin_turn_ev(roll_scratch, all_rolls)
    };

    /*     println!("Computed turn EV. Turn: {:?}. EV: {}.", turn, ev); */

    ev
}

fn main() {
    let Args {
        threads,
        chunk_size,
    } = Args::parse();
    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    let KeepAndRollData {
        keeps_and_rolls,
        all_rolls,
        n_keeps,
        n_rolls,
    } = KeepAndRoll::all();

    let turns_by_num_used_categories = Turn::all();

    let mut turn_evs: Vec<f32> = vec![0.0; Turn::TOTAL_TURNS as usize];

    let now = Instant::now();

    for turns in turns_by_num_used_categories.iter().rev() {
        let partial_turn_evs = turns
            .par_chunks(chunk_size)
            .flat_map(|turns| {
                let mut keep_scratch = vec![0.0; usize::from(n_keeps)];
                let mut roll_scratch = vec![0.0; usize::from(n_rolls)];
                let turn_evs = turns
                    .iter()
                    .map(|turn| {
                        (
                            turn,
                            compute_turn_ev(
                                turn,
                                &keeps_and_rolls,
                                &mut keep_scratch,
                                &mut roll_scratch,
                                &all_rolls,
                                &turn_evs,
                            ),
                        )
                    })
                    .collect::<Vec<(&Turn, f32)>>();

                turn_evs
            })
            .collect::<Vec<(&Turn, f32)>>();

        partial_turn_evs.iter().for_each(|(turn, ev)| {
            turn_evs[turn.as_int() as usize] = *ev;
        });
    }

    let time_elapsed = now.elapsed();
    println!(
        "Finished computing EVs for all turns. Elapsed time: {:?}. EVs: {:?}.",
        time_elapsed,
        turn_evs[0..50].to_vec()
    );
}
