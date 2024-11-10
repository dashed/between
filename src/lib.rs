use std::cmp;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::iter::FromIterator;

use itertools::Itertools;

/// A struct that provides functionality to find a string that is lexicographically
/// between two given strings, using a specified set of characters.
#[derive(Debug, Clone)]
pub struct Between {
    chars: Vec<char>,
    chars_set: HashSet<char>,
    chars_lookup: HashMap<char, usize>,
    high: char,
    low: char,
}

impl Between {
    /// Creates a new `Between` instance with a given set of characters.
    ///
    /// # Arguments
    ///
    /// * `chars` - A vector of characters to be used for generating between strings.
    ///
    /// # Panics
    ///
    /// Panics if the provided character set has fewer than two distinct characters.
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

    /// Initializes a `Between` instance with a default set of characters.
    pub fn init() -> Self {
        Default::default()
    }

    /// Returns a reference to the vector of characters used by this instance.
    pub fn chars(&self) -> &Vec<char> {
        &self.chars
    }

    /// Returns the highest character in the character set.
    pub fn high(&self) -> char {
        self.high
    }

    /// Returns the lowest character in the character set.
    pub fn low(&self) -> char {
        self.low
    }

    /// Checks if a given string is valid, i.e., contains only characters from the character set.
    ///
    /// # Arguments
    ///
    /// * `string` - A string to be validated.
    ///
    /// # Returns
    ///
    /// `true` if the string is valid, `false` otherwise.
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

    /// Finds a string that is lexicographically between two given strings.
    ///
    /// # Arguments
    ///
    /// * `this` - The first string.
    /// * `that` - The second string.
    ///
    /// # Returns
    ///
    /// An `Option<String>` that contains the between string if possible, or `None` if not.
    pub fn between<S, T>(&self, this: S, that: T) -> Option<String>
    where
        S: Into<String>,
        T: Into<String>,
    {
        // Convert the input parameters into Strings.
        // This allows us to work uniformly with the data regardless of the input types.
        let this: String = this.into();
        let that: String = that.into();

        // Trim any trailing occurrences of the lowest character from 'this' and 'that'.
        // This step is crucial because trailing low characters can complicate comparisons.
        // For instance, 'abc' and 'abc!' (if '!' is the lowest character) might not compare as expected.
        let this: String = this.trim_end_matches(self.low).into();
        let that: String = that.trim_end_matches(self.low).into();

        // Validate the inputs:
        // - Ensure 'this' is lexicographically less than 'that'.
        // - Ensure both 'this' and 'that' are valid strings (contain only characters from 'self.chars').
        // - We allow 'this' to be empty only if 'that' is valid and not empty.
        if this.cmp(&that) != Ordering::Less
            || (!this.is_empty() && !self.valid(&this))
            || !self.valid(&that)
        {
            // If any of the above conditions are not met, we cannot find a 'between' string.
            // Return 'None' to indicate that no valid string can be generated.
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

        // At this point, we have two valid strings 'this' and 'that', with 'this' < 'that'.
        // Our goal is to construct a new string 'between_string' that is lexicographically
        // between 'this' and 'that', using only characters from 'self.chars'.

        // Convert 'this' and 'that' into vectors of characters for easier indexing and comparison.
        let this_chars: Vec<char> = this.chars().collect();
        let that_chars: Vec<char> = that.chars().collect();

        // Initialize an empty vector to build the 'between_string'.
        let mut between_string: Vec<char> = vec![];

        // Set up a guard to prevent infinite loops.
        // The maximum number of iterations is the sum of the lengths of 'this' and 'that'.
        // This ensures that the loop will terminate even in edge cases.
        let guard = this.len() + that.len();

        // Determine the maximum length between 'this' and 'that'.
        // This helps us decide when we might need to consider adding new characters.
        let guard_max_len = cmp::max(this.len(), that.len());

        // Initialize the index to 0, to start processing from the first character.
        let mut index = 0;

        // Begin iterating over the characters to build 'between_string'.
        while index <= guard {
            // For the current index, get the character positions in 'self.chars' for both 'this' and 'that'.

            let this_char_position: usize = {
                // Attempt to get the character from 'this' at the current index.
                // If 'this' is shorter than the current index, we default to 'self.low' (lowest character).
                let this_char = this_chars.get(index).unwrap_or(&self.low);
                // Look up the index of 'this_char' in our character set.
                // Since 'this' is valid, this should not fail.
                *self.chars_lookup.get(this_char).unwrap()
            };

            let that_char_position: usize = {
                // Similarly, attempt to get the character from 'that' at the current index.
                // If 'that' is shorter than the current index, we default to 'self.high' (highest character).
                let that_char = that_chars.get(index).unwrap_or(&self.high);
                // Look up the index of 'that_char' in our character set.
                *self.chars_lookup.get(that_char).unwrap()
            };

            // Now, 'this_char_position' and 'that_char_position' represent the positions of the characters
            // at the current index in 'this' and 'that' within our character set 'self.chars'.
            // Since 'this' is less than 'that', we should have 'this_char_position' <= 'that_char_position'.

            // Our aim is to select a character to add to 'between_string' that will help us
            // construct a string that is lexicographically between 'this' and 'that'.

            // invariant: this_char_position <= that_char_position

            let char_candidate: char = {
                // If there are characters between this_char_position and that_char_position,
                // then we can pick the midpoint of the character candidate between them.
                // We also do this if we go past the maximum length of of either this or that.

                // Determine the position of the candidate character to add.
                let char_position: usize = if ((this_char_position + 1) < that_char_position)
                    // If there are characters available between 'this_char_position' and 'that_char_position':
                    // - This means we can choose a character that is greater than 'this_char' but less than 'that_char'.
                    || index >= guard_max_len
                // Or if we've reached beyond the maximum length of 'this' and 'that':
                // - This allows us to append additional characters to make 'between_string' greater than 'this'.
                {
                    // invariant: self.chars.len() >= 2
                    // If (this_char_position + 1) < that_char_position, then:
                    //    0 <= this_char_position <= max(self.chars.len() - 3, 0)
                    //    2 <= that_char_position <= self.chars.len() - 1
                    // This implies self.chars.len() >= 3. As in, this works for character sets of size 3 or more.
                    //
                    // For 2 character sets, we rely on: index >= guard_max_len

                    // Calculate the midpoint between 'this_char_position' and 'that_char_position'.
                    // We use the average and round it to the nearest integer to select a middle character.
                    ((this_char_position as f64 + that_char_position as f64) / 2.0).round() as usize
                } else {
                    // We use this_char_position so that the character candidate will be less than that_char_position
                    // in lexicographical order/ASCII order.

                    // If there are no characters in between, and we're still within the lengths,
                    // we use 'this_char_position' to keep 'between_string' as close as possible to 'this'.
                    this_char_position
                };

                // Retrieve the character at 'char_position' from 'self.chars'.
                // This is our candidate character to add to 'between_string'.
                self.chars[char_position]
            };

            // Add the candidate character to 'between_string'.
            between_string.push(char_candidate);

            // Now, we check if 'between_string' satisfies the conditions:
            // - It is lexicographically greater than 'this_chars'.
            // - It is lexicographically less than 'that_chars'.
            // - The last character added is not 'self.low' (to avoid trailing low characters).
            if (this_chars < between_string)
                && (between_string < that_chars)
                && char_candidate != self.low
            {
                // If all conditions are met, we have successfully found a valid 'between' string.
                // Convert 'between_string' from a vector of chars back into a String and return it.
                return Some(String::from_iter(between_string));
            }

            // If the conditions are not met, we proceed to the next index.
            // This allows us to modify the next character in 'between_string' to try to satisfy the conditions.
            index += 1;
        }

        // If we have exhausted all possibilities within the guard limit and not found a valid 'between' string,
        // we return 'None' to indicate failure.
        None
    }

    /// Finds a string that is lexicographically after a given string.
    ///
    /// # Arguments
    ///
    /// * `before_string` - The string to find a successor for.
    ///
    /// # Returns
    ///
    /// An `Option<String>` that contains the successor string if possible, or `None` if not.
    pub fn after<S>(&self, before_string: S) -> Option<String>
    where
        S: Into<String>,
    {
        self.between(before_string, self.high)
    }

    /// Finds a string that is lexicographically before a given string.
    ///
    /// # Arguments
    ///
    /// * `after_string` - The string to find a predecessor for.
    ///
    /// # Returns
    ///
    /// An `Option<String>` that contains the predecessor string if possible, or `None` if not.
    pub fn before<S>(&self, after_string: S) -> Option<String>
    where
        S: Into<String>,
    {
        self.between(self.low, after_string)
    }
}

impl Default for Between {
    /// Provides a default `Between` instance with a predefined set of characters.
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
