extern crate peer;

// Integer arithmetic in i128 is independent of the peer wrapping operations.

fn check_u8(x: u8, y: u8) {
    let a = u8::MIN;
    let b = u8::MAX;
    let c = 11u8;
    let d = 29u8;
    let e = peer::seed_u8();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as u8;
    let direct = peer::combine_u8(peer::wrap_u8(x), y, peer::U8(7), a, b, c, d, e);
    assert_eq!(peer::extract_u8(direct), expected);
    let call: fn(peer::U8, u8, peer::U8, u8, u8, u8, u8, u8) -> peer::U8 = peer::combine_u8;
    let pointed = call(peer::U8(x), y, peer::U8(7), a, b, c, d, e);
    assert_eq!(peer::extract_u8(pointed), expected);
}

fn check_i8(x: i8, y: i8) {
    let a = i8::MIN;
    let b = i8::MAX;
    let c = 11i8;
    let d = 29i8;
    let e = peer::seed_i8();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as i8;
    let direct = peer::combine_i8(peer::wrap_i8(x), y, peer::I8(7), a, b, c, d, e);
    assert_eq!(peer::extract_i8(direct), expected);
    let call: fn(peer::I8, i8, peer::I8, i8, i8, i8, i8, i8) -> peer::I8 = peer::combine_i8;
    let pointed = call(peer::I8(x), y, peer::I8(7), a, b, c, d, e);
    assert_eq!(peer::extract_i8(pointed), expected);
}

fn check_u16(x: u16, y: u16) {
    let a = u16::MIN;
    let b = u16::MAX;
    let c = 11u16;
    let d = 29u16;
    let e = peer::seed_u16();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as u16;
    let direct = peer::combine_u16(peer::wrap_u16(x), y, peer::U16(7), a, b, c, d, e);
    assert_eq!(peer::extract_u16(direct), expected);
    let call: fn(peer::U16, u16, peer::U16, u16, u16, u16, u16, u16) -> peer::U16 = peer::combine_u16;
    let pointed = call(peer::U16(x), y, peer::U16(7), a, b, c, d, e);
    assert_eq!(peer::extract_u16(pointed), expected);
}

fn check_i16(x: i16, y: i16) {
    let a = i16::MIN;
    let b = i16::MAX;
    let c = 11i16;
    let d = 29i16;
    let e = peer::seed_i16();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as i16;
    let direct = peer::combine_i16(peer::wrap_i16(x), y, peer::I16(7), a, b, c, d, e);
    assert_eq!(peer::extract_i16(direct), expected);
    let call: fn(peer::I16, i16, peer::I16, i16, i16, i16, i16, i16) -> peer::I16 = peer::combine_i16;
    let pointed = call(peer::I16(x), y, peer::I16(7), a, b, c, d, e);
    assert_eq!(peer::extract_i16(pointed), expected);
}

fn check_u32(x: u32, y: u32) {
    let a = u32::MIN;
    let b = u32::MAX;
    let c = 11u32;
    let d = 29u32;
    let e = peer::seed_u32();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as u32;
    let direct = peer::combine_u32(peer::wrap_u32(x), y, peer::U32(7), a, b, c, d, e);
    assert_eq!(peer::extract_u32(direct), expected);
    let call: fn(peer::U32, u32, peer::U32, u32, u32, u32, u32, u32) -> peer::U32 = peer::combine_u32;
    let pointed = call(peer::U32(x), y, peer::U32(7), a, b, c, d, e);
    assert_eq!(peer::extract_u32(pointed), expected);
}

fn check_i32(x: i32, y: i32) {
    let a = i32::MIN;
    let b = i32::MAX;
    let c = 11i32;
    let d = 29i32;
    let e = peer::seed_i32();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as i32;
    let direct = peer::combine_i32(peer::wrap_i32(x), y, peer::I32(7), a, b, c, d, e);
    assert_eq!(peer::extract_i32(direct), expected);
    let call: fn(peer::I32, i32, peer::I32, i32, i32, i32, i32, i32) -> peer::I32 = peer::combine_i32;
    let pointed = call(peer::I32(x), y, peer::I32(7), a, b, c, d, e);
    assert_eq!(peer::extract_i32(pointed), expected);
}

fn check_u64(x: u64, y: u64) {
    let a = u64::MIN;
    let b = u64::MAX;
    let c = 11u64;
    let d = 29u64;
    let e = peer::seed_u64();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as u64;
    let direct = peer::combine_u64(peer::wrap_u64(x), y, peer::U64(7), a, b, c, d, e);
    assert_eq!(peer::extract_u64(direct), expected);
    let call: fn(peer::U64, u64, peer::U64, u64, u64, u64, u64, u64) -> peer::U64 = peer::combine_u64;
    let pointed = call(peer::U64(x), y, peer::U64(7), a, b, c, d, e);
    assert_eq!(peer::extract_u64(pointed), expected);
}

fn check_i64(x: i64, y: i64) {
    let a = i64::MIN;
    let b = i64::MAX;
    let c = 11i64;
    let d = 29i64;
    let e = peer::seed_i64();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as i64;
    let direct = peer::combine_i64(peer::wrap_i64(x), y, peer::I64(7), a, b, c, d, e);
    assert_eq!(peer::extract_i64(direct), expected);
    let call: fn(peer::I64, i64, peer::I64, i64, i64, i64, i64, i64) -> peer::I64 = peer::combine_i64;
    let pointed = call(peer::I64(x), y, peer::I64(7), a, b, c, d, e);
    assert_eq!(peer::extract_i64(pointed), expected);
}

fn check_usize(x: usize, y: usize) {
    let a = usize::MIN;
    let b = usize::MAX;
    let c = 11usize;
    let d = 29usize;
    let e = peer::seed_usize();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as usize;
    let direct = peer::combine_usize(peer::wrap_usize(x), y, peer::USIZE(7), a, b, c, d, e);
    assert_eq!(peer::extract_usize(direct), expected);
    let call: fn(peer::USIZE, usize, peer::USIZE, usize, usize, usize, usize, usize) -> peer::USIZE = peer::combine_usize;
    let pointed = call(peer::USIZE(x), y, peer::USIZE(7), a, b, c, d, e);
    assert_eq!(peer::extract_usize(pointed), expected);
}

fn check_isize(x: isize, y: isize) {
    let a = isize::MIN;
    let b = isize::MAX;
    let c = 11isize;
    let d = 29isize;
    let e = peer::seed_isize();
    let expected = ((x as i128) + 3 * (y as i128) - 5 * 7
        + (a as i128) - (b as i128) + (c as i128) - (d as i128) + (e as i128)) as isize;
    let direct = peer::combine_isize(peer::wrap_isize(x), y, peer::ISIZE(7), a, b, c, d, e);
    assert_eq!(peer::extract_isize(direct), expected);
    let call: fn(peer::ISIZE, isize, peer::ISIZE, isize, isize, isize, isize, isize) -> peer::ISIZE = peer::combine_isize;
    let pointed = call(peer::ISIZE(x), y, peer::ISIZE(7), a, b, c, d, e);
    assert_eq!(peer::extract_isize(pointed), expected);
}

fn main() {
    for x in 0..256u16 { for y in 0..256u16 { check_u8(x as u8, y as u8); } }
    for x in 0..256u16 { for y in 0..256u16 { check_i8(x as i8, y as i8); } }
    for x in [u16::MIN, u16::MAX, 0, 1, 17, (u16::MAX / 2)] {
        for y in [u16::MIN, u16::MAX, 0, 1, 29, (u16::MAX / 2)] { check_u16(x, y); }
    }
    for x in [i16::MIN, i16::MAX, 0, 1, 17, (i16::MAX / 2)] {
        for y in [i16::MIN, i16::MAX, 0, 1, 29, (i16::MAX / 2)] { check_i16(x, y); }
    }
    for x in [u32::MIN, u32::MAX, 0, 1, 17, (u32::MAX / 2)] {
        for y in [u32::MIN, u32::MAX, 0, 1, 29, (u32::MAX / 2)] { check_u32(x, y); }
    }
    for x in [i32::MIN, i32::MAX, 0, 1, 17, (i32::MAX / 2)] {
        for y in [i32::MIN, i32::MAX, 0, 1, 29, (i32::MAX / 2)] { check_i32(x, y); }
    }
    for x in [u64::MIN, u64::MAX, 0, 1, 17, (u64::MAX / 2)] {
        for y in [u64::MIN, u64::MAX, 0, 1, 29, (u64::MAX / 2)] { check_u64(x, y); }
    }
    for x in [i64::MIN, i64::MAX, 0, 1, 17, (i64::MAX / 2)] {
        for y in [i64::MIN, i64::MAX, 0, 1, 29, (i64::MAX / 2)] { check_i64(x, y); }
    }
    for x in [usize::MIN, usize::MAX, 0, 1, 17, (usize::MAX / 2)] {
        for y in [usize::MIN, usize::MAX, 0, 1, 29, (usize::MAX / 2)] { check_usize(x, y); }
    }
    for x in [isize::MIN, isize::MAX, 0, 1, 17, (isize::MAX / 2)] {
        for y in [isize::MIN, isize::MAX, 0, 1, 29, (isize::MAX / 2)] { check_isize(x, y); }
    }
    println!("shared integer signatures: 262720 value and pointer results");
}
