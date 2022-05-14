
use simplelog::*;

use ::sun2000::*;

extern crate ini;
use self::ini::Ini;

use futures::future::join_all;
use humantime::format_duration;
use std::env;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use std::time::{Duration, Instant};
use tokio::task;
//use tokio_compat_02::FutureExt;


fn get_config_string(option_name: &str, section: Option<&str>) -> Option<String> {
    let conf = Ini::load_from_file("hard.conf").expect("Cannot open config file");
    conf.section(Some(section.unwrap_or("general").to_owned()))
        .and_then(|x| x.get(option_name).cloned())
}

fn get_config_bool(option_name: &str, section: Option<&str>) -> bool {
    let conf = Ini::load_from_file("hard.conf").expect("Cannot open config file");
    let value = conf
        .section(Some(section.unwrap_or("general").to_owned()))
        .and_then(|x| x.get(option_name).cloned());
    match value {
        Some(val) => match val.trim() {
            "yes" => true,
            "true" => true,
            "1" => true,
            _ => false,
        },
        _ => false,
    }
}



#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let started = Instant::now();
    logging::init(get_config_string("log", None));

    info!("🛡️ Welcome to hard (home automation rust-daemon)");

    //Ctrl-C / SIGTERM support
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    //common thread stuff
    let mut futures = vec![];
    let cancel_flag = Arc::new(AtomicBool::new(false));
    
    //sun2000 async task
    match get_config_string("host", Some("sun2000")) {
        Some(host) => {
            let worker_cancel_flag = cancel_flag.clone();
            let mut sun2000 = sun2000::Sun2000 {
                name: "sun2000".to_string(),
                host_port: host,
                poll_ok: 0,
                poll_errors: 0,
                influxdb_url: get_config_string("influxdb_url", None),
                influxdb_org: get_config_string("influxdb_org", None),
                influxdb_token: get_config_string("influxdb_token", None),
                influxdb_bucket: get_config_string("influxdb_bucket", None),
                mode_change_script: get_config_string("mode_change_script", Some("sun2000")),
                optimizers: get_config_bool("optimizers", Some("sun2000")),
                battery_installed: get_config_bool("battery_installed", Some("sun2000")),
                dongle_connection: get_config_bool("dongle_connection", Some("sun2000")),
            };
            let sun2000_future =
                task::spawn(async move { sun2000.worker(worker_cancel_flag).await });
            futures.push(sun2000_future);
        }
        _ => {}
    }


    debug!("Entering main loop...");
    loop {
        if !running.load(Ordering::SeqCst) {
            info!("🛑 Ctrl-C or SIGTERM signal detected, exiting...");
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    info!("🏁 Stopping all threads...");
    //inform all threads about termination
    cancel_flag.store(true, Ordering::SeqCst);
    //wait for tokio async tasks
    let _ = join_all(futures).await;

    info!(
        "🚩 hard terminated, daemon running time: {}",
        format_duration(started.elapsed()).to_string()
    );
}
