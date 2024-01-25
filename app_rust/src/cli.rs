use clap::Parser;
use colored::Colorize;
use std::num::ParseIntError;

mod commapi;
mod passthru;

use crate::{
    commapi::{comm_api::ComServer, protocols::uds::UDSECU},
    commapi::{comm_api::ISO15765Config, protocols::DiagCfg},
    commapi::peak_can_api::PeakCanAPI,
    commapi::iface,
    commapi::comm_api::CanFrame,
  };

// CLI parser
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Diagnostic modes: CANTRACER, UDS 
    #[arg(short, long, default_value_t = String::from("CANTRACER"), ignore_case = true)]
    mode: String,

    /// UDS Service ID
    #[arg(short, long, ignore_case = true)]
    SID: Option<String>,

    /// UDS Data ID
    #[arg(short, long, ignore_case = true)]
    DID: Option<String>,
}

fn is_hex(value: &String) -> bool {
    if value.starts_with("0x") {
        // Check if the remaining characters are valid hexadecimal digits
        if value[2..].chars().all(|c| c.is_digit(16)) {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

fn to_hex(value: &String) -> Vec<u8> {
    // assume value is conertible to hex
    let value = if value.starts_with("0x") {
        &value[2..]
    } else {
        value
    };

    // Parse each pair of characters as a u8
    let bytes: Vec<u8> = (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16))
        .collect::<Result<Vec<u8>, ParseIntError>>()
        .unwrap();

    bytes
}

fn print_error(r#type: u8, value: &String) {
    let mut error_message = String::from("");
    if (r#type == 1 && value.is_empty()) {
        error_message = String::from("error: empty value for '--sid <SID>'");        
    } else if (r#type == 1 && !value.is_empty()) {
        error_message = format!("error: invalid value '{}' for '--sid <SID>': expect hex format (e.g 0x11 for ECU RESET)", value);
    } else if (r#type == 2 && !value.is_empty()) {
        error_message = format!("error: invalid value '{}' for '--did <DID>': expect hex format (e.g 0xF18C for ECU Serial#)", value);
    } else if (r#type == 2 && value.is_empty()) {
        error_message = String::from("error: empty value for '--did <DID>'");        
    }

    // Split the error message into parts based on the custom markers ('error:' and 'ah')
    let parts: Vec<&str> = error_message.split([' '].as_ref()).collect();

    // Print each part with the appropriate color
    for part in parts {
        if part == "error:" {
            print!("{}", part.bright_red());
        } else if part == format!("'{}'", value) {
            print!("{}", part.bright_yellow());
        } else {
            print!("{}", part);
        }
        print!(" ");
    } 
    println!();
}  

fn start_uds(sid: Vec<u8>, did: Vec<u8>) {

    //println!("sid: {:?} did: {:?}", sid, did);

    let mut dev = PeakCanAPI::new(String::from("PeakCan"));

    if let Err(e) = dev.open_device() {
        println!("CAN Init: Fail {:?}", e);
        return;
    } else {
        println!("CAN Init: Success");
    }

    //Start ISO-TP UDS session with IC
    UDSECU::start_client_and_send_one_request(
        &dev.clone_box(),
        iface::InterfaceType::IsoTp,
        iface::InterfaceConfig::from_iso15765(ISO15765Config {
            baud: 0x001C,
            send_id: 0x784,
            recv_id: 0x7F0,
            block_size: 8,
            sep_time: 20,
            use_ext_isotp: false,
            use_ext_can: false,
        }),
        None,
        DiagCfg { // Not used
            send_id: 0,
            recv_id: 0,
            global_id: None,
        },
        &sid,
        &did,
    );
}

fn start_can_tracer() {
    let mut dev = PeakCanAPI::new(String::from("PeakCan"));

    if let Err(e) = dev.open_device() {
        println!("CAN Init: Fail {:?}", e);
        return;
    } else {
        println!("CAN Init: Success");
    }

    if let Err(e) = dev.open_can_interface(0x001C, false) {
        println!("CAN Setup: Fail {:?}", e);
        return;
    } else {
        println!("CAN Setup: Success");
    }

    while true {
        match dev.read_can_packets(2000, 1) {
            Ok(can_frames) => {
                for can_frame in can_frames {
                    println!("{}", can_frame); // Print each CanFrame
                }
            }
            Err(e) => {
                //println!("CAN Read: Fail {:?}", e);
                std::thread::sleep(std::time::Duration::from_millis(1000));

            //return;
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    if (args.mode == "CANTRACER") {
        // Start can tracer
        start_can_tracer();
    } else if (args.mode == "UDS") {
        // Parse SID
        match args.SID {
            Some(service) => {
                if !is_hex(&service) {
                    print_error(1, &service);
                }

                // Parse DID
                let mut did = String::from("");
                match args.DID {
                    Some(data) => {
                        if !is_hex(&data) {
                            print_error(2, &data);
                        }
                        did = data;
                    }
                    None => {// Okay for empty DID
                    }
                }
                // Start UDS with service data
                start_uds(to_hex(&service), to_hex(&did));
            }
            None => {
                print_error(1, &String::from(""));
            }
        }
    }
}