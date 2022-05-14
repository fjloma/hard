use serde::{Serialize, Deserialize};
use super::params::*;
use super::Result;
use simplelog::*;
use std::fs::OpenOptions;
use chrono::prelude::*;
use tokio::io::{AsyncWriteExt};
use tokio::fs::File;
use is_sorted::*;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct DiskParamValue {
    pub addr_offset: u16,
    pub value: ParamKind
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct DiskRecord {
    pub time_offset_from_day_start_millis: u32,
    pub base_address: u16,
    pub params: Vec<DiskParamValue>
}

pub async fn log_params(date: chrono::DateTime<chrono::Utc>, parameters: &Vec<Parameter>) -> Result<()>  {

    if parameters.len() == 0 {
        return Ok(())
    }

    let path = format!("sun2000_{}_{}_{}.bin", &date.year(), &date.month(), &date.day());

    let open = OpenOptions::new().append(true).create(true).open(&path);

    match open {
        Err(e) => {
            error!("ERROR OPENNING {} {}", &path, e);
            return Ok(())
        }
        _ => {}
    }


    let start_of_day: chrono::DateTime<chrono::Utc> = date.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();

    let time_offset = date.timestamp_millis() - start_of_day.timestamp_millis();

    debug!("{:?} start of day:{:?} time_offset: {:?}", date, start_of_day, time_offset);

    let mut params_sorted;
    let parameters = if !IsSorted::is_sorted_by_key(&mut parameters.iter(), |v| v.reg_address) {
        info!("SORTING!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        params_sorted = parameters.clone();
        params_sorted.sort_by(|a,b| a.reg_address.partial_cmp(&b.reg_address).unwrap());
        &params_sorted
    } else {
        parameters
    };

    let base_address = parameters[0].reg_address;


    let mut params = vec!();
    let mut prev_addr = base_address;
    for p in parameters {

        params.push(DiskParamValue{addr_offset: p.reg_address - prev_addr, value: p.value.clone()});
        prev_addr = p.reg_address;
    }
    

    let record: DiskRecord = DiskRecord {
        time_offset_from_day_start_millis: time_offset as u32,
        base_address: base_address,
        params: params
    };

    let coded = postcard::to_allocvec(&record).unwrap();
    let len_header: u16 = coded.len() as u16;
    
    debug!("ENCODED {} into {} bytes", parameters.len(), coded.len());

    tokio::spawn(async move {
        let mut f: tokio::fs::File = File::from_std(open.unwrap());
        f.write(&len_header.to_le_bytes()).await;
        f.write(&coded).await;
    });


    Ok(())
}
