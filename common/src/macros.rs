/// Implement PluginParameters for `$raw_parameters`. `$parameter_type` must
/// be an enum which implements `TryFrom<i32>` and `Display`
/// `$raw_parameters` must implement the following functions
/// get(&self, $parameter_type) -> f32
///     returns the normalized f32 value of the given parameter
/// set(&mut self, $parameter_type, value: f32)
///     sets the normalized f32 value of the given parameter
/// get_strings(&self, $parameter_type) -> (String, String)
///     returns a tuple where the first String is the parameter's name
///     (ex: "Master Volume") and the second tuple is the parameter's value
///     (ex: "12 db")
#[macro_export]
macro_rules! impl_plugin_parameters {
    ($raw_parameters: ident, $parameter_type: ident) => {
        impl vst::plugin::PluginParameters for $raw_parameters {
            fn get_parameter_label(&self, index: i32) -> String {
                use std::convert::TryFrom;
                if let Ok(parameter) = $parameter_type::try_from(index) {
                    self.get_strings(parameter).1
                } else {
                    "".to_string()
                }
            }

            fn get_parameter_text(&self, index: i32) -> String {
                use std::convert::TryFrom;
                if let Ok(parameter) = $parameter_type::try_from(index) {
                    self.get_strings(parameter).0
                } else {
                    "".to_string()
                }
            }

            fn get_parameter_name(&self, index: i32) -> String {
                use std::convert::TryFrom;
                if let Ok(param) = $parameter_type::try_from(index) {
                    param.to_string()
                } else {
                    "".to_string()
                }
            }

            fn get_parameter(&self, index: i32) -> f32 {
                use std::convert::TryFrom;
                if let Ok(parameter) = $parameter_type::try_from(index) {
                    self.get(parameter)
                } else {
                    0.0
                }
            }

            fn set_parameter(&self, index: i32, value: f32) {
                use std::convert::TryFrom;
                if let Ok(parameter) = $parameter_type::try_from(index) {
                    // This is needed because some VST hosts, such as Ableton, echo a
                    // parameter change back to the plugin. This causes issues such as
                    // weird knob behavior where the knob "flickers" because the user tries
                    // to change the knob value, but ableton keeps sending back old, echoed
                    // values.
                    #[allow(clippy::float_cmp)]
                    if self.get(parameter) == value {
                        return;
                    }

                    self.set(value, parameter);
                }
            }

            fn can_be_automated(&self, index: i32) -> bool {
                use std::convert::TryFrom;
                $parameter_type::try_from(index).is_ok()
            }

            fn string_to_parameter(&self, _index: i32, _text: String) -> bool {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! impl_get_set {
    ($raw_parameters: ident, $parameter_type: ident) => {
        impl $raw_parameters {
            pub fn set(&self, value: f32, parameter: $parameter_type) {
                // These are needed so Ableton will notice parameter changes in the
                // "Configure" window.
                // TODO: investigate if I should send this only on mouseup/mousedown
                self.host.begin_edit(parameter.into());
                self.get_ref(parameter).set(value);
                self.host.end_edit(parameter.into());
            }

            pub fn get(&self, parameter: $parameter_type) -> f32 {
                self.get_ref(parameter).get()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_display {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        impl std::fmt::Display for $parameter_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($parameter_type::$variant => write!(f, $name),)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_i32 {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        impl std::convert::TryFrom<i32> for $parameter_type {
            type Error = ();
            fn try_from(x: i32) -> Result<Self, Self::Error> {
                match x {
                    $($idx => Ok($parameter_type::$variant),)*
                    _ => Err(()),
                }
            }
        }
    }
}

#[macro_export]
macro_rules! impl_into_i32 {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        impl std::convert::From<$parameter_type> for i32 {
            fn from(x: $parameter_type) -> i32 {
                match x {
                    $($parameter_type::$variant => $idx,)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_get_ref {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        impl $raw_parameters {
            fn get_ref(&self, x: $parameter_type) -> &vst::util::AtomicFloat {
                match x {
                    $($parameter_type::$variant => &self.$field_name,)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_get_default {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        impl $raw_parameters {
            fn get_default(x: $parameter_type) -> f32 {
                match x {
                    $($parameter_type::$variant => $default,)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_default {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        impl $raw_parameters {
            fn default(host: vst::plugin::HostCallback) -> Self {
                $raw_parameters {
                    $($field_name: vst::util::AtomicFloat::new($default),)*
                    host,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_get_strings {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        impl $raw_parameters {
            /// Returns a user-facing text output for the given parameter. This is broken
            /// into a tuple consisting of (`value`, `units`)
            fn get_strings(&self, parameter: $parameter_type) -> (String, String) {
                let params = Parameters::from(self);
                match parameter {
                    $($parameter_type::$variant => $string(params.$field_name),)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! generate_raw_params {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        /// The raw parameter values that a host DAW will set and modify.
        /// These are unscaled and are always in the [0.0, 1.0] range
        pub struct $raw_parameters {
            $($field_name: AtomicFloat,)*
            /// The host callback, used for communicating with the VST host
            pub host: vst::plugin::HostCallback,
        }
    };
}

#[macro_export]
macro_rules! generate_parameter_type {
    ($raw_parameters: ident, $parameter_type: ident;
     $($variant:ident, $field_name:ident, $name:expr, $idx:expr, $default:expr, $string:expr;)*) => {
        /// The list of parameters that exist.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $parameter_type {
            $($variant,)*
        }
    };
}

#[macro_export]
macro_rules! impl_all {
    ($raw_parameters: ident, $parameter_type: ident, $table: ident) => {
        impl_plugin_parameters! {$raw_parameters, $parameter_type}
        impl_get_set! {$raw_parameters, $parameter_type}
        $table! {generate_raw_params}
        $table! {generate_parameter_type}
        $table! {impl_from_i32}
        $table! {impl_into_i32}
        $table! {impl_display}
        $table! {impl_get_ref}
        $table! {impl_default}
        $table! {impl_get_default}
        $table! {impl_get_strings}
    };
}
