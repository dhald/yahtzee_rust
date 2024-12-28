use crate::category;
use category::Category;
use category::CategorySet;

#[derive(Debug)]
pub struct Turn {
    pub used_categories: CategorySet,
    pub upper_category_score: u8,
}

impl Turn {
    const MAX_UPPER_CATEGORY_SCORE: u8 = 63;
    const NUM_UPPER_CATEGORY_SCORES: u8 = Self::MAX_UPPER_CATEGORY_SCORE + 1;
    pub const TOTAL_TURNS: u32 =
        category::CategorySet::NUM_CATEGORY_SETS as u32 * Self::NUM_UPPER_CATEGORY_SCORES as u32;

    pub fn new(used_categories: CategorySet, upper_category_score: u8) -> Turn {
        let upper_category_score = upper_category_score.min(Self::MAX_UPPER_CATEGORY_SCORE);
        Turn {
            used_categories,
            upper_category_score,
        }
    }

    pub fn as_int(&self) -> u32 {
        let Turn {
            used_categories,
            upper_category_score,
        } = self;
        Self::NUM_UPPER_CATEGORY_SCORES as u32 * used_categories.as_int() as u32
            + u32::from(*upper_category_score)
    }

    pub fn all() -> Vec<Vec<Turn>> {
        unsafe {
            let mut turns_by_num_used_categories: Vec<Vec<Turn>> =
                (0..=Category::NUM_CATEGORIES).map(|_| Vec::new()).collect();
            for category_set in CategorySet::all() {
                let turns = turns_by_num_used_categories
                    .get_unchecked_mut(category_set.num_used_categories() as usize);
                for upper_category_score in 0..Self::NUM_UPPER_CATEGORY_SCORES {
                    turns.push(Turn::new(category_set, upper_category_score));
                }
            }
            turns_by_num_used_categories
        }
    }
}
