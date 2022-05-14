use std::fmt;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PARAMETERS_INITIAL: (Vec<&'static Parameter>, Vec<(u16,u16)>) = filter_sort_params(true);
    pub static ref PARAMETERS_POLL: (Vec<&'static Parameter>, Vec<(u16,u16)>) = filter_sort_params(false);
    pub static ref PARAMETER_MAP: HashMap<u16, &'static Parameter> = make_map();
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamKind {
    Text(Option<String>),
    NumberU16(Option<u16>),
    NumberI16(Option<i16>),
    NumberU32(Option<u32>),
    NumberI32(Option<i32>),
}

impl fmt::Display for ParamKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParamKind::Text(v) => write!(f, "Text: {}", v.clone().unwrap()),
            ParamKind::NumberU16(v) => write!(f, "NumberU16: {}", v.clone().unwrap()),
            ParamKind::NumberI16(v) => write!(f, "NumberI16: {}", v.clone().unwrap()),
            ParamKind::NumberU32(v) => write!(f, "NumberU32: {}", v.clone().unwrap()),
            ParamKind::NumberI32(v) => write!(f, "NumberI32: {}", v.clone().unwrap()),
        }
    }
}


#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: &'static str,
    pub value: ParamKind,
    pub desc: Option<&'static str>,
    pub unit: Option<&'static str>,
    pub gain: u16,
    pub reg_address: u16,
    pub len: u16,
    pub initial_read: bool,
    pub save_to_influx: bool,
}

fn make_map() -> HashMap<u16, &'static Parameter> {
    let mut res = HashMap::new();
    for p in PARAMETERS.iter() {
        res.insert(p.reg_address, p);
    }
    return res
}

#[rustfmt::skip]
const PARAMETERS:  &'static [Parameter] = &[
        Parameter{name: "model_name", value: ParamKind::Text(None), desc: None,  unit: None, gain: 1, reg_address: 30000, len: 15, initial_read: true, save_to_influx: false},
        Parameter{name: "serial_number", value: ParamKind::Text(None), desc: None, unit:  None, gain: 1, reg_address: 30015, len: 10, initial_read: true, save_to_influx: false},
        Parameter{name: "product_number", value: ParamKind::Text(None), desc: None, unit:  None, gain: 1, reg_address: 30025, len: 10, initial_read: true, save_to_influx: false},
        Parameter{name: "model_id", value: ParamKind::NumberU16(None), desc: None, unit: None, gain: 1, reg_address: 30070, len: 1, initial_read: true, save_to_influx: false},
        Parameter{name: "nb_pv_strings", value: ParamKind::NumberU16(None), desc: None, unit: None, gain: 1, reg_address: 30071, len: 1, initial_read: true, save_to_influx: false},
        Parameter{name: "nb_mpp_tracks", value: ParamKind::NumberU16(None), desc: None, unit: None, gain: 1, reg_address: 30072, len: 1, initial_read: true, save_to_influx: false},
        Parameter{name: "rated_power", value: ParamKind::NumberU32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 30073, len: 2, initial_read: true, save_to_influx: false},
        //Parameter{name: "P_max", value: ParamKind::NumberU32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 30075, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "S_max", value: ParamKind::NumberU32(None), desc: None, unit: Some("VA"), gain: 1, reg_address: 30077, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "Q_max_out", value: ParamKind::NumberI32(None), desc: None, unit: Some("VAr"), gain: 1, reg_address: 30079, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "Q_max_in", value: ParamKind::NumberI32(None), desc: None, unit: Some("VAr"), gain: 1, reg_address: 30081, len: 2, initial_read: false, save_to_influx: false},
        Parameter{name: "state_1", value: ParamKind::NumberU16(None), desc: None, unit: Some("state_bitfield16"), gain: 1, reg_address: 32000, len: 1, initial_read: false, save_to_influx: false},
        Parameter{name: "state_2", value: ParamKind::NumberU16(None), desc: None, unit: Some("state_opt_bitfield16"), gain: 1, reg_address: 32002, len: 1, initial_read: false, save_to_influx: false},
        Parameter{name: "state_3", value: ParamKind::NumberU32(None), desc: None, unit: Some("state_opt_bitfield32"), gain: 1, reg_address: 32003, len: 2, initial_read: false, save_to_influx: false},
        Parameter{name: "alarm_1", value: ParamKind::NumberU16(None), desc: None, unit: Some("alarm_bitfield16"), gain: 1, reg_address: 32008, len: 1, initial_read: false, save_to_influx: false},
        Parameter{name: "alarm_2", value: ParamKind::NumberU16(None), desc: None, unit: Some("alarm_bitfield16"), gain: 1, reg_address: 32009, len: 1, initial_read: false, save_to_influx: false},
        Parameter{name: "alarm_3", value: ParamKind::NumberU16(None), desc: None, unit: Some("alarm_bitfield16"), gain: 1, reg_address: 32010, len: 1, initial_read: false, save_to_influx: false},
        Parameter{name: "input_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 32064, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "line_voltage_A_B", value: ParamKind::NumberU16(None), desc: Some("grid_voltage"), unit: Some("V"), gain: 10, reg_address: 32066, len: 1, initial_read: false, save_to_influx: true},
        //Parameter{name: "line_voltage_B_C", value: ParamKind::NumberU16(None), desc: None, unit: Some("V"), gain: 10, reg_address: 32067, len: 1, initial_read: false, save_to_influx: true},
        //Parameter{name: "line_voltage_C_A", value: ParamKind::NumberU16(None), desc: None, unit: Some("V"), gain: 10, reg_address: 32068, len: 1, initial_read: false, save_to_influx: true},
        //Parameter{name: "phase_A_voltage", value: ParamKind::NumberU16(None), desc: None, unit: Some("V"), gain: 10, reg_address: 32069, len: 1, initial_read: false, save_to_influx: true},
        //Parameter{name: "phase_B_voltage", value: ParamKind::NumberU16(None), desc: None, unit: Some("V"), gain: 10, reg_address: 32070, len: 1, initial_read: false, save_to_influx: true},
        //Parameter{name: "phase_C_voltage", value: ParamKind::NumberU16(None), desc: None, unit: Some("V"), gain: 10, reg_address: 32071, len: 1, initial_read: false, save_to_influx: true},
        //Parameter{name: "phase_A_current", value: ParamKind::NumberI32(None), desc: Some("grid_current"), unit: Some("A"), gain: 1000, reg_address: 32072, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "phase_B_current", value: ParamKind::NumberI32(None), desc: None, unit: Some("A"), gain: 1000, reg_address: 32074, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "phase_C_current", value: ParamKind::NumberI32(None), desc: None, unit: Some("A"), gain: 1000, reg_address: 32076, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "day_active_power_peak", value: ParamKind::NumberI32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 32078, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "active_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 32080, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "reactive_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("VA"), gain: 1, reg_address: 32082, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "power_factor", value: ParamKind::NumberI16(None), desc: None, unit: None, gain: 1000, reg_address: 32084, len: 1, initial_read: false, save_to_influx: true},
        Parameter{name: "grid_frequency", value: ParamKind::NumberU16(None), desc: None, unit: Some("Hz"), gain: 100, reg_address: 32085, len: 1, initial_read: false, save_to_influx: true},
        Parameter{name: "efficiency", value: ParamKind::NumberU16(None), desc: None, unit: Some("%"), gain: 100, reg_address: 32086, len: 1, initial_read: false, save_to_influx: true},
        Parameter{name: "internal_temperature", value: ParamKind::NumberI16(None), desc: None, unit: Some("°C"), gain: 10, reg_address: 32087, len: 1, initial_read: false, save_to_influx: true},
        Parameter{name: "insulation_resistance", value: ParamKind::NumberU16(None), desc: None, unit: Some("MΩ"), gain: 100, reg_address: 32088, len: 1, initial_read: false, save_to_influx: true},
        Parameter{name: "device_status", value: ParamKind::NumberU16(None), desc: None, unit: Some("status_enum"), gain: 1, reg_address: 32089, len: 1, initial_read: false, save_to_influx: false},
        Parameter{name: "fault_code", value: ParamKind::NumberU16(None), desc: None, unit: None, gain: 1, reg_address: 32090, len: 1, initial_read: false, save_to_influx: false},
        //Parameter{name: "startup_time", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 32091, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "shutdown_time", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 32093, len: 2, initial_read: false, save_to_influx: false},
        Parameter{name: "accumulated_yield_energy", value: ParamKind::NumberU32(None), desc: None, unit: Some("kWh"), gain: 100, reg_address: 32106, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "unknown_time_1", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 32110, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "unknown_time_2", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 32156, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "unknown_time_3", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 32160, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "unknown_time_4", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 35113, len: 2, initial_read: false, save_to_influx: false},

        //Parameter{name: "storage_status", value: ParamKind::NumberI16(None), desc: None, unit: Some("storage_status_enum"), gain: 1, reg_address: 37000, len: 1, initial_read: false, save_to_influx: false},
        Parameter{name: "grid_A_voltage", value: ParamKind::NumberI32(None), desc: None, unit: Some("V"), gain: 10, reg_address: 37101, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "grid_B_voltage", value: ParamKind::NumberI32(None), desc: None, unit: Some("V"), gain: 10, reg_address: 37103, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "grid_C_voltage", value: ParamKind::NumberI32(None), desc: None, unit: Some("V"), gain: 10, reg_address: 37105, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "active_grid_A_current", value: ParamKind::NumberI32(None), desc: None, unit: Some("I"), gain: 100, reg_address: 37107, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_B_current", value: ParamKind::NumberI32(None), desc: None, unit: Some("I"), gain: 100, reg_address: 37109, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_C_current", value: ParamKind::NumberI32(None), desc: None, unit: Some("I"), gain: 100, reg_address: 37111, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "power_meter_active_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 37113, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "daily_yield_energy", value: ParamKind::NumberU32(None), desc: None, unit: Some("kWh"), gain: 100, reg_address: 32114, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "power_meter_reactive_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("VA"), gain: 1, reg_address: 37115, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "active_grid_power_factor", value: ParamKind::NumberI16(None), desc: None, unit: None, gain: 1000, reg_address: 37117, len: 1, initial_read: false, save_to_influx: true},
        Parameter{name: "active_grid_frequency", value: ParamKind::NumberI16(None), desc: None, unit: Some("Hz"), gain: 100, reg_address: 37118, len: 1, initial_read: false, save_to_influx: true},
        Parameter{name: "grid_exported_energy", value: ParamKind::NumberI32(None), desc: None, unit: Some("kWh"), gain: 100, reg_address: 37119, len: 2, initial_read: false, save_to_influx: true},
        Parameter{name: "grid_accumulated_energy", value: ParamKind::NumberU32(None), desc: None, unit: Some("kWh"), gain: 100, reg_address: 37121, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "grid_accumulated_reactive", value: ParamKind::NumberU32(None), desc: None, unit: Some("kVarh"), gain: 100, reg_address: 37123, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_A_B_voltage", value: ParamKind::NumberI32(None), desc: None, unit: Some("V"), gain: 10, reg_address: 37126, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_B_C_voltage", value: ParamKind::NumberI32(None), desc: None, unit: Some("V"), gain: 10, reg_address: 37128, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_C_A_voltage", value: ParamKind::NumberI32(None), desc: None, unit: Some("V"), gain: 10, reg_address: 37130, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_A_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 37132, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_B_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 37134, len: 2, initial_read: false, save_to_influx: true},
        //Parameter{name: "active_grid_C_power", value: ParamKind::NumberI32(None), desc: None, unit: Some("W"), gain: 1, reg_address: 37136, len: 2, initial_read: false, save_to_influx: true},

        //Parameter{name: "system_time", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 40000, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "unknown_time_5", value: ParamKind::NumberU32(None), desc: None, unit: Some("epoch"), gain: 1, reg_address: 40500, len: 2, initial_read: false, save_to_influx: false},
        //Parameter{name: "grid_code", value: ParamKind::NumberU16(None), desc: None, unit: Some("grid_enum"), gain: 1, reg_address: 42000, len: 1, initial_read: false, save_to_influx: false},
        //Parameter{name: "time_zone", value: ParamKind::NumberI16(None), desc: None, unit: Some("min"), gain: 1, reg_address: 43006, len: 1, initial_read: false, save_to_influx: false},
    ];


fn filter_sort_params(initial_read: bool) -> (Vec<&'static Parameter>, Vec<(u16,u16)>)  {

    let mut params: Vec<&Parameter> = PARAMETERS.iter().collect();
    params.sort_by(|a,b| a.reg_address.partial_cmp(&b.reg_address).unwrap());

    let params_to_read: Vec<&Parameter> = params.into_iter().filter(|s| {
        (initial_read && s.initial_read)
            || (!initial_read
                && (s.save_to_influx
                    || s.name.starts_with("state_")
                    || s.name.starts_with("alarm_")
                    || s.name.ends_with("_status")
                    || s.name.ends_with("_code")))}).collect();

    let mut params_addr = BinaryHeap::new();
    use std::cmp::Reverse;
    for p in &params_to_read {
        for addr_offset in 0 .. p.len { 
            params_addr.push(Reverse(p.reg_address + addr_offset));
        }
    }

    
    let mut addr_span: Vec<(u16,u16)> = Vec::new();
    {
        let mut addr_init: u16 = params_addr.pop().unwrap().0;
        let mut addr_end: u16 = addr_init;
        let mut addr_len = 1;
        while !params_addr.is_empty() {
            let addr_next: u16 = params_addr.pop().unwrap().0;
            if addr_next == addr_end + 1 {
                addr_end = addr_next;
                addr_len = addr_len + 1;
                if params_addr.is_empty() {
                    addr_span.push((addr_init, addr_len));    
                }
                continue
            } else {
                addr_span.push((addr_init, addr_len));
                addr_init = addr_next;
                addr_end = addr_next;
                addr_len = 1;
            }
        }
    }
    

   (params_to_read, addr_span)
}
