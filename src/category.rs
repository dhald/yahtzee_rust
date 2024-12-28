use crate::dice::Dice;
use strum::EnumCount;
use strum_macros::EnumCount;
use strum_macros::EnumIter;

#[derive(EnumIter, EnumCount)]
pub enum Category {
    Ones,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    Chance,
    ThreeOfAKind,
    FourOfAKind,
    FullHouse,
    SmallStraight,
    LargeStraight,
    Yahtzee,
}

impl Category {
    pub const NUM_CATEGORIES: usize = Self::COUNT;

    pub fn score(&self, dice: &Dice) -> u8 {
        match self {
            Category::Ones => 1 * dice.ones,
            Category::Twos => 2 * dice.twos,
            Category::Threes => 3 * dice.threes,
            Category::Fours => 4 * dice.fours,
            Category::Fives => 5 * dice.fives,
            Category::Sixes => 6 * dice.sixes,
            Category::Chance => dice.sum(),
            Category::ThreeOfAKind => {
                if dice.has_n_of_a_kind(&3) {
                    dice.sum()
                } else {
                    0
                }
            }
            Category::FourOfAKind => {
                if dice.has_n_of_a_kind(&4) {
                    dice.sum()
                } else {
                    0
                }
            }
            Category::FullHouse => {
                if dice.has_full_house() {
                    25
                } else {
                    0
                }
            }
            Category::SmallStraight => {
                if dice.has_small_straight() {
                    30
                } else {
                    0
                }
            }
            Category::LargeStraight => {
                if dice.has_large_straight() {
                    40
                } else {
                    0
                }
            }
            Category::Yahtzee => {
                if dice.has_n_of_a_kind(&5) {
                    50
                } else {
                    0
                }
            }
        }
    }

    pub fn is_upper_category(&self) -> bool {
        match self {
            Category::Ones
            | Category::Twos
            | Category::Threes
            | Category::Fours
            | Category::Fives
            | Category::Sixes => true,
            Category::Chance
            | Category::ThreeOfAKind
            | Category::FourOfAKind
            | Category::FullHouse
            | Category::SmallStraight
            | Category::LargeStraight
            | Category::Yahtzee => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CategorySet(pub u16);

impl CategorySet {
    fn category_offset(category: &Category) -> u8 {
        match category {
            Category::Ones => 0,
            Category::Twos => 1,
            Category::Threes => 2,
            Category::Fours => 3,
            Category::Fives => 4,
            Category::Sixes => 5,
            Category::Chance => 6,
            Category::ThreeOfAKind => 7,
            Category::FourOfAKind => 8,
            Category::FullHouse => 9,
            Category::SmallStraight => 10,
            Category::LargeStraight => 11,
            Category::Yahtzee => 12,
        }
    }

    pub const NUM_CATEGORY_SETS: u16 = 1 << Category::NUM_CATEGORIES;
    pub const ALL_CATEGORIES_SET: u16 = Self::NUM_CATEGORY_SETS - 1;

    pub fn as_int(&self) -> u16 {
        self.0
    }

    pub fn num_used_categories(&self) -> u32 {
        self.0.count_ones()
    }

    /*  does this get constant folded? */
    fn category_mask(category: &Category) -> u16 {
        1 << Self::category_offset(category)
    }

    pub fn set(&self, category: &Category) -> Self {
        let category_mask = Self::category_mask(category);
        CategorySet(self.0 | category_mask)
    }

    pub fn is_set(&self, category: &Category) -> bool {
        let category_mask = Self::category_mask(category);
        category_mask & self.0 == category_mask
    }

    pub fn all_set(&self) -> bool {
        self.0 == Self::ALL_CATEGORIES_SET
    }

    pub fn all() -> Vec<CategorySet> {
        (0..Self::NUM_CATEGORY_SETS).map(CategorySet).collect()
    }
}
