use std::marker::PhantomData;

use crate::common::{Vec2f32, Vec3f32, Vec4f32};

use super::{
    limits::{
        DvarLimits, DvarLimitsBool, DvarLimitsColor, DvarLimitsColorXYZ,
        DvarLimitsEnumeration, DvarLimitsFloat, DvarLimitsInt, DvarLimitsInt64,
        DvarLimitsLinearColorRGB, DvarLimitsString, DvarLimitsVector2,
        DvarLimitsVector3, DvarLimitsVector4,
    },
    value::DvarValue,
    Dvar, DvarFlags,
};

macro_rules! zero_variant_enum {
    ($e:ident) => {
        #[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
        pub enum $e {
            #[default]
            __,
        }
    };
}

// Enums for [`DvarBuilder`]'s various typestates.
zero_variant_enum!(DvarBuilderStartState);
zero_variant_enum!(DvarBuilderDataState);
zero_variant_enum!(DvarBuilderTypeState);
zero_variant_enum!(DvarBuilderTypeBoolCurrentValueState);
zero_variant_enum!(DvarBuilderTypeBoolOtherValuesState);
zero_variant_enum!(DvarBuilderTypeFloatDomainState);
zero_variant_enum!(DvarBuilderTypeFloatCurrentValueState);
zero_variant_enum!(DvarBuilderTypeFloatOtherValuesState);
zero_variant_enum!(DvarBuilderTypeVector2DomainState);
zero_variant_enum!(DvarBuilderTypeVector2CurrentValueState);
zero_variant_enum!(DvarBuilderTypeVector2OtherValuesState);
zero_variant_enum!(DvarBuilderTypeVector3DomainState);
zero_variant_enum!(DvarBuilderTypeVector3CurrentValueState);
zero_variant_enum!(DvarBuilderTypeVector3OtherValuesState);
zero_variant_enum!(DvarBuilderTypeVector4DomainState);
zero_variant_enum!(DvarBuilderTypeVector4CurrentValueState);
zero_variant_enum!(DvarBuilderTypeVector4OtherValuesState);
zero_variant_enum!(DvarBuilderTypeIntDomainState);
zero_variant_enum!(DvarBuilderTypeIntCurrentValueState);
zero_variant_enum!(DvarBuilderTypeIntOtherValuesState);
zero_variant_enum!(DvarBuilderTypeStringCurrentValueState);
zero_variant_enum!(DvarBuilderTypeStringOtherValuesState);
zero_variant_enum!(DvarBuilderTypeEnumerationDomainState);
zero_variant_enum!(DvarBuilderTypeEnumerationCurrentValueState);
zero_variant_enum!(DvarBuilderTypeEnumerationOtherValuesState);
zero_variant_enum!(DvarBuilderTypeColorCurrentValueState);
zero_variant_enum!(DvarBuilderTypeColorOtherValuesState);
zero_variant_enum!(DvarBuilderTypeInt64DomainState);
zero_variant_enum!(DvarBuilderTypeInt64CurrentValueState);
zero_variant_enum!(DvarBuilderTypeInt64OtherValuesState);
zero_variant_enum!(DvarBuilderTypeLinearColorRGBDomainState);
zero_variant_enum!(DvarBuilderTypeLinearColorRGBCurrentValueState);
zero_variant_enum!(DvarBuilderTypeLinearColorRGBOtherValuesState);
zero_variant_enum!(DvarBuilderTypeColorXYZDomainState);
zero_variant_enum!(DvarBuilderTypeColorXYZCurrentValueState);
zero_variant_enum!(DvarBuilderTypeColorXYZOtherValuesState);

#[derive(Clone)]
struct DvarInProgress {
    pub name: Option<String>,
    pub description: Option<String>,
    pub flags: Option<DvarFlags>,
    pub modified: Option<bool>,
    pub loaded_from_save_game: Option<bool>,
    pub domain: Option<DvarLimits>,
    pub current: Option<DvarValue>,
    latched: Option<DvarValue>,
    reset: Option<DvarValue>,
    saved: Option<DvarValue>,
}

macro_rules! unwrap_or_return {
    ($i:expr, $r:expr) => {
        match $i {
            Some(i) => i,
            None => return $r,
        }
    };
}

impl TryFrom<DvarInProgress> for Dvar {
    type Error = ();
    fn try_from(value: DvarInProgress) -> Result<Self, Self::Error> {
        let name = value.name.unwrap_or_default();
        let description = value.description.unwrap_or_default();
        let flags = value.flags.unwrap_or_default();
        let modified = value.modified.unwrap_or_default();
        let loaded_from_save_game =
            value.loaded_from_save_game.unwrap_or_default();
        let domain = unwrap_or_return!(value.domain, Err(()));
        let current = unwrap_or_return!(value.current, Err(()));
        let latched = unwrap_or_return!(value.latched, Err(()));
        let reset = unwrap_or_return!(value.reset, Err(()));
        let saved = unwrap_or_return!(value.saved, Err(()));

        Ok(Self {
            name,
            description,
            flags,
            modified,
            loaded_from_save_game,
            domain,
            current,
            latched,
            reset,
            saved,
        })
    }
}

// Helper impl to make constructing Dvars easier
pub struct DvarBuilder<T> {
    dvar: DvarInProgress,
    extra: PhantomData<T>,
}

impl DvarBuilder<DvarBuilderStartState> {
    pub(super) fn new() -> DvarBuilder<DvarBuilderDataState> {
        DvarBuilder::<DvarBuilderDataState> {
            dvar: DvarInProgress {
                name: None,
                description: None,
                flags: None,
                modified: None,
                loaded_from_save_game: None,
                domain: None,
                current: None,
                latched: None,
                reset: None,
                saved: None,
            },
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderDataState> {
    pub(super) fn name(
        mut self,
        name: &str,
    ) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.name = name.to_string().into();
        self
    }

    pub(super) fn description(
        mut self,
        description: String,
    ) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.description = description.into();
        self
    }

    pub(super) fn flags(
        mut self,
        flags: DvarFlags,
    ) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.flags = flags.into();
        self
    }

    pub(super) fn loaded_from_save_game(
        mut self,
        b: bool,
    ) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.loaded_from_save_game = b.into();
        self
    }

    pub(super) fn type_bool(
        self,
    ) -> DvarBuilder<DvarBuilderTypeBoolCurrentValueState> {
        DvarBuilder::<DvarBuilderTypeBoolCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_float(
        self,
    ) -> DvarBuilder<DvarBuilderTypeFloatDomainState> {
        DvarBuilder::<DvarBuilderTypeFloatDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_vector2(
        self,
    ) -> DvarBuilder<DvarBuilderTypeVector2DomainState> {
        DvarBuilder::<DvarBuilderTypeVector2DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_vector3(
        self,
    ) -> DvarBuilder<DvarBuilderTypeVector3DomainState> {
        DvarBuilder::<DvarBuilderTypeVector3DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_vector4(
        self,
    ) -> DvarBuilder<DvarBuilderTypeVector4DomainState> {
        DvarBuilder::<DvarBuilderTypeVector4DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_int(self) -> DvarBuilder<DvarBuilderTypeIntDomainState> {
        DvarBuilder::<DvarBuilderTypeIntDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_string(
        self,
    ) -> DvarBuilder<DvarBuilderTypeStringCurrentValueState> {
        DvarBuilder::<DvarBuilderTypeStringCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_enumeration(
        self,
    ) -> DvarBuilder<DvarBuilderTypeEnumerationDomainState> {
        DvarBuilder::<DvarBuilderTypeEnumerationDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_color(
        self,
    ) -> DvarBuilder<DvarBuilderTypeColorCurrentValueState> {
        DvarBuilder::<DvarBuilderTypeColorCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_int64(
        self,
    ) -> DvarBuilder<DvarBuilderTypeInt64DomainState> {
        DvarBuilder::<DvarBuilderTypeInt64DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_linear_color_rgb(
        self,
    ) -> DvarBuilder<DvarBuilderTypeLinearColorRGBDomainState> {
        DvarBuilder::<DvarBuilderTypeLinearColorRGBDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub(super) fn type_color_xyz(
        self,
    ) -> DvarBuilder<DvarBuilderTypeColorXYZDomainState> {
        DvarBuilder::<DvarBuilderTypeColorXYZDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeFloatDomainState> {
    pub(super) fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeFloatCurrentValueState> {
        self.dvar.domain =
            DvarLimits::Float(DvarLimitsFloat::new(min, max)).into();
        DvarBuilder::<DvarBuilderTypeFloatCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector2DomainState> {
    pub(super) fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeVector2CurrentValueState> {
        self.dvar.domain =
            DvarLimits::Vector2(DvarLimitsVector2::new(min, max)).into();
        DvarBuilder::<DvarBuilderTypeVector2CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector3DomainState> {
    pub(super) fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeVector3CurrentValueState> {
        self.dvar.domain =
            DvarLimits::Vector3(DvarLimitsVector3::new(min, max)).into();
        DvarBuilder::<DvarBuilderTypeVector3CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector4DomainState> {
    pub(super) fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeVector4CurrentValueState> {
        self.dvar.domain =
            DvarLimits::Vector4(DvarLimitsVector4::new(min, max)).into();
        DvarBuilder::<DvarBuilderTypeVector4CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeIntDomainState> {
    pub(super) fn domain(
        mut self,
        min: i32,
        max: i32,
    ) -> DvarBuilder<DvarBuilderTypeIntCurrentValueState> {
        self.dvar.domain = DvarLimits::Int(DvarLimitsInt::new(min, max)).into();
        DvarBuilder::<DvarBuilderTypeIntCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeEnumerationDomainState> {
    pub(super) fn domain(
        mut self,
        domain: Vec<String>,
    ) -> DvarBuilder<DvarBuilderTypeEnumerationCurrentValueState> {
        self.dvar.domain =
            DvarLimits::Enumeration(DvarLimitsEnumeration::new(&domain)).into();
        DvarBuilder::<DvarBuilderTypeEnumerationCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeInt64DomainState> {
    pub(super) fn domain(
        mut self,
        min: i64,
        max: i64,
    ) -> DvarBuilder<DvarBuilderTypeInt64CurrentValueState> {
        self.dvar.domain =
            DvarLimits::Int64(DvarLimitsInt64::new(min, max)).into();
        DvarBuilder::<DvarBuilderTypeInt64CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeLinearColorRGBDomainState> {
    pub(super) fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeLinearColorRGBCurrentValueState> {
        self.dvar.domain =
            DvarLimits::LinearColorRGB(DvarLimitsLinearColorRGB::new(min, max))
                .into();
        DvarBuilder::<DvarBuilderTypeLinearColorRGBCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeColorXYZDomainState> {
    pub(super) fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeColorXYZCurrentValueState> {
        self.dvar.domain =
            DvarLimits::ColorXYZ(DvarLimitsColorXYZ::new(min, max)).into();
        DvarBuilder::<DvarBuilderTypeColorXYZCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeBoolCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: bool,
    ) -> DvarBuilder<DvarBuilderTypeBoolOtherValuesState> {
        self.dvar.domain = DvarLimits::Bool(DvarLimitsBool::new()).into();
        self.dvar.current = DvarValue::Bool(value).into();
        self.dvar.latched = DvarValue::Bool(value).into();
        self.dvar.saved = DvarValue::Bool(value).into();
        self.dvar.reset = DvarValue::Bool(value).into();
        DvarBuilder::<DvarBuilderTypeBoolOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeFloatCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: f32,
    ) -> DvarBuilder<DvarBuilderTypeFloatOtherValuesState> {
        self.dvar.current = DvarValue::Float(value).into();
        self.dvar.latched = DvarValue::Float(value).into();
        self.dvar.saved = DvarValue::Float(value).into();
        self.dvar.reset = DvarValue::Float(value).into();
        DvarBuilder::<DvarBuilderTypeFloatOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector2CurrentValueState> {
    pub(super) fn value(
        mut self,
        value: Vec2f32,
    ) -> DvarBuilder<DvarBuilderTypeVector2OtherValuesState> {
        self.dvar.current = DvarValue::Vector2(value).into();
        self.dvar.latched = DvarValue::Vector2(value).into();
        self.dvar.saved = DvarValue::Vector2(value).into();
        self.dvar.reset = DvarValue::Vector2(value).into();
        DvarBuilder::<DvarBuilderTypeVector2OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector3CurrentValueState> {
    pub(super) fn value(
        mut self,
        value: Vec3f32,
    ) -> DvarBuilder<DvarBuilderTypeVector3OtherValuesState> {
        self.dvar.current = DvarValue::Vector3(value).into();
        self.dvar.latched = DvarValue::Vector3(value).into();
        self.dvar.saved = DvarValue::Vector3(value).into();
        self.dvar.reset = DvarValue::Vector3(value).into();
        DvarBuilder::<DvarBuilderTypeVector3OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector4CurrentValueState> {
    pub(super) fn value(
        mut self,
        value: Vec4f32,
    ) -> DvarBuilder<DvarBuilderTypeVector4OtherValuesState> {
        self.dvar.current = DvarValue::Vector4(value).into();
        self.dvar.latched = DvarValue::Vector4(value).into();
        self.dvar.saved = DvarValue::Vector4(value).into();
        self.dvar.reset = DvarValue::Vector4(value).into();
        DvarBuilder::<DvarBuilderTypeVector4OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeIntCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: i32,
    ) -> DvarBuilder<DvarBuilderTypeIntOtherValuesState> {
        self.dvar.current = DvarValue::Int(value).into();
        self.dvar.latched = DvarValue::Int(value).into();
        self.dvar.saved = DvarValue::Int(value).into();
        self.dvar.reset = DvarValue::Int(value).into();
        DvarBuilder::<DvarBuilderTypeIntOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeStringCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: String,
    ) -> DvarBuilder<DvarBuilderTypeStringOtherValuesState> {
        self.dvar.domain = DvarLimits::String(DvarLimitsString::new()).into();
        self.dvar.current = DvarValue::String(value.clone()).into();
        self.dvar.latched = DvarValue::String(value.clone()).into();
        self.dvar.saved = DvarValue::String(value.clone()).into();
        self.dvar.reset = DvarValue::String(value).into();
        DvarBuilder::<DvarBuilderTypeStringOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeEnumerationCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: String,
    ) -> DvarBuilder<DvarBuilderTypeEnumerationOtherValuesState> {
        self.dvar.current = DvarValue::Enumeration(value.clone()).into();
        self.dvar.latched = DvarValue::Enumeration(value.clone()).into();
        self.dvar.saved = DvarValue::Enumeration(value.clone()).into();
        self.dvar.reset = DvarValue::Enumeration(value).into();
        DvarBuilder::<DvarBuilderTypeEnumerationOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeColorCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: Vec4f32,
    ) -> DvarBuilder<DvarBuilderTypeColorOtherValuesState> {
        self.dvar.domain = DvarLimits::Color(DvarLimitsColor::new()).into();
        self.dvar.current = DvarValue::Color(value).into();
        self.dvar.latched = DvarValue::Color(value).into();
        self.dvar.saved = DvarValue::Color(value).into();
        self.dvar.reset = DvarValue::Color(value).into();
        DvarBuilder::<DvarBuilderTypeColorOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeInt64CurrentValueState> {
    pub(super) fn value(
        mut self,
        value: i64,
    ) -> DvarBuilder<DvarBuilderTypeInt64OtherValuesState> {
        self.dvar.current = DvarValue::Int64(value).into();
        self.dvar.latched = DvarValue::Int64(value).into();
        self.dvar.saved = DvarValue::Int64(value).into();
        self.dvar.reset = DvarValue::Int64(value).into();
        DvarBuilder::<DvarBuilderTypeInt64OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeLinearColorRGBCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: Vec3f32,
    ) -> DvarBuilder<DvarBuilderTypeLinearColorRGBOtherValuesState> {
        self.dvar.current = DvarValue::LinearColorRGB(value).into();
        self.dvar.latched = DvarValue::LinearColorRGB(value).into();
        self.dvar.saved = DvarValue::LinearColorRGB(value).into();
        self.dvar.reset = DvarValue::LinearColorRGB(value).into();
        DvarBuilder::<DvarBuilderTypeLinearColorRGBOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeColorXYZCurrentValueState> {
    pub(super) fn value(
        mut self,
        value: Vec3f32,
    ) -> DvarBuilder<DvarBuilderTypeColorXYZOtherValuesState> {
        self.dvar.current = DvarValue::ColorXYZ(value).into();
        self.dvar.latched = DvarValue::ColorXYZ(value).into();
        self.dvar.saved = DvarValue::ColorXYZ(value).into();
        self.dvar.reset = DvarValue::ColorXYZ(value).into();
        DvarBuilder::<DvarBuilderTypeColorXYZOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeBoolOtherValuesState> {
    pub(super) fn latched(mut self, value: bool) -> Self {
        self.dvar.latched = DvarValue::Bool(value).into();
        self
    }

    pub(super) fn saved(mut self, value: bool) -> Self {
        self.dvar.saved = DvarValue::Bool(value).into();
        self
    }

    pub(super) fn reset(mut self, value: bool) -> Self {
        self.dvar.reset = DvarValue::Bool(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeFloatOtherValuesState> {
    pub(super) fn latched(mut self, value: f32) -> Self {
        self.dvar.latched = DvarValue::Float(value).into();
        self
    }

    pub(super) fn saved(mut self, value: f32) -> Self {
        self.dvar.saved = DvarValue::Float(value).into();
        self
    }

    pub(super) fn reset(mut self, value: f32) -> Self {
        self.dvar.reset = DvarValue::Float(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeVector2OtherValuesState> {
    pub(super) fn latched(mut self, value: Vec2f32) -> Self {
        self.dvar.latched = DvarValue::Vector2(value).into();
        self
    }

    pub(super) fn saved(mut self, value: Vec2f32) -> Self {
        self.dvar.saved = DvarValue::Vector2(value).into();
        self
    }

    pub(super) fn reset(mut self, value: Vec2f32) -> Self {
        self.dvar.reset = DvarValue::Vector2(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeVector3OtherValuesState> {
    pub(super) fn latched(mut self, value: Vec3f32) -> Self {
        self.dvar.latched = DvarValue::Vector3(value).into();
        self
    }

    pub(super) fn saved(mut self, value: Vec3f32) -> Self {
        self.dvar.saved = DvarValue::Vector3(value).into();
        self
    }

    pub(super) fn reset(mut self, value: Vec3f32) -> Self {
        self.dvar.reset = DvarValue::Vector3(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeVector4OtherValuesState> {
    pub(super) fn latched(mut self, value: Vec4f32) -> Self {
        self.dvar.latched = DvarValue::Vector4(value).into();
        self
    }

    pub(super) fn saved(mut self, value: Vec4f32) -> Self {
        self.dvar.saved = DvarValue::Vector4(value).into();
        self
    }

    pub(super) fn reset(mut self, value: Vec4f32) -> Self {
        self.dvar.reset = DvarValue::Vector4(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeIntOtherValuesState> {
    pub(super) fn latched(mut self, value: i32) -> Self {
        self.dvar.latched = DvarValue::Int(value).into();
        self
    }

    pub(super) fn saved(mut self, value: i32) -> Self {
        self.dvar.saved = DvarValue::Int(value).into();
        self
    }

    pub(super) fn reset(mut self, value: i32) -> Self {
        self.dvar.reset = DvarValue::Int(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeStringOtherValuesState> {
    pub(super) fn latched(mut self, value: String) -> Self {
        self.dvar.latched = DvarValue::String(value).into();
        self
    }

    pub(super) fn saved(mut self, value: String) -> Self {
        self.dvar.saved = DvarValue::String(value).into();
        self
    }

    pub(super) fn reset(mut self, value: String) -> Self {
        self.dvar.reset = DvarValue::String(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeEnumerationOtherValuesState> {
    pub(super) fn latched(mut self, value: String) -> Self {
        self.dvar.latched = DvarValue::Enumeration(value).into();
        self
    }

    pub(super) fn saved(mut self, value: String) -> Self {
        self.dvar.saved = DvarValue::Enumeration(value).into();
        self
    }

    pub(super) fn reset(mut self, value: String) -> Self {
        self.dvar.reset = DvarValue::Enumeration(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeColorOtherValuesState> {
    pub(super) fn latched(mut self, value: Vec4f32) -> Self {
        self.dvar.latched = DvarValue::Color(value).into();
        self
    }

    pub(super) fn saved(mut self, value: Vec4f32) -> Self {
        self.dvar.saved = DvarValue::Color(value).into();
        self
    }

    pub(super) fn reset(mut self, value: Vec4f32) -> Self {
        self.dvar.reset = DvarValue::Color(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeInt64OtherValuesState> {
    pub(super) fn latched(mut self, value: i64) -> Self {
        self.dvar.latched = DvarValue::Int64(value).into();
        self
    }

    pub(super) fn saved(mut self, value: i64) -> Self {
        self.dvar.saved = DvarValue::Int64(value).into();
        self
    }

    pub(super) fn reset(mut self, value: i64) -> Self {
        self.dvar.reset = DvarValue::Int64(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeLinearColorRGBOtherValuesState> {
    pub(super) fn latched(mut self, value: Vec3f32) -> Self {
        self.dvar.latched = DvarValue::LinearColorRGB(value).into();
        self
    }

    pub(super) fn saved(mut self, value: Vec3f32) -> Self {
        self.dvar.saved = DvarValue::LinearColorRGB(value).into();
        self
    }

    pub(super) fn reset(mut self, value: Vec3f32) -> Self {
        self.dvar.reset = DvarValue::LinearColorRGB(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}

impl DvarBuilder<DvarBuilderTypeColorXYZOtherValuesState> {
    pub(super) fn latched(mut self, value: Vec3f32) -> Self {
        self.dvar.latched = DvarValue::ColorXYZ(value).into();
        self
    }

    pub(super) fn saved(mut self, value: Vec3f32) -> Self {
        self.dvar.saved = DvarValue::ColorXYZ(value).into();
        self
    }

    pub(super) fn reset(mut self, value: Vec3f32) -> Self {
        self.dvar.reset = DvarValue::ColorXYZ(value).into();
        self
    }

    pub(super) fn build(self) -> Dvar {
        self.dvar.try_into().unwrap()
    }
}
