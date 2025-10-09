#[derive(Debug, Clone)]
pub struct Meters {
    pub input_voltage: f32,
    pub input_current: f32,
    pub output_voltage: f32,
    pub output_current: f32,
    pub batt_voltage: f32,
    pub batt_current: f32,
    pub batt_soc: f32,
    pub batt_int_resistance: f32,
}

impl Meters {
    pub fn average(values: Vec<Meters>) -> Self {
        let len = values.len() as f32;

        let mut sum = Meters {
            input_voltage: 0.0,
            input_current: 0.0,
            output_voltage: 0.0,
            output_current: 0.0,
            batt_voltage: 0.0,
            batt_current: 0.0,
            batt_soc: 0.0,
            batt_int_resistance: 0.0,
        };

        for v in values {
            sum.input_voltage += v.input_voltage;
            sum.input_current += v.input_current;
            sum.output_voltage += v.output_voltage;
            sum.output_current += v.output_current;
            sum.batt_voltage += v.batt_voltage;
            sum.batt_current += v.batt_current;
            sum.batt_soc += v.batt_soc;
            sum.batt_int_resistance += v.batt_int_resistance;
        }

        Meters {
            input_voltage: sum.input_voltage / len,
            input_current: sum.input_current / len,
            output_voltage: sum.output_voltage / len,
            output_current: sum.output_current / len,
            batt_voltage: sum.batt_voltage / len,
            batt_current: sum.batt_current / len,
            batt_soc: sum.batt_soc / len,
            batt_int_resistance: sum.batt_int_resistance / len,
        }
    }
}
