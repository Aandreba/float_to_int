use core::num::FpCategory;
use crate::FloatExt;

pub trait TryIntoInt<T> {
    fn try_into_int (self) -> Result<T, TryIntoIntError>;
}

pub trait TryFromFloat<T>: Sized {
    fn try_from_float (v: T) -> Result<Self, TryIntoIntError>;
}

macro_rules! impl_try_into_int {
    (@inner $f:ty as $b:literal => $($i:ty),+) => {
        $(
            impl TryIntoInt<$i> for $f {
                fn try_into_int (self) -> Result<$i, TryIntoIntError> {
                    const SIGNED : bool = <$i>::MIN == 0;
                    const MIN : Option<$f> = match !SIGNED || $b > <$i>::BITS - (SIGNED as u32) {
                        true => Some(<$i>::MIN as $f),
                        false => None
                    };
                    const MAX : Option<$f> = match $b > <$i>::BITS - (SIGNED as u32) {
                        true => Some(<$i>::MAX as $f),
                        false => None
                    };
            
                    match self.integer_classify() {
                        Ok(_) => {
                            if match MIN {
                                Some(min) => self < min,
                                _ => false
                            } {
                                return Err(TryIntoIntError::NegOverlow)
                            }
            
                            if match MAX {
                                Some(max) => self > max,
                                _ => false
                            } {
                                return Err(TryIntoIntError::PosOverflow)
                            }
            
                            return Ok(self as $i)
                        },
                        Err(FpCategory::Normal) => return Err(TryIntoIntError::Decimal),
                        Err(e) => return Err(TryIntoIntError::InvalidCategory(e))
                    }
                }
            }
        )+
    };

    ($($f:ty as $b:literal),+) => {
        $(
            impl_try_into_int! {
                @inner $f as $b => u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
            }
        )+
    };
}

#[allow(unused)]
macro_rules! half_impl_try_into_int {
    ($($i:ty),* | $($ui:ty),* | $($si:ty),*) => {
        $(
            #[cfg_attr(docsrs, doc(cfg(feature = "half")))]
            impl TryIntoInt<$i> for ::half::f16 {
                #[inline]
                fn try_into_int (self) -> Result<$i, TryIntoIntError> {
                    const MIN : ::half::f16 = ::half::f16::from_f32_const(<$i>::MIN as f32);
                    const MAX : ::half::f16 = ::half::f16::from_f32_const(<$i>::MAX as f32);
            
                    match self.integer_classify() {
                        Ok(_) => {
                            if self < MIN {
                                return Err(TryIntoIntError::NegOverlow)
                            }
            
                            if self > MAX {
                                return Err(TryIntoIntError::PosOverflow)
                            }
            
                            return Ok(num_traits::cast::AsPrimitive::<$i>::as_(self))
                        },
                        Err(FpCategory::Normal) => return Err(TryIntoIntError::Decimal),
                        Err(e) => return Err(TryIntoIntError::InvalidCategory(e))
                    }
                }
            }
        )*

        $(
            #[cfg_attr(docsrs, doc(cfg(feature = "half")))]
            impl TryIntoInt<$ui> for ::half::f16 {
                #[inline]
                fn try_into_int (self) -> Result<$ui, TryIntoIntError> {
                    match self.integer_classify() {
                        Ok(_) => {
                            if self < ::half::f16::ZERO {
                                return Err(TryIntoIntError::NegOverlow)
                            }
            
                            return Ok(num_traits::cast::AsPrimitive::<$ui>::as_(self))
                        },
                        Err(FpCategory::Normal) => return Err(TryIntoIntError::Decimal),
                        Err(e) => return Err(TryIntoIntError::InvalidCategory(e))
                    }
                }
            }
        )*

        $(
            #[cfg_attr(docsrs, doc(cfg(feature = "half")))]
            impl TryIntoInt<$si> for ::half::f16 {
                #[inline]
                fn try_into_int (self) -> Result<$si, TryIntoIntError> {
                    match self.integer_classify() {
                        Ok(_) => Ok(num_traits::cast::AsPrimitive::<$si>::as_(self)),
                        Err(FpCategory::Normal) => return Err(TryIntoIntError::Decimal),
                        Err(e) => return Err(TryIntoIntError::InvalidCategory(e))
                    }
                }
            }
        )*
    }
}

impl_try_into_int! {
    f32 as 32,
    f64 as 64
}

#[cfg(feature = "half")]
half_impl_try_into_int! {
    u8, i8, i16 | u16, u32, u64 | i32, i64
}

#[cfg(feature = "half")]
cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "8")] {
        half_impl_try_into_int! {
            usize, isize
        }
    } else if #[cfg(target_pointer_width = "16")] {
        half_impl_try_into_int! {
            isize | usize
        }
    } else {
        half_impl_try_into_int! {
            | usize | isize
        }
    }
}

#[cfg(feature = "half")]
#[cfg_attr(docsrs, doc(cfg(feature = "half")))]
impl TryIntoInt<u128> for ::half::f16 {
    #[inline]
    fn try_into_int (self) -> Result<u128, TryIntoIntError> {
        match self.integer_classify() {
            Ok(_) => {
                if self < ::half::f16::ZERO {
                    return Err(TryIntoIntError::NegOverlow)
                }

                return Ok(num_traits::cast::AsPrimitive::<u64>::as_(self) as u128)
            },
            Err(FpCategory::Normal) => return Err(TryIntoIntError::Decimal),
            Err(e) => return Err(TryIntoIntError::InvalidCategory(e))
        }
    }
}

#[cfg(feature = "half")]
#[cfg_attr(docsrs, doc(cfg(feature = "half")))]
impl TryIntoInt<i128> for ::half::f16 {
    #[inline]
    fn try_into_int (self) -> Result<i128, TryIntoIntError> {
        match self.integer_classify() {
            Ok(_) => Ok(num_traits::cast::AsPrimitive::<i64>::as_(self) as i128),
            Err(FpCategory::Normal) => return Err(TryIntoIntError::Decimal),
            Err(e) => return Err(TryIntoIntError::InvalidCategory(e))
        }
    }
}

impl<I, F: TryIntoInt<I>> TryFromFloat<F> for I {
    #[inline(always)]
    fn try_from_float (v: F) -> Result<Self, TryIntoIntError> {
        F::try_into_int(v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryIntoIntError {
    /// Value has a decimal part.
    Decimal,
    /// Value is too large to store in target integer type.
    PosOverflow,
    /// Value is too small to store in target integer type.
    NegOverlow,
    /// Value was Zero.
    /// 
    /// This variant will be emitted when the value passed has a value of zero, 
    /// which would be illegal for non-zero types.
    Zero,
    /// Value has a non-compatible floating-point category.
    InvalidCategory (FpCategory)
}

#[cfg(feature = "half")]
#[test]
fn test () {
    let test = i8::try_from_float(::half::f16::LN_2);
    panic!("{test:?}");
}