use arbitrary::Arbitrary;
use std::fmt;
use std::ops::{Add, Mul, Sub};
use std::{cmp::Ordering, iter::zip};

#[derive(Debug, Clone)]
pub struct Decimal {
    int: String,
    frac: String,
    is_positive: bool,
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sign = "";
        if !self.is_positive {
            sign = "-";
        }
        write!(f, "{}{}.{}", sign, self.int, self.frac)
    }
}

pub enum ParseDecimalError {
    Empty,
    InvalidDigit,
}

impl Decimal {
    pub fn new(mut numstr: String) -> Result<Self, ParseDecimalError> {
        if numstr.is_empty() {
            return Err(ParseDecimalError::Empty);
        }

        let mut is_positive = true;
        if numstr.starts_with('-') {
            is_positive = false;
            numstr.remove(0);
        } else if numstr.starts_with('+') {
            is_positive = true;
            numstr.remove(0);
        }

        let (int, frac) = numstr.split_once('.').unwrap_or((&numstr, ""));

        if int.chars().any(|c| !c.is_ascii_digit()) || frac.chars().any(|c| !c.is_ascii_digit()) {
            return Err(ParseDecimalError::InvalidDigit);
        }

        Ok(Self {
            is_positive,
            int: int.to_owned(),
            frac: frac.to_owned(),
        })
    }

    pub fn try_from(input: &str) -> Option<Self> {
        Self::new(input.to_owned()).ok()
    }

    pub fn is_empty(&self) -> bool {
        if self.int.is_empty() && self.frac.is_empty() {
            return true;
        }
        false
    }

    pub fn is_zero(&self) -> bool {
        if self.int.trim_matches('0').is_empty() && self.frac.trim_matches('0').is_empty() {
            return true;
        }
        false
    }
}

impl PartialEq for Decimal {
    fn eq(&self, _other: &Self) -> bool {
        if self.int.trim_start_matches("0") == _other.int.trim_start_matches("0")
            && self.frac.trim_end_matches("0") == _other.frac.trim_end_matches("0")
            && self.is_positive == _other.is_positive
        {
            return true;
        }
        false
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // If only one number is negative, it is smaller
        if self.is_positive && !other.is_positive {
            return Some(Ordering::Greater);
        }
        if !self.is_positive && other.is_positive {
            return Some(Ordering::Less);
        }
        // If the length of integer part of one number is bigger than other, it is bigger
        let len_cmp = self.int.len().cmp(&other.int.len());
        if len_cmp != Ordering::Equal {
            return Some(if self.is_positive {
                len_cmp
            } else {
                len_cmp.reverse()
            });
        }
        // If integer part of one number is bigger than other, it is bigger
        for (s, o) in zip(self.int.chars(), other.int.chars()) {
            let val_cmp = s.cmp(&o);
            if val_cmp != Ordering::Equal {
                return Some(if self.is_positive {
                    val_cmp
                } else {
                    val_cmp.reverse()
                });
            }
        }

        // If fractional part of one number is bigger than the other, it is bigger than the other
        for (s, o) in zip(self.frac.chars(), other.frac.chars()) {
            let val_cmp = s.cmp(&o);
            if val_cmp != Ordering::Equal {
                return Some(if self.is_positive {
                    val_cmp
                } else {
                    val_cmp.reverse()
                });
            }
        }
        // In case the lengths of fractional parts are different, but they are equal for
        // the length of shorter fractional part, longer length one will be bigger
        // This is for the case like 0.01 and 0.012
        let len_cmp = self.frac.len().cmp(&other.frac.len());
        if len_cmp != Ordering::Equal {
            return Some(if self.is_positive {
                len_cmp
            } else {
                len_cmp.reverse()
            });
        }
        Some(Ordering::Equal)
    }
}

fn pad(s: &mut String, target_len: usize, left: bool) {
    let missing = target_len.saturating_sub(s.len());
    if missing == 0 {
        return;
    }

    let zeros = "0".repeat(missing);
    if left {
        s.insert_str(0, &zeros);
    } else {
        s.push_str(&zeros);
    }
}

impl Add for Decimal {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut self_int = self.int.clone();
        let mut other_int = other.int.clone();
        let mut self_frac = self.frac.clone();
        let mut other_frac = other.frac.clone();

        let int_len = self_int.len().max(other_int.len());
        pad(&mut self_int, int_len, true);
        pad(&mut other_int, int_len, true);

        let frac_len = self_frac.len().max(other_frac.len());
        pad(&mut self_frac, frac_len, false);
        pad(&mut other_frac, frac_len, false);

        let bigger_sign = if (self_int.as_str(), self_frac.as_str())
            > (other_int.as_str(), other_frac.as_str())
        {
            self.is_positive
        } else if (self_int.as_str(), self_frac.as_str())
            < (other_int.as_str(), other_frac.as_str())
        {
            other.is_positive
        } else {
            true
        };

        let mut res;
        let res_is_positive;
        match self.is_positive == other.is_positive {
            // Same sign, sum the numbers, put the sign in front
            true => {
                let mut next_num = 0;
                // summing fractional parts in reverse order
                let mut res_frac = String::new();
                for (s, o) in zip(self_frac.chars().rev(), other_frac.chars().rev()) {
                    let (s_int, o_int) = ((s as u32 - 48), (o as u32 - 48));
                    let mut c_sum = s_int + o_int;
                    if next_num > 0 {
                        c_sum += next_num;
                        next_num = 0;
                    }
                    if c_sum >= 10 {
                        c_sum -= 10;
                        next_num += 1;
                    }
                    let a = char::from_u32(c_sum + 48).unwrap();
                    res_frac.insert(0, a);
                }

                let mut res_int = String::new();
                // summing integer parts in reverse order
                for (s, o) in zip(self_int.chars().rev(), other_int.chars().rev()) {
                    let (s_int, o_int) = ((s as u32 - 48), (o as u32 - 48));
                    let mut c_sum = s_int + o_int;
                    if next_num > 0 {
                        c_sum += next_num;
                        next_num = 0;
                    }
                    if c_sum >= 10 {
                        c_sum -= 10;
                        next_num += 1;
                    }
                    let a = char::from_u32(c_sum + 48).unwrap();
                    res_int.insert(0, a);
                }
                res_is_positive = self.is_positive;
                res = Decimal {
                    int: res_int,
                    frac: res_frac,
                    is_positive: res_is_positive,
                };
            }
            // Different sign, sub abs(smaller) from abs(bigger), put bigger number's sign in front
            false => {
                let self2 = Self {
                    int: self_int,
                    frac: self_frac,
                    is_positive: true,
                };
                let other2 = Self {
                    int: other_int,
                    frac: other_frac,
                    is_positive: true,
                };
                res = self2 - other2;
                res.is_positive = bigger_sign;
            }
        }
        res
    }
}

impl Sub for Decimal {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        let mut self_int = self.int.clone();
        let mut other_int = other.int.clone();
        let mut self_frac = self.frac.clone();
        let mut other_frac = other.frac.clone();
        let int_len = self_int.len().max(other_int.len());
        pad(&mut self_int, int_len, true);
        pad(&mut other_int, int_len, true);

        let frac_len = self_frac.len().max(other_frac.len());
        pad(&mut self_frac, frac_len, false);
        pad(&mut other_frac, frac_len, false);

        let (bigger_int, smaller_int, bigger_frac, smaller_frac) =
            if self_int > other_int || (self_int == other_int && self_frac >= other_frac) {
                (
                    self_int.clone(),
                    other_int.clone(),
                    self_frac.clone(),
                    other_frac.clone(),
                )
            } else {
                (
                    other_int.clone(),
                    self_int.clone(),
                    other_frac.clone(),
                    self_frac.clone(),
                )
            };

        let bigger_sign = if (self_int.as_str(), self_frac.as_str())
            >= (other_int.as_str(), other_frac.as_str())
        {
            self.is_positive
        } else {
            !other.is_positive
        };

        let mut res;
        match (self.is_positive, other.is_positive) {
            // Same sign, sub smaller from the bigger, put the sign of bigger number in front
            (true, true) | (false, false) => {
                let mut borrow = 0;
                // subbing fractional parts in reverse order
                let mut res_frac = String::new();
                for (s, o) in zip(bigger_frac.chars().rev(), smaller_frac.chars().rev()) {
                    let (s_frac, o_frac) = ((s as i32 - 48), (o as i32 - 48));
                    let mut c_sub = s_frac - o_frac - borrow;
                    borrow = 0;
                    if c_sub < 0 {
                        c_sub += 10;
                        borrow += 1;
                    }
                    let a = char::from_u32(c_sub as u32 + 48).unwrap();
                    res_frac.insert(0, a);
                }

                let mut res_int = String::new();
                // subbing integer parts in reverse order
                for (s, o) in zip(bigger_int.chars().rev(), smaller_int.chars().rev()) {
                    let (s_int, o_int) = ((s as i32 - 48), (o as i32 - 48));
                    let mut c_sub = s_int - o_int - borrow;
                    borrow = 0;
                    if c_sub < 0 {
                        c_sub += 10;
                        borrow += 1;
                    }
                    let a = char::from_u32(c_sub as u32 + 48).unwrap();
                    res_int.insert(0, a);
                }
                res = Decimal {
                    int: res_int,
                    frac: res_frac,
                    is_positive: bigger_sign,
                };
            }
            // Different sign, abs(smaller) from abs(bigger), put bigger number's sign in front
            (false, true) => {
                let mut self2 = self;
                self2.is_positive = true;
                res = self2 + other;
                res.is_positive = false;
            }
            (true, false) => {
                let mut other2 = other;
                other2.is_positive = true;
                res = self + other2;
            }
        }
        res
    }
}

fn trim_zeros(s: &str, leading: bool) -> String {
    let t = if leading {
        s.trim_start_matches('0')
    } else {
        s.trim_end_matches('0')
    };
    if t.is_empty() {
        "0".to_owned()
    } else {
        t.to_owned()
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut self_int = self.int.clone();
        let mut rhs_int = rhs.int.clone();

        let int_len = self_int.len().max(rhs_int.len());
        pad(&mut self_int, int_len, true);
        pad(&mut rhs_int, int_len, true);

        let mut self_frac = self.frac.clone();
        let mut rhs_frac = rhs.frac.clone();

        let frac_len = self_frac.len().max(rhs_frac.len());
        pad(&mut self_frac, frac_len, false);
        pad(&mut rhs_frac, frac_len, false);
        let div_index = 2 * self_frac.len();
        self_int.push_str(&self_frac);
        rhs_int.push_str(&rhs_frac);

        let res_vec_len = 2 * rhs_int.len();
        let mut res_vec = vec![0; res_vec_len];
        for (pad_s, s) in rhs_int.char_indices().rev() {
            let mut carry = 0;
            for (pad_f, f) in self_int.char_indices().rev() {
                let (s_int, f_int) = ((s as i32) - 48, (f as i32) - 48); // 5 2
                let mut inter_product = s_int * f_int + carry; // 10
                carry = inter_product / 10; // 1

                inter_product %= 10; // 0
                res_vec[pad_f + pad_s + 1] += inter_product; // 0
                if res_vec[pad_f + pad_s + 1] >= 10 {
                    res_vec[pad_f + pad_s] += 1;
                    res_vec[pad_f + pad_s + 1] -= 10;
                }
            }

            res_vec[pad_s] += carry;
        }
        let mut res_str: String = res_vec
            .iter()
            .map(|&x| char::from_u32(x as u32 + 48).unwrap_or_default())
            .collect();

        if div_index >= res_str.len() {
            pad(&mut res_str, div_index + 1, true);
        }

        let (res_int, res_frac) = res_str.split_at(res_str.len() - div_index);
        let res_int = trim_zeros(res_int, true);
        let res_frac = trim_zeros(res_frac, false);
        let res_is_positive = self.is_positive == rhs.is_positive;
        Decimal {
            int: res_int.to_owned(),
            frac: res_frac.to_owned(),
            is_positive: res_is_positive,
        }
    }
}

impl<'a> Arbitrary<'a> for Decimal {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let s = String::arbitrary(u)?;
        Decimal::try_from(&s).ok_or_else(|| arbitrary::Error::IncorrectFormat)
    }
}
