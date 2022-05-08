use influxdb2;
use chrono::Timelike;
use chrono::{Local, LocalResult, NaiveDateTime, TimeZone};
use io::ErrorKind;
use simplelog::*;
use std::fmt;
use std::io;
use std::collections::HashMap;
use std::collections::BinaryHeap;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

use super::defs::*;
use futures::prelude::*;

pub const SUN2000_POLL_INTERVAL_SECS: u32 = 5; //secs between polling
pub const SUN2000_STATS_DUMP_INTERVAL_SECS: f32 = 30.0; //secs between showing stats
pub const SUN2000_ATTEMPTS_PER_PARAM: u8 = 3; //max read attempts per single parameter

// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Debug, PartialEq)]
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
    name: String,
    value: ParamKind,
    desc: Option<&'static str>,
    unit: Option<&'static str>,
    gain: u16,
    reg_address: u16,
    len: u16,
    initial_read: bool,
    save_to_influx: bool,
}

impl Parameter {
    pub fn new(
        name: &'static str,
        value: ParamKind,
        desc: Option<&'static str>,
        unit: Option<&'static str>,
        gain: u16,
        reg_address: u16,
        len: u16,
        initial_read: bool,
        save_to_influx: bool,
    ) -> Self {
        Self {
            name: String::from(name),
            value,
            desc,
            unit,
            gain,
            reg_address,
            len,
            initial_read,
            save_to_influx,
        }
    }

    pub fn new_from_string(
        name: String,
        value: ParamKind,
        desc: Option<&'static str>,
        unit: Option<&'static str>,
        gain: u16,
        reg_address: u16,
        len: u16,
        initial_read: bool,
        save_to_influx: bool,
    ) -> Self {
        Self {
            name,
            value,
            desc,
            unit,
            gain,
            reg_address,
            len,
            initial_read,
            save_to_influx,
        }
    }

    pub fn get_text_value(&self) -> String {
        match &self.value {
            ParamKind::Text(v) => {
                return v.clone().unwrap();
            }
            ParamKind::NumberU16(v) => {
                return if self.gain != 1 {
                    (v.clone().unwrap() as f32 / self.gain as f32).to_string()
                } else {
                    v.clone().unwrap().to_string()
                }
            }
            ParamKind::NumberI16(v) => {
                return if self.gain != 1 {
                    (v.clone().unwrap() as f32 / self.gain as f32).to_string()
                } else {
                    v.clone().unwrap().to_string()
                }
            }
            ParamKind::NumberU32(v) => {
                return if self.gain != 1 {
                    (v.clone().unwrap() as f32 / self.gain as f32).to_string()
                } else {
                    if self.unit.unwrap_or_default() == "epoch" {
                        match *v {
                            Some(epoch_secs) => {
                                let naive = NaiveDateTime::from_timestamp(epoch_secs as i64, 0);
                                match Local.from_local_datetime(&naive) {
                                    LocalResult::Single(dt) => {
                                        format!("{}, {:?}", epoch_secs, dt.to_rfc2822())
                                    }
                                    _ => "timestamp conversion error".into(),
                                }
                            }
                            None => "None".into(),
                        }
                    } else {
                        v.clone().unwrap().to_string()
                    }
                }
            }
            ParamKind::NumberI32(v) => {
                return if self.gain != 1 {
                    (v.clone().unwrap() as f32 / self.gain as f32).to_string()
                } else {
                    v.clone().unwrap().to_string()
                }
            }
        }
    }

    pub fn get_influx_value(&self) -> influxdb2::models::FieldValue {
        match &self.value {
            ParamKind::Text(Some(v)) => {
                return influxdb2::models::FieldValue::String((*v).clone());
            }
            ParamKind::NumberU16(Some(v)) => {
                if self.gain != 1 {
                    return (*v as f64 / self.gain as f64).into();
                } else {
                    return (*v as i64).into();
                }
            }
            ParamKind::NumberI16(Some(v)) => {
                if self.gain != 1 {
                    return (*v as f64 / self.gain as f64).into();
                } else {
                    return (*v as i64).into();
                }
            }
            ParamKind::NumberU32(Some(v)) => {
                if self.gain != 1 {
                    return (*v as f64 / self.gain as f64).into();
                } else {
                    return (*v as i64).into();
                }
            }
            ParamKind::NumberI32(Some(v)) => {
                if self.gain != 1 {
                    return (*v as f64 / self.gain as f64).into();
                } else {
                    return (*v as i64).into();
                }
            }
            _ => {panic!("{:?}", self)}
        }
    }
}


pub struct Sun2000 {
    pub name: String,
    pub host_port: String,
    pub poll_ok: u64,
    pub poll_errors: u64,
    pub influxdb_url: Option<String>,
    pub influxdb_org: Option<String>,
    pub influxdb_token: Option<String>,
    pub influxdb_bucket: Option<String>,
    pub mode_change_script: Option<String>,
    pub optimizers: bool,
    pub battery_installed: bool,
    pub dongle_connection: bool,
}

impl Sun2000 {
    #[rustfmt::skip]
    pub fn param_table() -> Vec<Parameter> {
        vec![
            Parameter::new("model_name", ParamKind::Text(None), None,  None, 1, 30000, 15, true, false),
            Parameter::new("serial_number", ParamKind::Text(None), None,  None, 1, 30015, 10, true, false),
            Parameter::new("product_number", ParamKind::Text(None), None,  None, 1, 30025, 10, true, false),
            Parameter::new("model_id", ParamKind::NumberU16(None), None, None, 1, 30070, 1, true, false),
            Parameter::new("nb_pv_strings", ParamKind::NumberU16(None), None, None, 1, 30071, 1, true, false),
            Parameter::new("nb_mpp_tracks", ParamKind::NumberU16(None), None, None, 1, 30072, 1, true, false),
            Parameter::new("rated_power", ParamKind::NumberU32(None), None, Some("W"), 1, 30073, 2, true, false),
            //Parameter::new("P_max", ParamKind::NumberU32(None), None, Some("W"), 1, 30075, 2, false, false),
            //Parameter::new("S_max", ParamKind::NumberU32(None), None, Some("VA"), 1, 30077, 2, false, false),
            //Parameter::new("Q_max_out", ParamKind::NumberI32(None), None, Some("VAr"), 1, 30079, 2, false, false),
            //Parameter::new("Q_max_in", ParamKind::NumberI32(None), None, Some("VAr"), 1, 30081, 2, false, false),
            Parameter::new("state_1", ParamKind::NumberU16(None), None, Some("state_bitfield16"), 1, 32000, 1, false, false),
            Parameter::new("state_2", ParamKind::NumberU16(None), None, Some("state_opt_bitfield16"), 1, 32002, 1, false, false),
            Parameter::new("state_3", ParamKind::NumberU32(None), None, Some("state_opt_bitfield32"), 1, 32003, 2, false, false),
            Parameter::new("alarm_1", ParamKind::NumberU16(None), None, Some("alarm_bitfield16"), 1, 32008, 1, false, false),
            Parameter::new("alarm_2", ParamKind::NumberU16(None), None, Some("alarm_bitfield16"), 1, 32009, 1, false, false),
            Parameter::new("alarm_3", ParamKind::NumberU16(None), None, Some("alarm_bitfield16"), 1, 32010, 1, false, false),
            Parameter::new("input_power", ParamKind::NumberI32(None), None, Some("W"), 1, 32064, 2, false, true),
            //Parameter::new("line_voltage_A_B", ParamKind::NumberU16(None), Some("grid_voltage"), Some("V"), 10, 32066, 1, false, true),
            //Parameter::new("line_voltage_B_C", ParamKind::NumberU16(None), None, Some("V"), 10, 32067, 1, false, true),
            //Parameter::new("line_voltage_C_A", ParamKind::NumberU16(None), None, Some("V"), 10, 32068, 1, false, true),
            //Parameter::new("phase_A_voltage", ParamKind::NumberU16(None), None, Some("V"), 10, 32069, 1, false, true),
            //Parameter::new("phase_B_voltage", ParamKind::NumberU16(None), None, Some("V"), 10, 32070, 1, false, true),
            //Parameter::new("phase_C_voltage", ParamKind::NumberU16(None), None, Some("V"), 10, 32071, 1, false, true),
            //Parameter::new("phase_A_current", ParamKind::NumberI32(None), Some("grid_current"), Some("A"), 1000, 32072, 2, false, true),
            //Parameter::new("phase_B_current", ParamKind::NumberI32(None), None, Some("A"), 1000, 32074, 2, false, true),
            //Parameter::new("phase_C_current", ParamKind::NumberI32(None), None, Some("A"), 1000, 32076, 2, false, true),
            Parameter::new("day_active_power_peak", ParamKind::NumberI32(None), None, Some("W"), 1, 32078, 2, false, true),
            Parameter::new("active_power", ParamKind::NumberI32(None), None, Some("W"), 1, 32080, 2, false, true),
            Parameter::new("reactive_power", ParamKind::NumberI32(None), None, Some("VA"), 1, 32082, 2, false, true),
            Parameter::new("power_factor", ParamKind::NumberI16(None), None, None, 1000, 32084, 1, false, true),
            Parameter::new("grid_frequency", ParamKind::NumberU16(None), None, Some("Hz"), 100, 32085, 1, false, true),
            Parameter::new("efficiency", ParamKind::NumberU16(None), None, Some("%"), 100, 32086, 1, false, true),
            Parameter::new("internal_temperature", ParamKind::NumberI16(None), None, Some("°C"), 10, 32087, 1, false, true),
            Parameter::new("insulation_resistance", ParamKind::NumberU16(None), None, Some("MΩ"), 100, 32088, 1, false, true),
            Parameter::new("device_status", ParamKind::NumberU16(None), None, Some("status_enum"), 1, 32089, 1, false, false),
            Parameter::new("fault_code", ParamKind::NumberU16(None), None, None, 1, 32090, 1, false, false),
            //Parameter::new("startup_time", ParamKind::NumberU32(None), None, Some("epoch"), 1, 32091, 2, false, false),
            //Parameter::new("shutdown_time", ParamKind::NumberU32(None), None, Some("epoch"), 1, 32093, 2, false, false),
            Parameter::new("accumulated_yield_energy", ParamKind::NumberU32(None), None, Some("kWh"), 100, 32106, 2, false, true),
            //Parameter::new("unknown_time_1", ParamKind::NumberU32(None), None, Some("epoch"), 1, 32110, 2, false, false),
            //Parameter::new("unknown_time_2", ParamKind::NumberU32(None), None, Some("epoch"), 1, 32156, 2, false, false),
            //Parameter::new("unknown_time_3", ParamKind::NumberU32(None), None, Some("epoch"), 1, 32160, 2, false, false),
            //Parameter::new("unknown_time_4", ParamKind::NumberU32(None), None, Some("epoch"), 1, 35113, 2, false, false),

            //Parameter::new("storage_status", ParamKind::NumberI16(None), None, Some("storage_status_enum"), 1, 37000, 1, false, false),
            Parameter::new("grid_A_voltage", ParamKind::NumberI32(None), None, Some("V"), 10, 37101, 2, false, true),
            //Parameter::new("grid_B_voltage", ParamKind::NumberI32(None), None, Some("V"), 10, 37103, 2, false, true),
            //Parameter::new("grid_C_voltage", ParamKind::NumberI32(None), None, Some("V"), 10, 37105, 2, false, true),
            Parameter::new("active_grid_A_current", ParamKind::NumberI32(None), None, Some("I"), 100, 37107, 2, false, true),
            //Parameter::new("active_grid_B_current", ParamKind::NumberI32(None), None, Some("I"), 100, 37109, 2, false, true),
            //Parameter::new("active_grid_C_current", ParamKind::NumberI32(None), None, Some("I"), 100, 37111, 2, false, true),
            Parameter::new("power_meter_active_power", ParamKind::NumberI32(None), None, Some("W"), 1, 37113, 2, false, true),
            Parameter::new("daily_yield_energy", ParamKind::NumberU32(None), None, Some("kWh"), 100, 32114, 2, false, true),
            Parameter::new("power_meter_reactive_power", ParamKind::NumberI32(None), None, Some("VA"), 1, 37115, 2, false, true),
            Parameter::new("active_grid_power_factor", ParamKind::NumberI16(None), None, None, 1000, 37117, 1, false, true),
            Parameter::new("active_grid_frequency", ParamKind::NumberI16(None), None, Some("Hz"), 100, 37118, 1, false, true),
            Parameter::new("grid_exported_energy", ParamKind::NumberI32(None), None, Some("kWh"), 100, 37119, 2, false, true),
            Parameter::new("grid_accumulated_energy", ParamKind::NumberU32(None), None, Some("kWh"), 100, 37121, 2, false, true),
            //Parameter::new("grid_accumulated_reactive", ParamKind::NumberU32(None), None, Some("kVarh"), 100, 37123, 2, false, true),
            //Parameter::new("active_grid_A_B_voltage", ParamKind::NumberI32(None), None, Some("V"), 10, 37126, 2, false, true),
            //Parameter::new("active_grid_B_C_voltage", ParamKind::NumberI32(None), None, Some("V"), 10, 37128, 2, false, true),
            //Parameter::new("active_grid_C_A_voltage", ParamKind::NumberI32(None), None, Some("V"), 10, 37130, 2, false, true),
            //Parameter::new("active_grid_A_power", ParamKind::NumberI32(None), None, Some("W"), 1, 37132, 2, false, true),
            //Parameter::new("active_grid_B_power", ParamKind::NumberI32(None), None, Some("W"), 1, 37134, 2, false, true),
            //Parameter::new("active_grid_C_power", ParamKind::NumberI32(None), None, Some("W"), 1, 37136, 2, false, true),

            //Parameter::new("system_time", ParamKind::NumberU32(None), None, Some("epoch"), 1, 40000, 2, false, false),
            //Parameter::new("unknown_time_5", ParamKind::NumberU32(None), None, Some("epoch"), 1, 40500, 2, false, false),
            //Parameter::new("grid_code", ParamKind::NumberU16(None), None, Some("grid_enum"), 1, 42000, 1, false, false),
            //Parameter::new("time_zone", ParamKind::NumberI16(None), None, Some("min"), 1, 43006, 1, false, false),
        ]
    }

    async fn read_params(
        &mut self,
        mut ctx: Context,
        parameters: &Vec<Parameter>,
        initial_read: bool
    ) -> io::Result<(Context, Vec<Parameter>, u64)> {

        let mut params: Vec<Parameter> = vec![];
        let mut disconnected = false;
        let now = Instant::now();

        let params_to_read: Vec<&Parameter> = parameters.into_iter().filter(|s| {
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
                debug!("-> READ {} {}...", p.name, p.reg_address + addr_offset);
            }
        }

        
        let mut addr_span = Vec::new();
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

        let mut value_map = HashMap::new();

        for (addr_start, addr_len) in &addr_span {
            if disconnected {
                break;
            }
            let mut attempts = 0;
            while attempts < SUN2000_ATTEMPTS_PER_PARAM {
                attempts = attempts + 1;
                debug!("-> obtaining spam {} {}...", addr_start, addr_len);
                let retval = ctx.read_holding_registers(*addr_start, *addr_len);
                let read_res;
                let start = Instant::now();
                let read_time;
                match timeout(Duration::from_secs_f32(1.2), retval).await {
                    Ok(res) => {
                        read_res = res;
                        read_time = start.elapsed();
                    }
                    Err(e) => {
                        let msg = format!(
                            "<i>{}</i>: read timeout (attempt #{} of {}), register: <green><i>{}-{}</>, error: <b>{}</>",
                            self.name, attempts, SUN2000_ATTEMPTS_PER_PARAM, addr_start, addr_len, e
                        );
                        if attempts == SUN2000_ATTEMPTS_PER_PARAM {
                            error!("{}", msg);
                            break;
                        } else {
                            warn!("{}", msg);
                            continue;
                        };
                    }
                }
                match read_res {
                    Ok(data) => {
                        if read_time > Duration::from_secs_f32(1.0) {
                            warn!(
                                "<i>{}</i>: inverter has lagged during read, register: <green><i>{}-{}</>, read time: <b>{:?}</>",
                                self.name, addr_start, addr_len, read_time
                            );
                        }

                        let mut addr = *addr_start;
                        for v in data {
                            value_map.insert(addr, v);
                            addr = addr + 1;
                        }

                        break; //read next parameter span
                    }
                    Err(e) => {
                        let msg = format!(
                            "<i>{}</i>: read error (attempt #{} of {}), register: <green><i>{}-{}</>, error: <b>{}</>, read time: <b>{:?}</>",
                            self.name, attempts, SUN2000_ATTEMPTS_PER_PARAM, addr_start, addr_len, e, read_time
                        );
                        match e.kind() {
                            ErrorKind::BrokenPipe | ErrorKind::ConnectionReset => {
                                error!("{}", msg);
                                disconnected = true;
                                break;
                            }
                            _ => {
                                if attempts == SUN2000_ATTEMPTS_PER_PARAM {
                                    error!("{}", msg);
                                    break;
                                } else {
                                    warn!("{}", msg);
                                    continue;
                                };
                            }
                        }
                    }
                }
            }
        }

        for (a, v) in &value_map {
            debug!("MAP {} {}", a, v);
        }

        for p in &params_to_read {
            let mut val2:ParamKind;
            let mut values = Vec::new();
            for addr in p.reg_address .. p.reg_address +  p.len {                
                let v = value_map.get(&addr);
                match v {
                    Some(value) => {
                        values.push(*value)
                    }
                    None => {
                        continue
                    }
                }
            }
            match &p.value {
                ParamKind::Text(_) => {
                    let bytes: Vec<u8> = values.iter().fold(vec![], |mut x, elem| {
                        if (elem >> 8) as u8 != 0 {
                            x.push((elem >> 8) as u8);
                        }
                        if (elem & 0xff) as u8 != 0 {
                            x.push((elem & 0xff) as u8);
                        }
                        x
                    });
                    let id = String::from_utf8(bytes).unwrap();
                    val2 = ParamKind::Text(Some(id));
                }
                ParamKind::NumberU16(_) => {
                    debug!("-> {} = {:?}", p.name, values);
                    val2 = ParamKind::NumberU16(Some(values[0] as u16));
                }
                ParamKind::NumberI16(_) => {
                    debug!("-> {} = {:?}", p.name, values);
                    val2 = ParamKind::NumberI16(Some(values[0] as i16));
                }
                ParamKind::NumberU32(_) => {
                    let new_val: u32 = ((values[0] as u32) << 16) | values[1] as u32;
                    debug!("-> {} = {:X?} {:X}", p.name, values, new_val);
                    val2 = ParamKind::NumberU32(Some(new_val));
                    if p.unit.unwrap_or_default() == "epoch" && new_val == 0 {
                        //zero epoch makes no sense, let's set it to None
                        val2 = ParamKind::NumberU32(None);
                    }
                }
                ParamKind::NumberI32(_) => {
                    let new_val: i32 =
                        ((values[0] as i32) << 16) | (values[1] as u32) as i32;
                    debug!("-> {} = {:X?} {:X}", p.name, values, new_val);
                    val2 = ParamKind::NumberI32(Some(new_val));
                }
            }

            let param = Parameter::new_from_string(
                p.name.clone(),
                val2,
                p.desc.clone(),
                p.unit.clone(),
                p.gain,
                p.reg_address,
                p.len,
                p.initial_read,
                p.save_to_influx,
            );
            params.push(param.clone());
        }

        let elapsed = now.elapsed();
        let ms = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;
        info!(
            "{}: read {} parameters [⏱️ {} ms]",
            self.name,
            params.len(),
            ms
        );

        Ok((ctx, params, ms))
    }

    fn attribute_parser(&self, mut a: Vec<u8>) -> Result<()> {
        //search for 'Description about the first device' (0x88)
        if let Some(index) = a.iter().position(|&x| x == 0x88) {
            //strip beginning bytes up to descriptor start
            a.drain(0..=index);

            //next (first) byte is len
            let len = a[0] as usize;

            //leave only the relevant descriptor string
            a = a.drain(1..=len).collect();

            //convert it to string
            let x = String::from_utf8(a)?;

            //split by semicolons
            let split = x.split(";");

            //parse and dump all attributes
            info!(
                "<i>{}</i>: <blue>Device Description attributes:</>",
                self.name
            );
            for s in split {
                let mut sp = s.split("=");
                let id = sp.next();
                let val = sp.next();
                if id.is_none() || val.is_none() {
                    continue;
                }
                info!(
                    "<i>{}</i>: <black>{}:</> {}: <b><cyan>{}</>",
                    self.name,
                    id.unwrap(),
                    get_attribute_name(id.unwrap()),
                    val.unwrap()
                );
            }
        }
        Ok(())
    }

    #[rustfmt::skip]
    pub async fn worker(&mut self, worker_cancel_flag: Arc<AtomicBool>) -> Result<()> {
        info!("<i>{}</>: Starting task", self.name);
        let mut stats_interval = Instant::now();
        let mut terminated = false;

        let mut state = Sun2000State {
            device_status: None,
            storage_status: None,
            grid_code: None,
            state_1: None,
            state_2: None,
            state_3: None,
            alarm_1: None,
            alarm_2: None,
            alarm_3: None,
            fault_code: None,
        };

        loop {
            if terminated || worker_cancel_flag.load(Ordering::SeqCst) {
                break;
            }

            let socket_addr = self.host_port.parse().unwrap();

            let slave;
            if self.dongle_connection {
                //USB dongle connection: Slave ID has to be 0x01
                slave = Slave(0x01);
            } else {
                //internal wifi: Slave ID has to be 0x00, otherwise the inverter is not responding
                slave = Slave(0x00);
            }

            info!("<i>{}</>: connecting to <u>{}</>...", self.name, self.host_port);
            let retval = tcp::connect_slave(socket_addr, slave);
            let conn;
            match timeout(Duration::from_secs(5), retval).await {
                Ok(res) => { conn = res; }
                Err(e) => {
                    error!("<i>{}</>: connect timeout: <b>{}</>", self.name, e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            }

            match conn {
                Ok(mut ctx) => {
                    info!("<i>{}</>: connected successfully", self.name);
                    //initial parameters table
                    let parameters = Sun2000::param_table();
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    //obtaining all parameters from inverter
                    let (new_ctx, params, _) = self.read_params(ctx, &parameters, true).await?;
                    ctx = new_ctx;
                    
                    for p in &params {
                        match &p.value {
                            ParamKind::Text(_) => match p.name.as_ref() {
                                "model_name" => {
                                    info!("<i>{}</>: model name: <b><cyan>{}</>", self.name, &p.get_text_value());
                                }
                                "serial_number" => {
                                    info!("<i>{}</>: serial number: <b><cyan>{}</>", self.name, &p.get_text_value());
                                }
                                "product_number" => {
                                    info!("<i>{}</>: product number: <b><cyan>{}</>", self.name, &p.get_text_value());
                                }
                                _ => {}
                            },
                            ParamKind::NumberU32(_) => match p.name.as_ref() {
                                "rated_power" => {
                                    info!(
                                        "<i>{}</>: rated power: <b><cyan>{} {}</>",
                                        self.name,
                                        &p.get_text_value(),
                                        p.unit.clone().unwrap_or_default()
                                    );
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }

                    /*match nb_pv_strings {
                        Some(n) => {
                            info!("<i>{}</>: number of available strings: <b><cyan>{}</>", self.name, n);
                            for i in 1..=n {
                                parameters.push(Parameter::new_from_string(format!("pv_{:02}_voltage", i), ParamKind::NumberI16(None), None, Some("V"), 10, 32014 + i*2, 1, false, true));
                                parameters.push(Parameter::new_from_string(format!("pv_{:02}_current", i), ParamKind::NumberI16(None), None, Some("A"), 100, 32015 + i*2, 1, false, true));
                            }
                        }
                        None => {}
                    }

                    if self.optimizers {
                        info!("<i>{}</>: config: optimizers enabled", self.name);
                        parameters.push(Parameter::new("nb_optimizers", ParamKind::NumberU16(None), None, None, 1, 37200, 1, false, false));
                        parameters.push(Parameter::new("nb_online_optimizers", ParamKind::NumberU16(None), None, None, 1, 37201, 1, false, true));
                    }

                    if self.battery_installed {
                        info!("<i>{}</>: config: battery installed", self.name);
                        parameters.push(Parameter::new("storage_working_mode", ParamKind::NumberI16(None), None, Some("storage_working_mode_enum"), 1, 47004, 1, false, true));
                        parameters.push(Parameter::new("storage_time_of_use_price", ParamKind::NumberI16(None), None, Some("storage_tou_price_enum"), 1, 47027, 1, false, true));
                        parameters.push(Parameter::new("storage_lcoe", ParamKind::NumberU32(None), None, None, 1000, 47069, 2, false, true));
                        parameters.push(Parameter::new("storage_maximum_charging_power", ParamKind::NumberU32(None), None, Some("W"), 1, 47075, 2, false, true));
                        parameters.push(Parameter::new("storage_maximum_discharging_power", ParamKind::NumberU32(None), None, Some("W"), 1, 47077, 2, false, true));
                        parameters.push(Parameter::new("storage_power_limit_grid_tied_point", ParamKind::NumberI32(None), None, Some("W"), 1, 47079, 2, false, true));
                        parameters.push(Parameter::new("storage_charging_cutoff_capacity", ParamKind::NumberU16(None), None, Some("%"), 10, 47081, 1, false, true));
                        parameters.push(Parameter::new("storage_discharging_cutoff_capacity", ParamKind::NumberU16(None), None, Some("%"), 10, 47082, 1, false, true));
                        parameters.push(Parameter::new("storage_forced_charging_and_discharging_period", ParamKind::NumberU16(None), None, Some("min"), 1, 47083, 1, false, true));
                        parameters.push(Parameter::new("storage_forced_charging_and_discharging_power", ParamKind::NumberI32(None), None, Some("min"), 1, 47084, 2, false, true));
                        parameters.push(Parameter::new("storage_current_day_charge_capacity", ParamKind::NumberU32(None), None, Some("kWh"), 100, 37015, 2, false, true));
                        parameters.push(Parameter::new("storage_current_day_discharge_capacity", ParamKind::NumberU32(None), None, Some("kWh"), 100, 37017, 2, false, true));
                    }*/

                    // obtain Device Description Definition
                    use tokio_modbus::prelude::*;
                    let retval = ctx.call(Request::Custom(0x2b, vec![0x0e, 0x03, 0x87]));
                    match timeout(Duration::from_secs_f32(5.0), retval).await {
                        Ok(res) => match res {
                            Ok(rsp) => match rsp {
                                Response::Custom(f, rsp) => {
                                    debug!("<i>{}</>: Result for function {} is '{:?}'", self.name, f, rsp);
                                    let _ = self.attribute_parser(rsp);
                                }
                                _ => {
                                    error!("<i>{}</>: unexpected Reading Device Identifiers (0x2B) result", self.name);
                                }
                            },
                            Err(e) => {
                                warn!("<i>{}</i>: read error during <green><i>Reading Device Identifiers (0x2B)</>, error: <b>{}</>", self.name, e);
                            }
                        },
                        Err(e) => {
                            warn!("<i>{}</i>: read timeout during <green><i>Reading Device Identifiers (0x2B)</>, error: <b>{}</>", self.name, e);
                        }
                    }

                    

                    let mut daily_yield_energy: Option<u32> = None;
                    loop {
                        if worker_cancel_flag.load(Ordering::SeqCst) {
                            debug!("<i>{}</>: Got terminate signal from main", self.name);
                            terminated = true;
                        }

                        if terminated
                            || stats_interval.elapsed()
                                > Duration::from_secs_f32(SUN2000_STATS_DUMP_INTERVAL_SECS)
                        {
                            stats_interval = Instant::now();
                            info!(
                                "<i>{}</>: 📊 inverter query statistics: ok: <b>{}</>, errors: <b>{}</>, daily energy yield: <b>{:.1} kWh</>",
                                self.name, self.poll_ok, self.poll_errors,
                                daily_yield_energy.unwrap_or_default() as f64 / 100.0,
                            );

                            if terminated {
                                break;
                            }
                        }

                        let now = chrono::Utc::now();
                        let mut start = now.with_second((now.second()/SUN2000_POLL_INTERVAL_SECS)*SUN2000_POLL_INTERVAL_SECS).unwrap().with_nanosecond(0).unwrap().signed_duration_since(now);

                        let period = chrono::Duration::seconds(SUN2000_POLL_INTERVAL_SECS.into()).to_std().unwrap();
                        while start < chrono::Duration::seconds(0) {
                            start = start.checked_add(&chrono::Duration::seconds(SUN2000_POLL_INTERVAL_SECS.into())).unwrap();
                        }
                        let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + start.to_std().unwrap(), period);    
                        interval.tick().await;

                        let mut device_status: Option<u16> = None;
                        let mut storage_status: Option<i16> = None;
                        let mut grid_code: Option<u16> = None;
                        let mut state_1: Option<u16> = None;
                        let mut state_2: Option<u16> = None;
                        let mut state_3: Option<u32> = None;
                        let mut alarm_1: Option<u16> = None;
                        let mut alarm_2: Option<u16> = None;
                        let mut alarm_3: Option<u16> = None;
                        let mut fault_code: Option<u16> = None;
                        
                        //obtaining all parameters from inverter

                        let (new_ctx, params, ms) =
                            self.read_params(ctx, &parameters, false).await?;
                        ctx = new_ctx;


                        for p in &params {
                            match p.value {
                                ParamKind::NumberU16(n) => match p.name.as_ref() {
                                    "fault_code" => match n {
                                        Some(fc) => {
                                            if fc != 0 {
                                                error!(
                                                    "<i>{}</>: inverter fault code is: <b><red>{:#08X}</>",
                                                    self.name, fc
                                                );
                                            }
                                            fault_code = n;
                                        }
                                        _ => {}
                                    },
                                    "device_status" => device_status = n,
                                    "grid_code" => grid_code = n,
                                    "state_1" => state_1 = n,
                                    "state_2" => state_2 = n,
                                    "alarm_1" => alarm_1 = n,
                                    "alarm_2" => alarm_2 = n,
                                    "alarm_3" => alarm_3 = n,
                                    _ => {}
                                },
                                ParamKind::NumberI16(n) => match p.name.as_ref() {
                                    "storage_status" => storage_status = n,
                                    _ => {}
                                },
                                ParamKind::NumberU32(n) => match p.name.as_ref() {
                                    "state_3" => state_3 = n,
                                    "daily_yield_energy" => daily_yield_energy = n,
                                    _ => {}
                                },
                                _ => {}
                            }
                        }

                        let param_count = parameters.iter().filter(|s| s.save_to_influx ||
                            s.name.starts_with("state_") ||
                            s.name.starts_with("alarm_") ||
                            s.name.ends_with("_status") ||
                            s.name.ends_with("_code")).count();
                        if params.len() != param_count {
                            error!("<i>{}</>: problem obtaining a complete parameter list (read: {}, expected: {}), reconnecting...", self.name, params.len(), param_count);
                            self.poll_errors = self.poll_errors + 1;
                            break;
                        } else {
                            self.poll_ok = self.poll_ok + 1;
                        }

                        let mut point = influxdb2::models::DataPoint::builder("inverter");

                        for p in &params {
                            if p.save_to_influx {
                                point = point.field(p.name.clone(), p.get_influx_value());
                            }
                        }

                        let mut points = vec![point.build()?];

                                        
                        //save query time                
                        points.push(influxdb2::models::DataPoint::builder("inverter_query_time")
                            .field("value", ms as i64)
                            .field("param_count", param_count as i64).build()?);
                                        

                        //setting new inverter state/alarm
                        let mut state_changes = HashMap::new();
                        state.set_new_status(
                            &self.name,
                            device_status,
                            storage_status,
                            grid_code,
                            state_1,
                            state_2,
                            state_3,
                            alarm_1,
                            alarm_2,
                            alarm_3,
                            fault_code,
                            &mut state_changes
                        );

                        if !state_changes.is_empty() {
                            let mut point = influxdb2::models::DataPoint::builder("inverter_status");
                            for (state_key, state_str) in state_changes.iter() {
                                point = point.field((*state_key).clone(), (*state_str).clone());
                            }
                            points.push(point.build()?);
                        }


                        if let (Some(influx_url),Some(influx_org),Some(influxdb_token),Some(influxdb_bucket)) = (&self.influxdb_url, &self.influxdb_org, &self.influxdb_token, &self.influxdb_bucket) {
                            let client = influxdb2::Client::new(influx_url, influx_org, influxdb_token);

                            let res = client.write(influxdb_bucket, stream::iter(points)).await;

                            match res {
                                Ok(msg) => {
                                    debug!("{}: influxdb write success: {:?}", &self.name, msg);
                                }
                                Err(e) => {
                                    error!("<i>{}</>: influxdb write error: <b>{:?}</>", &self.name, e);
                                }
                            }
                        }


                        //process obtained parameters
                        debug!("Query complete, dump results:");
                        for p in &params {
                            debug!(
                                "  {} ({:?}): {} {}",
                                p.name,
                                p.desc.clone().unwrap_or_default(),
                                p.get_text_value(),
                                p.unit.clone().unwrap_or_default()
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("<i>{}</>: connection error: <b>{}</>", self.name, e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }

        info!("{}: task stopped", self.name);
        Ok(())
    }
}
