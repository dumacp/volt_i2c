use std::error::Error;
use std::time;
use volt_i2c::adc::{FlagRegister, ADC};
use volt_i2c::logs;
// use std::sync::{Arc};
// use std::sync::atomic::{AtomicBool, Ordering};
use clap::{self, App, Arg};
use paho_mqtt as mqtt;
use std::process;
use tokio;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::time::Duration;
use log::{debug, error, info, warn};

const APPNAME: &'static str = "volt";
const VERSION: &'static str = "1.0.3";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("volt")
        .version("1.0")
        .author("soporte <soporte@nebulae.com.co>")
        .about("ADC sensor")
        .arg(
            Arg::with_name("alert-under-range")
                .short("u")
                .long("underRange")
                .value_name("under_range")
                .help("Set alert under range value")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("alert-over-range")
                .short("o")
                .long("overRange")
                .value_name("over_range")
                .help("Set alert over range value")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("timeout")
                .help("Set timeout value in secs")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("logSdt")
                .short("l")
                .long("logStd")
                .help("send logs to stderr")
        )
        .arg(
            Arg::with_name("version")
                .short("version")
                .long("version")
                .help("show version"),
        )
        .get_matches();

    let logstd = args.is_present("logStd");
    let version = args.is_present("version");
    if version {
        println!("version: {}", VERSION);
        process::exit(-2);
    }
    info!(r#"runnin "{}", version "{}""#, APPNAME, VERSION);

    logs::init_std_log(logstd, APPNAME)?;

    let timeout: u64 = clap::value_t!(args.value_of("timeout"), u64).unwrap_or(30);
    let over_range: f32 = clap::value_t!(args.value_of("alert-over-range"), f32).unwrap_or(50.0);
    let under_range: f32 = clap::value_t!(args.value_of("alert-under-range"), f32).unwrap_or(9.5);

    println!("alert over range: {}", over_range);
    println!("alert under range: {}", under_range);

    let mut term = signal(SignalKind::terminate())?;
    let mut inte = signal(SignalKind::interrupt())?;

    const LOWEST_VALUE: f32 = 9.5;
    const HIHGEST_VALUE: f32 = 50.0;

    #[derive(Debug)]
    struct Values {
        current: f32,
        min: f32,
        max: f32,
        alert_under: bool,
        alert_over: bool,
    }

    // let term = Arc::new(AtomicBool::new(false));
    // signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    let flags = FlagRegister::AlertFlagEnable as u8
        | FlagRegister::AlertPINEnable as u8
        | FlagRegister::Tx32 as u8;

    let mut dev = ADC::new()?;

    let result = dev.read_register_byte(0x00).unwrap();
    println!("register: {}", result);

    dev.set_conf_register(flags)?;
    dev.set_alert_over_range(over_range)?;
    dev.set_alert_under_range(under_range)?;
    dev.set_alert_hysteresis(0x005D)?;

    let (result, alert) = dev.read_value()?;
    println!("volt now: {}", result);
    if alert {
        let (over, under) = dev.read_alert()?;
        println!("alert?: over: {}, under {}", over, under);
    }
    let min = dev.read_min_value()?;
    println!("min: {}", min);
    let mut min_old = min;
    dev.write_min_value(50.0)?;
    let max = dev.read_max_value()?;
    println!("max: {}", max);
    let mut max_old = max;
    dev.write_max_value(1.0)?;

    let register = dev.read_register_word(0x04)?;
    println!("register 0x04: {:#X}", register);
    let register = dev.read_register_word(0x03)?;
    println!("register 0x03: {:#X}", register);
    let register = dev.read_register_word(0x05)?;
    println!("register 0x05: {:#X}", register);
    let register = dev.read_register_byte(0x01)?;
    println!("register 0x01: {:#X}", register);
    let register = dev.read_register_byte(0x02)?;
    println!("register 0x02: {:#X}", register);

    // Create a client & define connect options
    let cli = mqtt::AsyncClient::new("tcp://localhost:1883").unwrap_or_else(|err| {
        error!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptions::new();

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(conn_opts).wait() {
        error!("Unable to connect: {:?}", e);
        process::exit(1);
    }

    let (tx, mut rx) = mpsc::channel(32);
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    tokio::spawn(async move {
        let mut min_old = 0.0;
        let mut max_old = 0.0;
        let mut current_old = 0.0;
        loop {
            tokio::select! {
                _ = term.recv() => {
                    let _ = tx.closed();
                    error!("Received SIGTERM kill signal. Exiting...");
                    return ()
                },
                _ = inte.recv() => {
                    let _ = tx.closed();
                    error!("Received SIGINT kill signal. Exiting...");
                    return ()
                },
                _ = tick.tick() => {
                    let (current, alert) = dev.read_value().unwrap_or_else(|error| {
                        warn!("ADC read_value error: {}", error);
                        (current_old, false)
                    });
                    let mut alert_under = false;
                    let mut alert_over = false;
                    if alert {
                        let (alert_over_t, alert_under_t) = dev.read_alert().unwrap_or_else(|error| {
                            warn!("ADC read_value error: {}", error);
                            (false, false)
                        });
                        alert_under = alert_under_t;
                        alert_over = alert_over_t;
                    }
                    let min = dev.read_min_value().unwrap_or_else(|error| {
                        warn!("ADC read_min_value error: {}", error);
                        min_old
                    });

                    let max = dev.read_max_value().unwrap_or_else(|error| {
                        warn!("ADC read_max_value error: {}", error);
                        max_old
                    });

                    let value = Values{
                        current: current,
                        min: min,
                        max: max,
                        alert_over: alert_over,
                        alert_under: alert_under,
                    };
                    if let Err(error) = tx.send(value).await {
                        error!("sending error: {}", error);
                        return ()
                    }

                    // if alter_hold is enables comment outcarg
                    // if alert {
                    //     dev.clear_alerts().unwrap_or_else(|error| {
                    //         warn!("ADC read_max_value error: {}", error);
                    //         ()
                    //     });
                    // }
                    if min < LOWEST_VALUE && current > LOWEST_VALUE {
                        dev.write_min_value(50.0).unwrap_or_else(|error| {
                            warn!("ADC write_min_value error: {}", error);
                            ()
                        });
                    }
                    if max > HIHGEST_VALUE && current < HIHGEST_VALUE {
                        dev.write_max_value(1.0).unwrap_or_else(|error| {
                            warn!("ADC write_max_value error: {}", error);
                            ()
                        });
                    }

                    min_old = min;
                    max_old = max;
                    current_old = current;
                },
            }

            // thread::sleep(time::Duration::from_secs(1));
        }
    });

    let mut old_time = time::SystemTime::now();
    while let Some(received) = rx.recv().await {
        let nsec = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(n) => n.as_secs_f64(),
            Err(_) => {
                warn!("SystemTime before UNIX EPOCH!");
                0.0
            }
        };

        let elapse = old_time.elapsed();
        match elapse {
            Ok(value) => {
                if value > time::Duration::from_secs(timeout) {
                    debug!("1970-01-01 00:00:00 UTC was {} seconds ago!", nsec);
                    old_time = time::SystemTime::now();
                    debug!("Publishing a message on the 'EVENTS/volt' topic");
                    debug!("Got: {:?}", received);
                    //let msg = mqtt::Message::new("test", "Hello world!", 0);
                    let msg = mqtt::Message::new(
                        "EVENTS/volt",
                        format!(
                            r#"{{"timeStamp": {}, "value": {}, "type": "current_volt"}}"#,
                            nsec, received.current
                        ),
                        0,
                    );
                    let tok = cli.publish(msg);
                    if let Err(e) = tok.wait() {
                        error!("Error sending message: {:?}", e);
                    }
                }
            }
            Err(_) => {}
        };

        if received.min < LOWEST_VALUE {
            warn!("alert_volt -> {}", received.min);
            let msg = mqtt::Message::new(
                "EVENTS/volt",
                format!(
                    r#"{{"timeStamp": {}, "value": {}, "type": "alert_volt"}}"#,
                    nsec, received.min
                ),
                0,
            );
            let tok = cli.publish(msg);
            if let Err(e) = tok.wait() {
                error!("Error sending message: {:?}", e);
            }
        } else if received.max > HIHGEST_VALUE {
            warn!("alert_volt -> {}", received.max);
            let msg = mqtt::Message::new(
                "EVENTS/volt",
                format!(
                    r#"{{"timeStamp": {}, "value": {}", "type": "alert_volt"}}"#,
                    nsec, received.max
                ),
                0,
            );
            let tok = cli.publish(msg);
            if let Err(e) = tok.wait() {
                error!("Error sending message: {:?}", e);
            }
        } else if min_old > received.min {
            warn!("lowest_volt -> {}", received.min);
            min_old = received.min;
            let msg = mqtt::Message::new(
                "EVENTS/volt",
                format!(
                    r#"{{"timeStamp": {}, "value": {}, "type": "lowest_volt"}}"#,
                    nsec, received.min
                ),
                0,
            );
            let tok = cli.publish(msg);
            if let Err(e) = tok.wait() {
                error!("Error sending message: {:?}", e);
            }
        } else if max_old < received.max {
            warn!("highest_volt -> {}", received.max);
            max_old = received.max;
            let msg = mqtt::Message::new(
                "EVENTS/volt",
                format!(
                    r#"{{"timeStamp": {}, "value": {}, "type": "highest_volt"}}"#,
                    nsec, received.max
                ),
                0,
            );
            let tok = cli.publish(msg);
            if let Err(e) = tok.wait() {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    println!("Received kill signal. Exiting...");

    // Disconnect from the broker
    let tok = cli.disconnect(None);
    tok.wait()?;
    Ok(())
}
