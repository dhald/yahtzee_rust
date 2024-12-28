use std::collections::hash_map::Entry;
use std::collections::HashMap;
/* TODO dhald: check generated assembly code */

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct Dice {
    pub ones: u8,
    pub twos: u8,
    pub threes: u8,
    pub fours: u8,
    pub fives: u8,
    pub sixes: u8,
}

impl Dice {
    pub const EMPTY: Dice = Dice {
        ones: 0,
        twos: 0,
        threes: 0,
        fours: 0,
        fives: 0,
        sixes: 0,
    };

    const DICE_PER_ROLL: u8 = 5;

    pub fn combine(dice1: Dice, dice2: Dice) -> Dice {
        Dice {
            ones: dice1.ones + dice2.ones,
            twos: dice1.twos + dice2.twos,
            threes: dice1.threes + dice2.threes,
            fours: dice1.fours + dice2.fours,
            fives: dice1.fives + dice2.fives,
            sixes: dice1.sixes + dice2.sixes,
        }
    }

    pub fn n_dice(&self) -> u8 {
        let Dice {
            ones,
            twos,
            threes,
            fours,
            fives,
            sixes,
        } = self;
        ones + twos + threes + fours + fives + sixes
    }

    pub fn sum(&self) -> u8 {
        let Dice {
            ones,
            twos,
            threes,
            fours,
            fives,
            sixes,
        } = self;
        1 * ones + 2 * twos + 3 * threes + 4 * fours + 5 * fives + 6 * sixes
    }

    pub fn has_n_of_a_kind(&self, n: &u8) -> bool {
        let Dice {
            ones,
            twos,
            threes,
            fours,
            fives,
            sixes,
        } = self;
        ones >= n || twos >= n || threes >= n || fours >= n || fives >= n || sixes >= n
    }

    pub fn has_small_straight(&self) -> bool {
        let Dice {
            ones,
            twos,
            threes,
            fours,
            fives,
            sixes,
        } = self;
        ones > &0 && twos > &0 && threes > &0 && fours > &0
            || twos > &0 && threes > &0 && fours > &0 && fives > &0
            || threes > &0 && fours > &0 && fives > &0 && sixes > &0
    }

    pub fn has_large_straight(&self) -> bool {
        let Dice {
            ones,
            twos,
            threes,
            fours,
            fives,
            sixes,
        } = self;
        ones > &0 && twos > &0 && threes > &0 && fours > &0 && fives > &0
            || twos > &0 && threes > &0 && fours > &0 && fives > &0 && sixes > &0
    }

    pub fn has_full_house(&self) -> bool {
        let Dice {
            ones,
            twos,
            threes,
            fours,
            fives,
            sixes,
        } = self;
        (ones == &2 || twos == &2 || threes == &2 || fours == &2 || fives == &2 || sixes == &2)
            && (ones == &3
                || twos == &3
                || threes == &3
                || fours == &3
                || fives == &3
                || sixes == &3)
    }

    fn factorial(n: &u8) -> u32 {
        match n {
            0 => 1,
            1 => 1,
            2 => 2,
            3 => 6,
            4 => 24,
            5 => 120,
            _ => unreachable!(),
        }
    }

    pub fn probability_to_roll(&self) -> f32 {
        let Dice {
            ones,
            twos,
            threes,
            fours,
            fives,
            sixes,
        } = self;
        let n_dice = self.n_dice();
        let numerator = Self::factorial(&n_dice);
        let multinomial_denominator = Self::factorial(ones)
            * Self::factorial(twos)
            * Self::factorial(threes)
            * Self::factorial(fours)
            * Self::factorial(fives)
            * Self::factorial(sixes);
        let total_rolls_denominator = u32::pow(6, n_dice as u32);
        (numerator as f32) / ((total_rolls_denominator * multinomial_denominator) as f32)
    }

    pub fn all_keeps() -> Vec<Dice> {
        let mut keeps = Vec::new();
        keeps.push(Self::EMPTY);
        for value in 1..=6 {
            keeps = keeps
                .iter()
                .flat_map(|keep| {
                    let n_dice = keep.n_dice();
                    let n_dice_remaining = Self::DICE_PER_ROLL - n_dice;
                    (0..=n_dice_remaining).map(|count| {
                        let mut keep = keep.clone();
                        match value {
                            1 => keep.ones = count,
                            2 => keep.twos = count,
                            3 => keep.threes = count,
                            4 => keep.fours = count,
                            5 => keep.fives = count,
                            6 => keep.sixes = count,
                            _ => unreachable!(),
                        };
                        keep
                    })
                })
                .collect()
        }
        keeps
    }
}

pub struct KeepAndRoll {
    pub keep_index: u16,
    pub roll_index: u8,
    pub probability: f32,
}

pub struct KeepAndRollData {
    pub keeps_and_rolls: Vec<KeepAndRoll>,
    pub all_rolls: Vec<Dice>,
    pub n_keeps: u16,
    pub n_rolls: u8,
}

impl KeepAndRoll {
    pub fn all() -> KeepAndRollData {
        unsafe {
            let mut n_keeps = 0;
            let mut n_rolls = 0;
            let mut keeps_and_rolls = Vec::new();
            let mut keeps_by_n_dice: Vec<Vec<Dice>> = Vec::new();
            let mut roll_indices: HashMap<Dice, u8> = HashMap::new();
            let mut all_rolls = Vec::new();
            for _ in 0..=5 {
                keeps_by_n_dice.push(Vec::new());
            }
            for keep in Dice::all_keeps() {
                let n_dice = keep.n_dice();
                keeps_by_n_dice
                    .get_unchecked_mut(usize::from(n_dice))
                    .push(keep);
            }
            for keep_count in 0..=5 {
                let keeps = keeps_by_n_dice.get_unchecked(keep_count);
                let rerolls = keeps_by_n_dice.get_unchecked(5 - keep_count);
                for keep in keeps.iter() {
                    n_keeps += 1;
                    for reroll in rerolls.iter() {
                        let roll = Dice::combine(*keep, *reroll);
                        let roll_index = match roll_indices.entry(roll) {
                            Entry::Occupied(roll_index) => roll_index.into_mut(),
                            Entry::Vacant(v) => {
                                all_rolls.push(roll);
                                n_rolls += 1;
                                v.insert(n_rolls - 1)
                            }
                        };
                        let probability = reroll.probability_to_roll();
                        keeps_and_rolls.push(KeepAndRoll {
                            keep_index: n_keeps - 1,
                            roll_index: *roll_index,
                            probability,
                        })
                    }
                }
            }
            KeepAndRollData {
                keeps_and_rolls,
                all_rolls,
                n_keeps,
                n_rolls,
            }
        }
    }
}
