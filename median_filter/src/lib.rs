#[macro_use]
extern crate common;

use std::sync::Arc;

use median::heap::Filter;
use vst::{
    api::Supported,
    buffer::AudioBuffer,
    host::Host,
    plugin::{CanDo, Category, HostCallback, Info, Plugin, PluginParameters},
    util::AtomicFloat,
};

struct MedianFilter {
    params: Arc<RawParameters>,
    left_filter: Filter<f32>,
    right_filter: Filter<f32>,
    last_window_size: usize,
}

impl Plugin for MedianFilter {
    fn new(host: HostCallback) -> Self {
        MedianFilter {
            params: Arc::new(RawParameters::default(host)),
            left_filter: Filter::new(50),
            right_filter: Filter::new(50),
            last_window_size: 50,
        }
    }

    fn init(&mut self) {
        let params = Parameters::from(self.params.as_ref());
        self.last_window_size = params.window_size;
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Median Filter".to_string(),
            vendor: "a2aaron".to_string(),
            // Used by hosts to differentiate between plugins.
            // Don't worry much about this now - just fill in a random number.
            unique_id: 612413,
            version: 1,
            category: Category::Effect,
            // Subtract one here due to "error" type
            parameters: ParameterType::COUNT as i32,
            // Two audio inputs
            inputs: 2,
            // Two channel audio!
            outputs: 2,
            // For now, fill in the rest of our fields with `Default` info.
            ..Default::default()
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::Bypass => Supported::Yes,
            _ => Supported::No,
        }
    }

    // Output audio given the current state of the VST
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        self.reset_if_changed();
        let params = Parameters::from(self.params.as_ref());
        let wet_dry = params.wet_dry;
        let num_samples = buffer.samples();

        let (inputs, mut outputs) = buffer.split();
        let left_input = &inputs[0];
        let left_output = &mut outputs[0];

        for i in 0..num_samples {
            self.left_filter.consume(left_input[i]);
            let out = if self.left_filter.is_empty() != 0 {
                self.left_filter.median()
            } else {
                0.0
            };
            left_output[i] = left_input[i] * (1.0 - wet_dry) + out * wet_dry;
        }

        let right_input = &inputs[1];
        let right_output = &mut outputs[1];

        for i in 0..num_samples {
            self.right_filter.consume(right_input[i]);
            let out = if self.right_filter.is_empty() != 0 {
                self.right_filter.median()
            } else {
                0.0
            };
            right_output[i] = right_input[i] * (1.0 - wet_dry) + out * wet_dry;
        }
    }

    // The raw parameters exposed to the host
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl MedianFilter {
    fn reset_if_changed(&mut self) {
        let params = Parameters::from(self.params.as_ref());
        if params.window_size != self.last_window_size {
            self.left_filter = Filter::new(params.window_size);
            self.right_filter = Filter::new(params.window_size);
            self.last_window_size = params.window_size;
        }
    }
}

struct Parameters {
    window_size: usize,
    wet_dry: f32,
}

impl From<&RawParameters> for Parameters {
    fn from(params: &RawParameters) -> Self {
        Parameters {
            window_size: ((params.window_size.get() * 100.0) as usize).max(1),
            wet_dry: params.wet_dry.get(),
        }
    }
}

impl RawParameters {
    pub fn set(&self, value: f32, parameter: ParameterType) {
        // These are needed so Ableton will notice parameter changes in the
        // "Configure" window.
        // TODO: investigate if I should send this only on mouseup/mousedown
        self.host.begin_edit(parameter.into());
        self.get_ref(parameter).set(value);
        self.host.end_edit(parameter.into());
    }

    pub fn get(&self, parameter: ParameterType) -> f32 {
        self.get_ref(parameter).get()
    }

    /// Returns a user-facing text output for the given parameter. This is broken
    /// into a tuple consisting of (`value`, `units`)
    fn get_strings(&self, parameter: ParameterType) -> (String, String) {
        let params = Parameters::from(self);

        fn make_strings(value: f32, label: &str) -> (String, String) {
            (format!("{:.2}", value), label.to_string())
        }

        match parameter {
            ParameterType::WetDry => make_strings(params.wet_dry * 100.0, "% Wet"),
            ParameterType::WindowSize => {
                (format!("{}", params.window_size), " Samples".to_string())
            }
        }
    }
}

/// The raw parameter values that a host DAW will set and modify.
/// These are unscaled and are always in the [0.0, 1.0] range
pub struct RawParameters {
    window_size: AtomicFloat,
    wet_dry: AtomicFloat,
    host: HostCallback,
}

/// The type of parameter. "Error" is included as a convience type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    WindowSize,
    WetDry,
}

macro_rules! table {
    ($macro:ident) => {
        $macro! {
        //  RawParameter identifier, ParameterType identifier
            RawParameters,           ParameterType;
        //  variant                     idx    name            field_name    default
            ParameterType::WetDry,      0,     "Wet/Dry",      wet_dry,      0.5;
            ParameterType::WindowSize,  1,     "Window Size",  window_size,  0.5;
        }
    };
}

impl ParameterType {
    pub const COUNT: usize = 2;
}

impl_all! {RawParameters, ParameterType, table}

// Export symbols for main
vst::plugin_main!(MedianFilter);
