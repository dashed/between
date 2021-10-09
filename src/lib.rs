use std::cmp;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::iter::FromIterator;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Between {
    chars: Vec<char>,
    chars_set: HashSet<char>,
    chars_lookup: HashMap<char, usize>,
    high: char,
    low: char,
}

impl Between {
    pub fn new(chars: Vec<char>) -> Self {
        let chars: Vec<char> = chars.into_iter().unique().sorted_unstable().collect();
        assert!(
            chars.len() >= 2,
            "Expect chars to have at least two distinct characters."
        );
        let low = chars.first().unwrap();
        let high = chars.last().unwrap();

        let mut chars_lookup: HashMap<char, usize> = HashMap::new();
        for (index, c) in chars.iter().enumerate() {
            chars_lookup.insert(*c, index);
        }

        Between {
            high: *high,
            low: *low,
            chars_set: chars.iter().cloned().collect(),
            chars_lookup,
            chars,
        }
    }

    pub fn init() -> Self {
        Default::default()
    }

    pub fn chars(&self) -> &Vec<char> {
        &self.chars
    }

    pub fn high(&self) -> char {
        self.high
    }

    pub fn low(&self) -> char {
        self.low
    }

    pub fn valid<S>(&self, string: S) -> bool
    where
        S: Into<String>,
    {
        let string: String = string.into();
        if string.is_empty() {
            return false;
        }
        for c in string.chars() {
            if !self.chars_set.contains(&c) {
                return false;
            }
        }
        true
    }

    /// Generate a string that can sort between `this` and `that`.
    pub fn between<S, T>(&self, this: S, that: T) -> Option<String>
    where
        S: Into<String>,
        T: Into<String>,
    {
        // trim any self.low chars on the right
        let this: String = this.into();
        let this: String = this.trim_end_matches(self.low).into();

        let that: String = that.into();
        let that: String = that.trim_end_matches(self.low).into();

        if this.cmp(&that) != Ordering::Less
            || (!this.is_empty() && !self.valid(&this))
            || !self.valid(&that)
        {
            return None;
        }

        // invariant: this < that

        let this_chars: Vec<char> = this.chars().collect();
        let that_chars: Vec<char> = that.chars().collect();

        let mut between_string: Vec<char> = vec![];
        let guard = this.len() + that.len();
        let guard2 = cmp::max(this.len(), that.len());
        let mut index = 0;

        while index <= guard {
            let this_char_position: usize = {
                let this_char = this_chars.get(index).unwrap_or(&self.low);
                *self.chars_lookup.get(this_char).unwrap()
            };

            let that_char_position: usize = {
                let that_char = that_chars.get(index).unwrap_or(&self.high);
                *self.chars_lookup.get(that_char).unwrap_or(&0)
            };

            let char_candidate: char = {
                let char_position: usize = if ((this_char_position + 1) < that_char_position)
                    || index >= guard2
                {
                    ((this_char_position as f64 + that_char_position as f64) / 2.0).round() as usize
                } else {
                    this_char_position
                };

                self.chars[char_position]
            };

            between_string.push(char_candidate);

            if (this_chars < between_string)
                && (between_string < that_chars)
                && char_candidate != self.low
            {
                return Some(String::from_iter(between_string));
            }

            index += 1;
        }

        None
    }

    pub fn after<S>(&self, before_string: S) -> Option<String>
    where
        S: Into<String>,
    {
        self.between(before_string, self.high)
    }

    pub fn before<S>(&self, after_string: S) -> Option<String>
    where
        S: Into<String>,
    {
        self.between(self.low, after_string)
    }
}

impl Default for Between {
    fn default() -> Self {
        let default_chars: Vec<char> =
            "!0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~"
                .chars()
                .collect();
        Between::new(default_chars)
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use crate::Between;

    #[test]
    fn panics_on_invalid_chars() {
        let result = std::panic::catch_unwind(|| Between::new(vec![]));
        assert!(result.is_err());

        let result = std::panic::catch_unwind(|| Between::new(vec!['a']));
        assert!(result.is_err());
    }

    #[test]
    fn sorts_and_dedupes_given_chars() {
        let chars = vec!['c', 'b', 'a', 'c'];
        let between = Between::new(chars);
        assert_eq!(between.chars(), &vec!['a', 'b', 'c']);

        assert_eq!(between.low(), 'a');
        assert_eq!(between.high(), 'c');
    }

    #[test]
    fn inits_default() {
        let between = Between::init();
        assert_eq!(
            String::from_iter(between.chars()),
            "!0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~"
        );
        assert_eq!(between.low(), '!');
        assert_eq!(between.high(), '~');
    }

    #[test]
    fn test_valid() {
        let between = Between::init();
        assert!(between.valid("") == false);
        assert!(between.valid("abc") == true);
        assert!(between.valid("ab$c") == false);
    }

    #[test]
    fn test_two_char_sets() {
        let between = Between::new("01".chars().collect());
        assert!(between.valid("") == false);
        assert!(between.valid("abc") == false);
        assert!(between.valid("010") == true);

        let result = between.between('0', "001");
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result == "0001", "{}", result);
        assert!(between.low().to_string() < result);
        assert!(result < between.high().to_string());
        assert!("0" < &result);
        assert!(result < "001".to_string());

        assert!(between.between("001", '0').is_none());
        assert!(between.between("001", "").is_none());

        let result = between.between("", "001").unwrap();
        assert!(result == "0001", "{}", result);

        assert_eq!(between.after("").unwrap(), "01");
        assert!(between.after("").unwrap() > "".to_string());
        assert!(between.after("").unwrap() > between.low().to_string());

        assert_eq!(between.after("0").unwrap(), "01");
        assert!(between.after("0").unwrap() > "0".to_string());
        assert!(between.after("0").unwrap() > between.low().to_string());

        assert_eq!(between.after("00").unwrap(), "01");
        assert!(between.after("00").unwrap() > "00".to_string());
        assert!(between.after("00").unwrap() > between.low().to_string());

        assert!(between.before("").is_none());

        assert_eq!(between.before("1").unwrap(), "01");
        assert!(between.before("1").unwrap() < "1".to_string());

        assert_eq!(between.before("11").unwrap(), "001");
        assert!(between.before("11").unwrap() < "11".to_string());
    }
}
