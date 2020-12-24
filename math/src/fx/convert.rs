use super::fx::Fx;
use fixed::types::I32F32;
use std::mem;

#[inline(always)]
pub const fn ff(n: f64) -> Fx {
    return fx_f64(n);
}

#[inline(always)]
pub const fn fi(n: i64) -> Fx {
    return fx_i64(n);
}

#[inline(always)]
pub const fn fx_isize(n: isize) -> Fx {
    if n > i32::MAX as isize {
        return Fx(I32F32::MAX);
    } else if n < i32::MIN as isize {
        return Fx(I32F32::MIN);
    } else {
        return Fx(I32F32::from_bits((n as i64) << 32));
    }
}

#[inline(always)]
pub const fn fx_usize(n: usize) -> Fx {
    if n > i32::MAX as usize {
        return Fx(I32F32::MAX);
    } else {
        return Fx(I32F32::from_bits((n as i64) << 32));
    }
}

#[inline(always)]
pub const fn fx_i64(n: i64) -> Fx {
    if n > i32::MAX as i64 {
        return Fx(I32F32::MAX);
    } else if n < i32::MIN as i64 {
        return Fx(I32F32::MIN);
    } else {
        return Fx(I32F32::from_bits(n << 32));
    }
}

#[inline(always)]
pub const fn fx_u64(n: u64) -> Fx {
    if n > i32::MAX as u64 {
        return Fx(I32F32::MAX);
    } else {
        return Fx(I32F32::from_bits((n as i64) << 32));
    }
}

#[inline(always)]
pub const fn fx_i32(n: i32) -> Fx {
    return Fx(I32F32::from_bits((n as i64) << 32));
}

#[inline(always)]
pub const fn fx_u32(n: u32) -> Fx {
    return Fx(I32F32::from_bits((n as i64) << 32));
}

#[inline(always)]
pub const fn fx_i16(n: i16) -> Fx {
    return Fx(I32F32::from_bits((n as i64) << 32));
}

#[inline(always)]
pub const fn fx_u16(n: u16) -> Fx {
    return Fx(I32F32::from_bits((n as i64) << 32));
}

#[inline(always)]
pub const fn fx_i8(n: i8) -> Fx {
    return Fx(I32F32::from_bits((n as i64) << 32));
}

#[inline(always)]
pub const fn fx_u8(n: u8) -> Fx {
    return Fx(I32F32::from_bits((n as i64) << 32));
}

union F64Union {
    f: f64,
    i: u64,
}

#[inline]
pub const fn fx_f64(n: f64) -> Fx {
    let bits: u64 = unsafe { F64Union{ f: n }.i };
    let sign = (bits & 0x8000_0000_0000_0000) as i64;
    let expo = ((bits & 0x7FF0_0000_0000_0000) >> 52) as i64 - 1023;
    let base = ((bits & 0x000F_FFFF_FFFF_FFFF) + 0x0010_0000_0000_0000) as i64;
    if expo > 30 {
        if sign >= 0 {
            return Fx(I32F32::MAX);
        } else {
            return Fx(I32F32::MIN);
        };
    } else if expo < -32 {
        return Fx(I32F32::from_bits(0));
    } else {
        let off = -20 + expo;
        let true_code = if off >= 0 { base << off } else { base >> -off };
        let comp_code = if sign >= 0 { true_code } else { !true_code + 1 };
        return Fx(I32F32::from_bits(sign | comp_code));
    }
}

union F32Union {
    f: f32,
    i: u32,
}

#[inline]
pub const fn fx_f32(n: f32) -> Fx {
    let bits: u32 = unsafe { F32Union{ f: n }.i };
    let sign = ((bits & 0x8000_0000) as i64) << 32;
    let expo = ((bits & 0x7F80_0000) >> 23) as i64 - 127;
    let base = ((bits & 0x007F_FFFF) + 0x0080_0000) as i64;
    if expo > 30 {
        if sign >= 0 {
            return Fx(I32F32::MAX);
        } else {
            return Fx(I32F32::MIN);
        };
    } else if expo < -32 {
        return Fx(I32F32::from_bits(0));
    } else {
        let off = 9 + expo;
        let true_code = if off >= 0 { base << off } else { base >> -off };
        let comp_code = if sign >= 0 { true_code } else { !true_code + 1 };
        return Fx(I32F32::from_bits(sign | comp_code));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fx_convert_int() {
        assert_eq!(fx_isize(0), Fx(I32F32::from_num(0)));
        assert_eq!(fx_isize(2147483647), Fx(I32F32::from_num(2147483647)));
        assert_eq!(fx_isize(-2147483648), Fx(I32F32::from_num(-2147483648)));
        assert_eq!(fx_isize(isize::MAX), Fx(I32F32::MAX));
        assert_eq!(fx_isize(isize::MIN), Fx(I32F32::MIN));

        assert_eq!(fx_usize(0), Fx(I32F32::from_num(0)));
        assert_eq!(fx_usize(2147483647), Fx(I32F32::from_num(2147483647)));
        assert_eq!(fx_usize(usize::MAX), Fx(I32F32::MAX));

        assert_eq!(fx_i64(0), Fx(I32F32::from_num(0)));
        assert_eq!(fx_i64(2147483647), Fx(I32F32::from_num(2147483647)));
        assert_eq!(fx_i64(-2147483648), Fx(I32F32::from_num(-2147483648)));
        assert_eq!(fx_i64(i64::MAX), Fx(I32F32::MAX));
        assert_eq!(fx_i64(i64::MIN), Fx(I32F32::MIN));

        assert_eq!(fx_u64(0), Fx(I32F32::from_num(0)));
        assert_eq!(fx_u64(2147483647), Fx(I32F32::from_num(2147483647)));
        assert_eq!(fx_u64(u64::MAX), Fx(I32F32::MAX));
    }

    #[test]
    fn test_fx_convert_f64() {
        assert_eq!(fx_f64(0.0), Fx(I32F32::from_num(0)));
        assert_eq!(fx_f64(1.0), Fx(I32F32::from_num(1)));
        assert_eq!(fx_f64(-1.0), Fx(I32F32::from_num(-1)));
        assert_eq!(fx_f64(2147483646.0), Fx(I32F32::from_num(2147483646)));
        assert_eq!(fx_f64(2147483647.0), Fx(I32F32::from_num(2147483647)));
        assert_eq!(fx_f64(2147483648.0), Fx(I32F32::MAX));
        assert_eq!(fx_f64(-2147483646.0), Fx(I32F32::from_num(-2147483646)));
        assert_eq!(fx_f64(-2147483647.0), Fx(I32F32::from_num(-2147483647)));
        assert_eq!(fx_f64(-2147483648.0), Fx(I32F32::MIN));
        assert_eq!(fx_f64(-2147483649.0), Fx(I32F32::MIN));
        assert_eq!(fx_f64(12345678.0), Fx(I32F32::from_num(12345678)));
        assert_eq!(fx_f64(-12345678.0), Fx(I32F32::from_num(-12345678)));
        assert_eq!(fx_f64(0.5), Fx(I32F32::from_num(0.5)));
        assert_eq!(fx_f64(-0.5), Fx(I32F32::from_num(-0.5)));
        assert_eq!(fx_f64(0.00000000025), Fx(I32F32::from_num(0.00000000025)));
        assert_eq!(fx_f64(-0.00000000025), Fx(I32F32::from_num(-0.00000000025)));
        assert_eq!(fx_f64(0.98765432), Fx(I32F32::from_num(0.98765432)));
        assert_eq!(fx_f64(-0.98765432), Fx(I32F32::from_num(-0.98765432)));
        assert_eq!(fx_f64(0.98765432), Fx(I32F32::from_num(0.98765432)));
        assert_eq!(fx_f64(-0.98765432), Fx(I32F32::from_num(-0.98765432)));
        assert_eq!(fx_f64(f64::INFINITY), Fx(I32F32::MAX));
        assert_eq!(fx_f64(f64::NEG_INFINITY), Fx(I32F32::MIN));
    }

    #[test]
    fn test_fx_convert_f32() {
        assert_eq!(fx_f32(0.0), Fx(I32F32::from_num(0)));
        assert_eq!(fx_f32(1.0), Fx(I32F32::from_num(1)));
        assert_eq!(fx_f32(-1.0), Fx(I32F32::from_num(-1)));
        assert_eq!(fx_f32(2147483392.0), Fx(I32F32::from_num(2147483392.0)));
        assert_eq!(fx_f32(2147483520.0), Fx(I32F32::from_num(2147483520.0)));
        assert_eq!(fx_f32(2147483647.0), Fx(I32F32::MAX));
        assert_eq!(fx_f32(-2147483392.0), Fx(I32F32::from_num(-2147483392.0)));
        assert_eq!(fx_f32(-2147483520.0), Fx(I32F32::from_num(-2147483520.0)));
        assert_eq!(fx_f32(-2147483648.0), Fx(I32F32::MIN));
        assert_eq!(fx_f32(0.5), Fx(I32F32::from_num(0.5)));
        assert_eq!(fx_f32(-0.5), Fx(I32F32::from_num(-0.5)));
        assert_eq!(fx_f32(0.00000000025), Fx(I32F32::from_num(0.00000000025)));
        assert_eq!(fx_f32(-0.00000000025), Fx(I32F32::from_num(-0.00000000025)));
        assert_eq!(fx_f32(0.625), Fx(I32F32::from_num(0.625)));
        assert_eq!(fx_f32(-0.625), Fx(I32F32::from_num(-0.625)));
        assert_eq!(fx_f32(f32::INFINITY), Fx(I32F32::MAX));
        assert_eq!(fx_f32(f32::NEG_INFINITY), Fx(I32F32::MIN));
    }
}
