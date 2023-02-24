// DvarLimitsXXXXX hold the domain for each possible type of Dvar
// Display is impl'ed to print said domain
// Default is impl'ed where possible, and should always resolve to the largest
// possible domain

// bool has no custom-definable domain, it'll always be 0 or 1/true or false
// DvarLimitsBool still needs to be defined for printing the domain

use core::fmt::Display;
use std::collections::HashSet;

/// Domain for [`Dvar`] with value type [`DvarValue::Bool`]
///
/// Since [`bool`]'s domain of [`true`]/[`false`] is enforeced by the compiler,
/// no custom-defined domain is necessary. However, the struct still needs to
/// exist to impl Display for the domain.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct DvarLimitsBool;

impl Display for DvarLimitsBool {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Domain is 0 or 1")
    }
}

impl DvarLimitsBool {
    /// Creates a new [`DvarLimitsBool`].
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsBool::new();
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub const fn new() -> Self {
        Self {}
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Float`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`f32`] provided
/// `min <= max`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DvarLimitsFloat {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsFloat {
    /// Returns a [`DvarLimitsFloat`] with `min` field set to [`f32::MIN`]
    /// and `max` field set to [`f32::MAX`].
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsFloat {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if (self.min - f32::MIN).abs() < f32::EPSILON {
            if (self.max - f32::MAX).abs() < f32::EPSILON {
                write!(f, "Domain is any number")
            } else {
                write!(f, "Domain is any number {} or smaller", self.max)
            }
        } else if (self.max - f32::MAX).abs() < f32::EPSILON {
            write!(f, "Domain is any number {} or bigger", self.min)
        } else {
            write!(f, "Domain is any number from {} to {}", self.min, self.max)
        }
    }
}

impl DvarLimitsFloat {
    /// Creates a new [`DvarLimitsFloat`] with the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `max`, cannot be greater.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max`. Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsFloat::new(0.0, 10.0);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: f32, max: f32) -> Self {
        // Panic if max is greater than min
        // (possibly implement better error handling in the future)
        if min > max {
            panic!("DvarLimitsFloat::new(): supplied min is greater than max");
        }

        Self { min, max }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Vector2`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`f32`] provided
/// `min <= max`. All elements of the vector share the domain
/// (i.e., the domain cannot be defined on a per-element basis).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DvarLimitsVector2 {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsVector2 {
    /// Returns a [`DvarLimitsVector2`] with `min` field set to [`f32::MIN`]
    /// and `max` field set to [`f32::MAX`].
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsVector2 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if (self.min - f32::MIN).abs() < f32::EPSILON {
            if (self.max - f32::MAX).abs() < f32::EPSILON {
                write!(f, "Domain is any 2D vector")
            } else {
                write!(
                    f,
                    "Domain is any 2D vector with components {} or smaller",
                    self.max
                )
            }
        } else if (self.max - f32::MAX).abs() < f32::EPSILON {
            write!(
                f,
                "Domain is any 2D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 2D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsVector2 {
    /// Creates a new [`DvarLimitsVector2`] with the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `max`, cannot be greater.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max`. Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsVector2::new(0.0, 10.0);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: f32, max: f32) -> Self {
        if min > max {
            panic!(
                "DvarLimitsVector2::new(): supplied min is greater than max"
            );
        }

        Self { min, max }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Vector3`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`f32`] provided
/// `min <= max`. All elements of the vector share the domain
/// (i.e., the domain cannot be defined on a per-element basis).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DvarLimitsVector3 {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsVector3 {
    /// Returns a [`DvarLimitsVector3`] with `min` field set to [`f32::MIN`]
    /// and `max` field set to [`f32::MAX`].
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsVector3 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if (self.min - f32::MIN).abs() < f32::EPSILON {
            if (self.max - f32::MAX).abs() < f32::EPSILON {
                write!(f, "Domain is any 3D vector")
            } else {
                write!(
                    f,
                    "Domain is any 3D vector with components {} or smaller",
                    self.max
                )
            }
        } else if (self.max - f32::MAX).abs() < f32::EPSILON {
            write!(
                f,
                "Domain is any 3D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 3D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsVector3 {
    /// Creates a new [`DvarLimitsVector3`] with the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `max`, cannot be greater.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max`. Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsVector3::new(0.0, 10.0);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: f32, max: f32) -> Self {
        if min > max {
            panic!(
                "DvarLimitsVector3::new(): supplied min is greater than max"
            );
        }

        Self { min, max }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Vector4`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`f32`] provided
/// `min <= max`. All elements of the vector share the domain
/// (i.e., the domain cannot be defined on a per-element basis).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DvarLimitsVector4 {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsVector4 {
    /// Returns a [`DvarLimitsVector4`] with `min` field set to [`f32::MIN`]
    /// and `max` field set to [`f32::MAX`].
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsVector4 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if (self.min - f32::MIN).abs() < f32::EPSILON {
            if (self.max - f32::MAX).abs() < f32::EPSILON {
                write!(f, "Domain is any 4D vector")
            } else {
                write!(
                    f,
                    "Domain is any 4D vector with components {} or smaller",
                    self.max
                )
            }
        } else if (self.max - f32::MAX).abs() < f32::EPSILON {
            write!(
                f,
                "Domain is any 4D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 4D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsVector4 {
    /// Creates a new [`DvarLimitsVector4`] with the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `max`, cannot be greater.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max`. Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsVector4::new(0.0, 10.0);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: f32, max: f32) -> Self {
        if min > max {
            panic!(
                "DvarLimitsVector4::new(): supplied min is greater than max"
            );
        }

        Self { min, max }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Int`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`i32`] provided
/// `min <= max`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DvarLimitsInt {
    pub min: i32,
    pub max: i32,
}

impl Default for DvarLimitsInt {
    /// Returns a [`DvarLimitsInt`] with `min` field set to [`i32::MIN`]
    /// and `max` field set to [`i32::MAX`].
    fn default() -> Self {
        Self {
            min: i32::MIN,
            max: i32::MAX,
        }
    }
}

impl Display for DvarLimitsInt {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.min == i32::MIN {
            if self.max == i32::MAX {
                write!(f, "Domain is any integer")
            } else {
                write!(f, "Domain is any integer {} or smaller", self.max)
            }
        } else if self.max == i32::MAX {
            write!(f, "Domain is any integer {} or bigger", self.min)
        } else {
            write!(f, "Domain is any integer from {} to {}", self.min, self.max)
        }
    }
}

impl DvarLimitsInt {
    /// Creates a new [`DvarLimitsInt`] with the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `max`, cannot be greater.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max`. Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsInt::new(0, 10);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: i32, max: i32) -> Self {
        if min > max {
            panic!("DvarLimitsInt::new(): supplied min is greater than max");
        }

        Self { min, max }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::String`]
///
/// Like with [`bool`], there is no custom definable domain;
/// the compiler already enforces that domain as "valid UTF-8 strings".
/// Also like with [`bool`], the struct still needs to
/// exist to impl [`Display`] for the domain.
///
/// For a [`String`] "bounded" in the sense of "being able to hold
/// one of one or more pre-defined values", use [`DvarValue::Enumeration`]
/// instead.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct DvarLimitsString;

impl Display for DvarLimitsString {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Domain is any text")
    }
}

impl DvarLimitsString {
    /// Creates a new [`DvarLimitsString`].
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsString::new(0, 10);
    /// ```
    pub const fn new() -> Self {
        Self {}
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Enumeration`]
///
/// The domain may consist of one or more different [`String`]s of
/// any value, but it *must* at least contain at least the current
/// value of the [`Dvar`].
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DvarLimitsEnumeration {
    pub strings: HashSet<String>,
}

impl Display for DvarLimitsEnumeration {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Domain is any one of the following:")?;
        for (i, s) in self.strings.iter().enumerate() {
            write!(f, "\n  {:2}: {}", i, s)?;
        }

        core::fmt::Result::Ok(())
    }
}

impl DvarLimitsEnumeration {
    /// Creates a new [`DvarLimitsEnumeration`] with the supplied domain.
    ///
    /// # Parameters
    /// * `domain` - A slice of [`String`]s containing the valid values for
    /// the [`Dvar`]. The [`Dvar`]'s initial value *must* be included in this domain.
    ///
    /// # Panics
    /// Currently will panic if [`domain.is_empty()`]. Might be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsEnumeration::new(&vec![
    ///     "test".to_owned(),
    ///     "test2".to_owned(),
    /// ]);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(domain: &[String]) -> Self {
        if domain.is_empty() {
            panic!("DvarLimitsEnumeration::new(): domain is empty.");
        }

        Self {
            strings: domain.iter().cloned().collect::<HashSet<_>>(),
        }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Color`]
///
/// All RGBA values are valid for [`DvarValue::Color`], so no
/// custom domain is necessary. As with [`bool`] and [`String`],
/// the struct still needs to exist to impl [`Display`] for the domain.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct DvarLimitsColor;

impl Display for DvarLimitsColor {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Domain is any 4-component color, in RGBA format")
    }
}

impl DvarLimitsColor {
    /// Creates a new [`DvarLimitsColor`] with the supplied domain.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsColor::new();
    /// ```
    pub const fn new() -> Self {
        Self {}
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::Int64`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`i64`] provided
/// `min <= max`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DvarLimitsInt64 {
    pub min: i64,
    pub max: i64,
}

impl Default for DvarLimitsInt64 {
    /// Returns a [`DvarLimitsInt64`] with `min` field set to [`i64::MIN`]
    /// and `max` field set to [`i64::MAX`].
    fn default() -> Self {
        Self {
            min: i64::MIN,
            max: i64::MAX,
        }
    }
}

impl Display for DvarLimitsInt64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.min == i64::MIN {
            if self.max == i64::MAX {
                write!(f, "Domain is any integer")
            } else {
                write!(f, "Domain is any integer {} or smaller", self.max)
            }
        } else if self.max == i64::MAX {
            write!(f, "Domain is any integer {} or bigger", self.min)
        } else {
            write!(f, "Domain is any integer from {} to {}", self.min, self.max)
        }
    }
}

impl DvarLimitsInt64 {
    /// Creates a new [`DvarLimitsInt64`] with the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `max`, cannot be greater.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max`. Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsInt64::new(0i64, 10i64);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: i64, max: i64) -> Self {
        if min > max {
            panic!("DvarLimitsInt64::new(): supplied min is greater than max");
        }

        Self { min, max }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::LinearColorRGB`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`f32`] provided
/// `min <= max`. All elements of the vector share the domain
/// (i.e., the domain cannot be defined on a per-element basis).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DvarLimitsLinearColorRGB {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsLinearColorRGB {
    /// Returns a [`DvarLimitsLinearColorRGB`] with `min` field set to [`f32::MIN`]
    /// and `max` field set to [`f32::MAX`].
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsLinearColorRGB {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if (self.min - f32::MIN).abs() < f32::EPSILON {
            if (self.max - f32::MAX).abs() < f32::EPSILON {
                write!(f, "Domain is any 3D vector")
            } else {
                write!(
                    f,
                    "Domain is any 3D vector with components {} or smaller",
                    self.max
                )
            }
        } else if (self.max - f32::MAX).abs() < f32::EPSILON {
            write!(
                f,
                "Domain is any 3D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 3D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsLinearColorRGB {
    /// Creates a new [`DvarLimitsLinearColorRGB`] with
    /// the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// 0.0 <= min <= max <= 1.0 must hold true.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// 0.0 <= min <= max <= 1.0 must hold true.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max` or
    /// if 0 <= min <= max <= 1.0 does not hold true.
    /// Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsLinearColorRGB::new(0.3, 0.7);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: f32, max: f32) -> Self {
        if min < 0.0 || max < 0.0 || min > 1.0 || max > 1.0 || min > max {
            panic!(
                "DvarLimitsLinearColorRGB::new(): \
            supplied min is greater than max,\
            or min and/or max are not within [0.0, 1.0]"
            );
        }

        Self { min, max }
    }
}

/// Domain for [`Dvar`] with value type [`DvarValue::ColorXYZ`]
///
/// The domain is bounded by a custom-defined `min` and `max`,
/// which may be any values representable by [`f32`] provided
/// `min <= max`. All elements of the vector share the domain
/// (i.e., the domain cannot be defined on a per-element basis).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DvarLimitsColorXYZ {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsColorXYZ {
    /// Returns a [`DvarLimitsColorXYZ`] with `min` field set to [`f32::MIN`]
    /// and `max` field set to [`f32::MAX`].
    fn default() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsColorXYZ {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if (self.min - f32::MIN).abs() < f32::EPSILON {
            if (self.max - f32::MAX).abs() < f32::EPSILON {
                write!(f, "Domain is any 3D vector")
            } else {
                write!(
                    f,
                    "Domain is any 3D vector with components {} or smaller",
                    self.max
                )
            }
        } else if (self.max - f32::MAX).abs() < f32::EPSILON {
            write!(
                f,
                "Domain is any 3D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 3D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsColorXYZ {
    /// Creates a new [`DvarLimitsColorXYZ`] with
    /// the supplied `min` and `max`.
    ///
    /// # Parameters
    /// * `min` - The minimum value allowed for the [`Dvar`]'s values.
    /// 0.0 <= min <= max <= 1.0 must hold true.
    /// * `max` - The maximum value allowed for the [`Dvar`]'s values.
    /// 0.0 <= min <= max <= 1.0 must hold true.
    /// Can be equal to `min`, cannot be less.
    ///
    /// # Panics
    /// Currently will panic if `min` > `max` or
    /// if 0 <= min <= max <= 1.0 does not hold true.
    /// Should be changed at some point.
    ///
    /// # Example
    /// ```
    /// let domain = DvarLimitsColorXYZ::new(0.3, 0.7);
    /// ```
    #[allow(clippy::panic, clippy::manual_assert)]
    pub fn new(min: f32, max: f32) -> Self {
        if min < 0.0 || max < 0.0 || min > 1.0 || max > 1.0 || min > max {
            panic!(
                "DvarLimitsLinearColorRGB::new(): \
            supplied min is greater than max,\
            or min and/or max are not within [0.0, 1.0]"
            );
        }

        Self { min, max }
    }
}

// Enum to tie all the DvarLimitsXXXX's together
#[derive(Clone, Debug)]
pub enum DvarLimits {
    Bool(DvarLimitsBool),
    Float(DvarLimitsFloat),
    Vector2(DvarLimitsVector2),
    Vector3(DvarLimitsVector3),
    Vector4(DvarLimitsVector4),
    Int(DvarLimitsInt),
    String(DvarLimitsString),
    Enumeration(DvarLimitsEnumeration),
    Color(DvarLimitsColor),
    Int64(DvarLimitsInt64),
    LinearColorRGB(DvarLimitsLinearColorRGB),
    ColorXYZ(DvarLimitsColorXYZ),
}

// Display should display the domain of the current value
impl Display for DvarLimits {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
            Self::Float(n) => write!(f, "{}", n),
            Self::Vector2(v) => write!(f, "{}", v),
            Self::Vector3(v) => write!(f, "{}", v),
            Self::Vector4(v) => write!(f, "{}", v),
            Self::Int(i) => write!(f, "{}", i),
            Self::String(s) => write!(f, "{}", s),
            Self::Enumeration(e) => write!(f, "{}", e),
            Self::Color(c) => write!(f, "{}", c),
            Self::Int64(i) => write!(f, "{}", i),
            Self::LinearColorRGB(c) => write!(f, "{}", c),
            Self::ColorXYZ(c) => write!(f, "{}", c),
        }
    }
}

impl DvarLimits {
    // A bunch of helper functions to extract the domain
    // Useful if a given Dvar is known to be a specific type
    // Otherwise long match expressions would be required
    pub const fn as_bool_limits(&self) -> Option<DvarLimitsBool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub const fn as_float_limits(&self) -> Option<DvarLimitsFloat> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub const fn as_vector2_limits(&self) -> Option<DvarLimitsVector2> {
        match self {
            Self::Vector2(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_vector3_limits(&self) -> Option<DvarLimitsVector3> {
        match self {
            Self::Vector3(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_vector4_limits(&self) -> Option<DvarLimitsVector4> {
        match self {
            Self::Vector4(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_int_limits(&self) -> Option<DvarLimitsInt> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub const fn as_string_limits(&self) -> Option<DvarLimitsString> {
        match self {
            Self::String(s) => Some(*s),
            _ => None,
        }
    }

    pub fn as_enumeration_limits(&self) -> Option<DvarLimitsEnumeration> {
        match self {
            Self::Enumeration(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub const fn as_color_limits(&self) -> Option<DvarLimitsColor> {
        match self {
            Self::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub const fn as_int64_limits(&self) -> Option<DvarLimitsInt64> {
        match self {
            Self::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub const fn as_linear_color_rgb_limits(
        &self,
    ) -> Option<DvarLimitsLinearColorRGB> {
        match self {
            Self::LinearColorRGB(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_color_xyz_limits(&self) -> Option<DvarLimitsColorXYZ> {
        match self {
            Self::ColorXYZ(v) => Some(*v),
            _ => None,
        }
    }
}
