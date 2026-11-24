// Exhaustive finite-domain arithmetic checks. Expected results use i32/u32
// arithmetic, where every intermediate here is representable. The fixture is
// also run by the normal rustc/native differential and C sanitizer gates.

fn signed_wrap(n: i32) -> i8 {
    let residue = n & 255;
    if residue >= 128 { (residue - 256) as i8 } else { residue as i8 }
}

fn signed_clamp(n: i32) -> i8 {
    if n < -128 { -128 } else if n > 127 { 127 } else { n as i8 }
}

fn unsigned_clamp(n: i32) -> u8 {
    if n < 0 { 0 } else if n > 255 { 255 } else { n as u8 }
}

fn signed_pairs() -> u32 {
    let mut pairs = 0u32;
    let mut x = -128i32;
    while x <= 127 {
        let a = x as i8;
        let mut y = -128i32;
        while y <= 127 {
            let b = y as i8;
            let add = x + y;
            let sub = x - y;
            let mul = x * y;
            let add_overflow = add < -128 || add > 127;
            let sub_overflow = sub < -128 || sub > 127;
            let mul_overflow = mul < -128 || mul > 127;
            assert_eq!(a.wrapping_add(b), signed_wrap(add));
            assert_eq!(a.wrapping_sub(b), signed_wrap(sub));
            assert_eq!(a.wrapping_mul(b), signed_wrap(mul));
            assert_eq!(a.overflowing_add(b), (signed_wrap(add), add_overflow));
            assert_eq!(a.overflowing_sub(b), (signed_wrap(sub), sub_overflow));
            assert_eq!(a.overflowing_mul(b), (signed_wrap(mul), mul_overflow));
            assert_eq!(a.checked_add(b), if add_overflow { None } else { Some(add as i8) });
            assert_eq!(a.checked_sub(b), if sub_overflow { None } else { Some(sub as i8) });
            assert_eq!(a.checked_mul(b), if mul_overflow { None } else { Some(mul as i8) });
            assert_eq!(a.saturating_add(b), signed_clamp(add));
            assert_eq!(a.saturating_sub(b), signed_clamp(sub));
            assert_eq!(a.saturating_mul(b), signed_clamp(mul));
            assert_eq!(a & b, signed_wrap(x & y));
            assert_eq!(a | b, signed_wrap(x | y));
            assert_eq!(a ^ b, signed_wrap(x ^ y));
            assert_eq!(a < b, x < y);
            assert_eq!(a == b, x == y);
            let distance = if x >= y { x - y } else { y - x };
            assert_eq!(a.abs_diff(b), distance as u8);
            if y == 0 {
                assert_eq!(a.checked_div(b), None);
                assert_eq!(a.checked_rem(b), None);
            } else {
                let quotient = x / y;
                let remainder = x % y;
                let magnitude = if y < 0 { -y } else { y };
                let euclid_remainder = if remainder < 0 { remainder + magnitude } else { remainder };
                let euclid_quotient = (x - euclid_remainder) / y;
                let overflow = x == -128 && y == -1;
                assert_eq!(a.wrapping_div(b), signed_wrap(quotient));
                assert_eq!(a.wrapping_rem(b), signed_wrap(remainder));
                assert_eq!(a.overflowing_div(b), (signed_wrap(quotient), overflow));
                assert_eq!(a.overflowing_rem(b), (signed_wrap(remainder), overflow));
                assert_eq!(a.checked_div(b), if overflow { None } else { Some(quotient as i8) });
                assert_eq!(a.checked_rem(b), if overflow { None } else { Some(remainder as i8) });
                assert_eq!(a.saturating_div(b), signed_clamp(quotient));
                if !overflow {
                    assert_eq!(a / b, quotient as i8);
                    assert_eq!(a % b, remainder as i8);
                    assert_eq!(a.div_euclid(b), euclid_quotient as i8);
                    assert_eq!(a.rem_euclid(b), euclid_remainder as i8);
                }
            }
            pairs += 1;
            y += 1;
        }
        x += 1;
    }
    pairs
}

fn unsigned_pairs() -> u32 {
    let mut pairs = 0u32;
    let mut x = 0i32;
    while x <= 255 {
        let a = x as u8;
        let mut y = 0i32;
        while y <= 255 {
            let b = y as u8;
            let add = x + y;
            let sub = x - y;
            let mul = x * y;
            assert_eq!(a.wrapping_add(b), (add & 255) as u8);
            assert_eq!(a.wrapping_sub(b), (sub & 255) as u8);
            assert_eq!(a.wrapping_mul(b), (mul & 255) as u8);
            assert_eq!(a.overflowing_add(b), ((add & 255) as u8, add > 255));
            assert_eq!(a.overflowing_sub(b), ((sub & 255) as u8, sub < 0));
            assert_eq!(a.overflowing_mul(b), ((mul & 255) as u8, mul > 255));
            assert_eq!(a.checked_add(b), if add > 255 { None } else { Some(add as u8) });
            assert_eq!(a.checked_sub(b), if sub < 0 { None } else { Some(sub as u8) });
            assert_eq!(a.checked_mul(b), if mul > 255 { None } else { Some(mul as u8) });
            assert_eq!(a.saturating_add(b), unsigned_clamp(add));
            assert_eq!(a.saturating_sub(b), unsigned_clamp(sub));
            assert_eq!(a.saturating_mul(b), unsigned_clamp(mul));
            assert_eq!(a & b, (x & y) as u8);
            assert_eq!(a | b, (x | y) as u8);
            assert_eq!(a ^ b, (x ^ y) as u8);
            assert_eq!(a < b, x < y);
            assert_eq!(a == b, x == y);
            assert_eq!(a.abs_diff(b), (if x >= y { x - y } else { y - x }) as u8);
            if y == 0 {
                assert_eq!(a.checked_div(b), None);
                assert_eq!(a.checked_rem(b), None);
            } else {
                let quotient = (x / y) as u8;
                let remainder = (x % y) as u8;
                assert_eq!(a / b, quotient);
                assert_eq!(a % b, remainder);
                assert_eq!(a.wrapping_div(b), quotient);
                assert_eq!(a.wrapping_rem(b), remainder);
                assert_eq!(a.overflowing_div(b), (quotient, false));
                assert_eq!(a.overflowing_rem(b), (remainder, false));
                assert_eq!(a.checked_div(b), Some(quotient));
                assert_eq!(a.checked_rem(b), Some(remainder));
                assert_eq!(a.saturating_div(b), quotient);
                assert_eq!(a.div_euclid(b), quotient);
                assert_eq!(a.rem_euclid(b), remainder);
            }
            pairs += 1;
            y += 1;
        }
        x += 1;
    }
    pairs
}

fn unary_and_shifts() -> u32 {
    let amounts = [0u32, 1, 2, 7, 8, 9, 15, 16, 31, 32, 255, u32::MAX];
    let mut cases = 0u32;
    let mut raw = 0u32;
    while raw <= 255 {
        let a = raw as u8;
        let signed = if raw >= 128 { raw as i32 - 256 } else { raw as i32 };
        let b = signed as i8;
        assert_eq!(b.wrapping_neg(), signed_wrap(-signed));
        assert_eq!(b.overflowing_neg(), (signed_wrap(-signed), signed == -128));
        assert_eq!(b.checked_neg(), if signed == -128 { None } else { Some((-signed) as i8) });
        assert_eq!(b.saturating_neg(), signed_clamp(-signed));
        assert_eq!(b.unsigned_abs(), (if signed < 0 { -signed } else { signed }) as u8);
        let mut reversed = 0u32;
        let mut ones = 0u32;
        let mut i = 0u32;
        while i < 8 {
            let bit = (raw >> i) & 1;
            reversed += bit << (7 - i);
            ones += bit;
            i += 1;
        }
        assert_eq!(a.reverse_bits(), reversed as u8);
        assert_eq!(b.reverse_bits(), signed_wrap(reversed as i32));
        assert_eq!(a.count_ones(), ones);
        assert_eq!(b.count_ones(), ones);
        for amount in amounts {
            let shift = amount & 7;
            let scale = 1i32 << shift;
            let left = signed_wrap(signed * scale);
            let right = if signed >= 0 { signed / scale } else { (signed - scale + 1) / scale } as i8;
            let unsigned_left = ((raw << shift) & 255) as u8;
            let unsigned_right = (raw >> shift) as u8;
            let rotate_left = (((raw << shift) | (raw >> (8 - shift))) & 255) as u8;
            let rotate_right = (((raw >> shift) | (raw << (8 - shift))) & 255) as u8;
            assert_eq!(a.wrapping_shl(amount), unsigned_left);
            assert_eq!(a.wrapping_shr(amount), unsigned_right);
            assert_eq!(b.wrapping_shl(amount), left);
            assert_eq!(b.wrapping_shr(amount), right);
            assert_eq!(a.overflowing_shl(amount), (unsigned_left, amount >= 8));
            assert_eq!(a.overflowing_shr(amount), (unsigned_right, amount >= 8));
            assert_eq!(b.overflowing_shl(amount), (left, amount >= 8));
            assert_eq!(b.overflowing_shr(amount), (right, amount >= 8));
            assert_eq!(a.checked_shl(amount), if amount >= 8 { None } else { Some(unsigned_left) });
            assert_eq!(a.checked_shr(amount), if amount >= 8 { None } else { Some(unsigned_right) });
            assert_eq!(b.checked_shl(amount), if amount >= 8 { None } else { Some(left) });
            assert_eq!(b.checked_shr(amount), if amount >= 8 { None } else { Some(right) });
            assert_eq!(a.rotate_left(amount), rotate_left);
            assert_eq!(a.rotate_right(amount), rotate_right);
            assert_eq!(b.rotate_left(amount), signed_wrap(rotate_left as i32));
            assert_eq!(b.rotate_right(amount), signed_wrap(rotate_right as i32));
            cases += 1;
        }
        raw += 1;
    }
    cases
}

fn main() {
    let signed = signed_pairs();
    let unsigned = unsigned_pairs();
    let shifts = unary_and_shifts();
    assert_eq!(signed, 65_536);
    assert_eq!(unsigned, 65_536);
    assert_eq!(shifts, 3_072);
    println!("integer domain verified: {} signed pairs, {} unsigned pairs, {} shift cases", signed, unsigned, shifts);
}
