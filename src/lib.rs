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

        // invariant: this < that (in lexographical order/ASCII order)
        //
        // - In lexicographical order/ASCII order, you compare character by character on each string until a difference
        //   is found.
        // - The ASCII value of the characters are used for the comparison.
        // - A lower ASCII value means the character is less than the other character with a higher ASCII value.
        //   In other words, the character is deemed lexicographically smaller than the other character.
        // - If the characters are the same, then you move on to the next character in the string.
        // - In lexicographical order, if a string is a prefix of another string (meaning it matches the beginning of
        //   the longer string), it's considered "smaller".

        let this_chars: Vec<char> = this.chars().collect();
        let that_chars: Vec<char> = that.chars().collect();

        let mut between_string: Vec<char> = vec![];
        // This guard exists to prevent potential infinite loops.
        let guard = this.len() + that.len();
        let guard_max_len = cmp::max(this.len(), that.len());
        let mut index = 0;

        while index <= guard {
            let this_char_position: usize = {
                let this_char = this_chars.get(index).unwrap_or(&self.low);
                *self.chars_lookup.get(this_char).unwrap()
            };

            let that_char_position: usize = {
                let that_char = that_chars.get(index).unwrap_or(&self.high);
                *self.chars_lookup.get(that_char).unwrap()
            };

            // invariant: this_char_position <= that_char_position

            let char_candidate: char = {
                // If there are characters between this_char_position and that_char_position,
                // then we can pick the midpoint of the character candidate between them.
                // We also do this if we go past the maximum length of of either this or that.
                let char_position: usize = if ((this_char_position + 1) < that_char_position)
                    || index >= guard_max_len
                {
                    // invariant: self.chars.len() >= 2
                    // If (this_char_position + 1) < that_char_position, then:
                    //    0 <= this_char_position <= max(self.chars.len() - 3, 0)
                    //    2 <= that_char_position <= self.chars.len() - 1
                    // This implies self.chars.len() >= 3. As in, this works for character sets of size 3 or more.
                    //
                    // For 2 character sets, we rely on: index >= guard_max_len
                    ((this_char_position as f64 + that_char_position as f64) / 2.0).round() as usize
                } else {
                    // We use this_char_position so that the character candidate will be less than that_char_position
                    // in lexicographical order/ASCII order.
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

        assert_eq!(between.after("!!!!").unwrap(), "V");
        assert!(between.before("!!!!").is_none());
        assert_eq!(between.before("!!!0").unwrap(), "!!!!V");

        assert!(between.after("~~~~").is_none());
        assert_eq!(between.before("~~~~").unwrap(), "V");
        assert!(between.after("~~~0").is_none());
        assert_eq!(between.after("0~~0").unwrap(), "W");
        assert!("0~~0" < "W");

        assert_eq!(between.between("A", "B").unwrap(), "AV");
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

        assert!(between.low() == '0');
        assert!(between.high() == '1');

        assert_eq!(between.between("0", '1').unwrap(), "01");

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

        assert_eq!(between.after("0001").unwrap(), "00011");
        assert_eq!(between.before("0001").unwrap(), "00001");

        assert!(between.after("1111").is_none());
        assert_eq!(between.before("1111").unwrap(), "00001");

        assert!(between.after("1110").is_none());
        assert_eq!(between.before("1110").unwrap(), "0001");
    }

    #[test]
    fn test_between_strings_with_same_prefix() {
        let between = Between::new(vec!['a', 'b']);

        // Test between strings with same prefix
        assert_eq!(between.between("a", "ab").unwrap(), "aab");
        assert_eq!(between.between("aa", "ab").unwrap(), "aab");
        assert_eq!(between.between("a", "b").unwrap(), "ab");
    }

    #[test]
    fn test_between_empty_and_non_empty_string() {
        let between = Between::new(vec!['a', 'b', 'c']);

        // Test between empty string and a non-empty string
        assert!(between.between("", "a").is_none());
        assert!(between.between("", "aa").is_none());
        assert!(between.between("", "").is_none());
    }

    #[test]
    fn test_between_with_full_range_of_chars() {
        let between = Between::init();

        // Test between strings using the default character set
        assert_eq!(between.between("A", "B").unwrap(), "AV");
        assert_eq!(between.between("A", "A~").unwrap(), "AV");
        assert_eq!(between.between("A", "A!"), None);
    }

    #[test]
    fn test_between_with_special_characters() {
        let between = Between::new(vec!['!', '@', '#', '$', '%']);

        // Test between strings with special characters
        assert_eq!(between.between("!", "%").unwrap(), "$");
        assert_eq!(between.between("!$", "!%").unwrap(), "!$$");
        assert!(between.between("%", "!").is_none());
    }

    #[test]
    fn test_before_and_after_methods() {
        let between = Between::new(vec!['0', '1']);

        // Test after method
        assert_eq!(between.after("0").unwrap(), "01");
        assert_eq!(between.after("01").unwrap(), "011");

        // Test before method
        assert_eq!(between.before("1").unwrap(), "01");
        assert_eq!(between.before("10").unwrap(), "01");
    }

    #[test]
    fn test_validity_checks() {
        let between = Between::new(vec!['x', 'y', 'z']);

        // Test valid strings
        assert!(between.valid("x"));
        assert!(between.valid("xyz"));
        assert!(!between.valid(""));
        assert!(!between.valid("abc"));
    }

    #[test]
    fn test_empty_strings() {
        let between = Between::new(vec!['a', 'b', 'c']);

        // Both strings are empty
        assert!(between.between("", "").is_none());

        // 'this' is empty, 'that' is valid
        assert!(between.between("", "a").is_none());

        // 'that' is empty, 'this' is valid
        assert!(between.between("a", "").is_none());
    }

    #[test]
    fn test_strings_with_low_high_chars() {
        let between = Between::new(vec!['a', 'b', 'c']);

        // 'this' is at low boundary
        assert_eq!(between.between("a", "b").unwrap(), "ab");

        // 'that' is at high boundary
        assert_eq!(between.between("b", "c").unwrap(), "bb");

        // Both 'this' and 'that' are at boundaries
        assert_eq!(between.between("a", "c").unwrap(), "b");
    }

    #[test]
    fn test_prefix_cases() {
        let between = Between::new(vec!['a', 'b', 'c']);

        // 'this' is a prefix of 'that', but there is no string strictly between
        assert!(between.between("a", "aa").is_none());

        // 'that' is a prefix of 'this' (should return None)
        assert!(between.between("aa", "a").is_none());

        // 'this' equals 'that' (should return None)
        assert!(between.between("abc", "abc").is_none());
    }

    #[test]
    fn test_non_alphanumeric_chars() {
        let between = Between::new(vec!['!', '@', '#', '$', '%']);

        // Test between special characters
        assert_eq!(between.between("!", "%").unwrap(), "$");
        assert!(between.between("%", "!").is_none());
    }

    #[test]
    fn test_full_char_set() {
        let between = Between::init();

        // Test between two strings using full default character set
        assert_eq!(between.between("A", "B").unwrap(), "AV");

        // Test when 'this' is high and 'that' is low (should return None)
        assert!(between.between("~", "!").is_none());
    }

    #[test]
    fn test_after_before_methods_with_boundaries() {
        let between = Between::new(vec!['0', '1']);

        // After the highest possible string (should return None)
        assert!(between.after("1").is_none());

        // Before the lowest possible string (should return None)
        assert!(between.before("0").is_none());

        // After a string consisting of '1's
        assert!(between.after("111").is_none());

        // Before a string consisting of '0's
        assert!(between.before("000").is_none());
    }

    #[test]
    fn test_large_strings() {
        let between = Between::new(vec!['a', 'b', 'c']);

        // Test with very long strings
        let long_a = "a".repeat(1000);
        let long_c = "c".repeat(1000);

        // The minimal string between long_a and long_c is "b"
        assert_eq!(between.between(&long_a, &long_c).unwrap(), "b");

        // Test after a long string
        assert_eq!(between.after(&long_a).unwrap(), "b");

        // Test before a long string
        assert_eq!(between.before(&long_c).unwrap(), "b");
    }

    #[test]
    fn test_invalid_inputs() {
        let between = Between::new(vec!['x', 'y', 'z']);

        // 'this' contains invalid characters
        assert!(between.between("a", "z").is_none());

        // 'that' contains invalid characters
        assert!(between.between("x", "1").is_none());

        // Both contain invalid characters
        assert!(between.between("1", "a").is_none());
    }

    #[test]
    fn test_characters_with_duplicates() {
        let between = Between::new(vec!['a', 'b', 'b', 'c', 'c', 'c']);

        // Duplicates should be removed
        assert_eq!(between.chars(), &vec!['a', 'b', 'c']);

        // Test between
        assert_eq!(between.between("a", "c").unwrap(), "b");
    }

    #[test]
    fn test_single_character_set() {
        let result = std::panic::catch_unwind(|| Between::new(vec!['x']));
        assert!(
            result.is_err(),
            "Should panic when only one character is provided"
        );
    }

    #[test]
    fn test_edge_case_characters() {
        let between = Between::new(vec!['a', 'b', 'c', 'd', 'e', 'f']);

        // Find a string strictly between "abcde" and "abcdf"
        assert_eq!(between.between("abcde", "abcdf").unwrap(), "abcded");

        // Test when 'this' is longer than 'that' (should return None)
        assert!(between.between("abcdef", "abcde").is_none());
    }

    #[test]
    fn test_numeric_characters() {
        let between = Between::new(vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);

        // Test between numeric strings
        assert_eq!(between.between("123", "125").unwrap(), "124");

        // Test after a numeric string (should return None because "999" is the highest)
        assert!(between.after("999").is_none());

        // Test before a numeric string
        assert_eq!(between.before("100").unwrap(), "05");
    }

    #[test]
    fn test_unicode_characters() {
        let between = Between::new(vec!['α', 'β', 'γ', 'δ', 'ε']);

        // Test between Unicode strings
        assert_eq!(between.between("α", "γ").unwrap(), "β");

        // Test after a Unicode string
        assert_eq!(between.after("ε").is_none(), true);

        // Test before a Unicode string
        assert_eq!(between.before("α").is_none(), true);
    }
}
