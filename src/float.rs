use core::num::FpCategory;

pub trait FloatExt {
    /// # Example
    /// ```rust
    /// assert_eq!(1f32.integer_classify(), Ok(()));
    /// assert!(!3.14f64.integer_classify(), Err(FpCategory::Normal));
    /// ```
    fn integer_classify (self) -> Result<(), FpCategory>;

    /// Checks if the value contained inside the float is an integer.
    /// 
    /// # Example
    /// ```rust
    /// assert!(1f32.is_integer());
    /// assert!(!3.14f64.is_integer());
    /// ```
    #[inline(always)]
    fn is_integer (self) -> bool where Self: Sized {
        return self.integer_classify().is_ok()
    }
}

macro_rules! impl_float {
    ($($(#[$meta:meta])* $t:ty as $bits:ty),+) => {
        $(
            $(#[$meta])*
            impl FloatExt for $t {
                fn integer_classify (self) -> Result<(), FpCategory> {
                    const MANTISSA_M1 : $bits = (<$t>::MANTISSA_DIGITS - 1) as $bits;
                    const EXP_MASK : $bits = (1 << ((<$bits>::BITS as $bits) - MANTISSA_M1 - 1)) - 1;
                    const DELTA : $bits = EXP_MASK >> 1;

                    cfg_if::cfg_if! {
                        if #[cfg(all(not(debug_assertions), feature = "nighlty"))] {
                            #[inline(always)]
                            pub const unsafe fn unchecked_sub (this: $bits, other: $bits) -> $bits {
                                return this.unchecked_sub(other)
                            }
            
                            #[inline(always)]
                            pub const unsafe fn unchecked_shl (this: $bits, other: $bits) -> $bits {
                                return this.unchecked_shl(other)
                            }
            
                            #[inline(always)]
                            pub const unsafe fn unchecked_shr (this: $bits, other: $bits) -> $bits {
                                return this.unchecked_shr(other)
                            }
                        } else {
                            #[inline(always)]
                            pub const unsafe fn unchecked_sub (this: $bits, other: $bits) -> $bits {
                                return this - other
                            }
            
                            #[inline(always)]
                            pub const unsafe fn unchecked_shl (this: $bits, other: $bits) -> $bits {
                                return this << other
                            }
            
                            #[inline(always)]
                            pub const unsafe fn unchecked_shr (this: $bits, other: $bits) -> $bits {
                                return this >> other
                            }
                        }
                    }
            
                    match self.classify() {
                        FpCategory::Zero => return Ok(()),
                        FpCategory::Normal => unsafe {
                            let bits = <$t>::to_bits(self);
            
                            if let Some(exp) = (unchecked_shr(bits, MANTISSA_M1) & EXP_MASK).checked_sub(DELTA) {
                                match exp {
                                    0 => return match bits & MANTISSA_M1 == 0 {
                                        true => Ok(()),
                                        false => Err(FpCategory::Normal)
                                    },
                                    MANTISSA_M1.. => return Ok(()),
                                    exp => {
                                        let mask = unchecked_sub(
                                            unchecked_shl(1, exp),
                                            1
                                        );
                            
                                        let v = unchecked_shr(
                                            bits,
                                            unchecked_sub(MANTISSA_M1, exp)
                                        ) & mask;
                        
                                        return match v <= mask {
                                            true => Ok(()),
                                            false => Err(FpCategory::Normal)
                                        }
                                    }
                                }
                            };

                            return Err(FpCategory::Normal);
                        },
                        other => return Err(other)
                    }
                }
            }
        )+
    };
}

impl_float! {
    #[cfg(feature = "half")]
    #[cfg_attr(docsrs, doc(cfg(feature = "half")))]
    ::half::f16 as u16,
    f32 as u32,
    f64 as u64
}

#[cfg(test)]
mod tests {
    use rand::random;
    use crate::FloatExt;
    
    #[cfg(feature = "half")]
    #[test]
    fn test_f16 () {
        use half::f16;
        use num_traits::float::FloatCore;

        assert!(!f16::NAN.is_integer());
        assert!(!f16::INFINITY.is_integer());
        assert!(!f16::NEG_INFINITY.is_integer());
        assert!(f16::ZERO.is_integer());
        assert!(f16::ONE.is_integer());
        assert!(f16::NEG_ONE.is_integer());

        for _ in 0..=50_000 {
            let v = f16::from_f32(2f32 * random::<f32>() - 1f32);
            assert_eq!(v.is_integer(), FloatCore::round(v) == v);
        }
    }

    #[test]
    fn test_f32 () {
        assert!(!f32::NAN.is_integer());
        assert!(!f32::INFINITY.is_integer());
        assert!(!f32::NEG_INFINITY.is_integer());
        assert!(0f32.is_integer());
        assert!(1f32.is_integer());
        assert!((-1f32).is_integer());

        for _ in 0..=50_000 {
            let v = 2f32 * random::<f32>() - 1f32;
            assert_eq!(v.is_integer(), v.round() == v);
        }
    }

    #[test]
    fn test_f64 () {
        assert!(!f64::NAN.is_integer());
        assert!(!f64::INFINITY.is_integer());
        assert!(!f64::NEG_INFINITY.is_integer());
        assert!(0f64.is_integer());
        assert!(1f64.is_integer());
        assert!((-1f64).is_integer());

        for _ in 0..=50_000 {
            let v = 2f64 * random::<f64>() - 1f64;
            assert_eq!(v.is_integer(), v.round() == v);
        }
    }
}
