//! Numeral primitives shared by parsed prefixes and fixed-content list markers.

/// Word repeats the selected letter after each 26-item cycle: Z, AA, BB.
pub(crate) fn format_word_alphabetic_counter(number: u32, is_uppercase: bool) -> String {
    if number == 0 {
        return String::new();
    }
    let base: u8 = if is_uppercase { b'A' } else { b'a' };
    let letter: char = (base + ((number - 1) % 26) as u8) as char;
    letter.to_string().repeat(((number - 1) / 26 + 1) as usize)
}

pub(crate) fn format_roman_counter(mut number: u32, is_uppercase: bool) -> String {
    const ROMAN_VALUES: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut result: String = String::new();
    for (value, symbol) in ROMAN_VALUES {
        while number >= *value {
            number -= *value;
            result.push_str(symbol);
        }
    }
    if is_uppercase {
        result
    } else {
        result.to_lowercase()
    }
}
