use lazy_static::lazy_static;

pub struct Telemetry {}

impl Telemetry {
    pub fn new() -> Self {
        Telemetry {}
    }
}

lazy_static! {
    pub static ref TELEMETRY: Telemetry = Telemetry::new();
}
