// james was here
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use rosc::{encoder, OscMessage, OscPacket, OscType};
use std::collections::HashMap;
use std::net::{SocketAddrV4, UdpSocket};
use std::sync::{Arc, Mutex};
use uuid::{uuid, Uuid};

const TP9: Uuid = uuid!("273e0003-4c4d-454d-96be-f03bac821358");
const AF7: Uuid = uuid!("273e0004-4c4d-454d-96be-f03bac821358");
const AF8: Uuid = uuid!("273e0005-4c4d-454d-96be-f03bac821358");
const TP10: Uuid = uuid!("273e0006-4c4d-454d-96be-f03bac821358");
const AUX: Uuid = uuid!("273e0007-4c4d-454d-96be-f03bac821358");

const ALL_SENSORS_ORDERED: [Uuid; 5] = [TP9, AF7, AF8, TP10, AUX];
const OSC_ADDRESS_PATH: &str = "/eeg";

fn stream_lsl(stream_name: String) {
    println!("LSL not implemented... yet... Would stream {}", stream_name);
}

#[tauri::command]
fn stream_osc(ip: String, port_str: String) {
    println!("Starting OSC stream to {}:{}", ip, port_str);

    let target_addr_str = format!("{}:{}", ip, port_str);
    let target_socket_addr: SocketAddrV4 = match target_addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Invalid IP:Port format '{}': {}", target_addr_str, e);
            return;
        }
    };

    let send_socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => Arc::new(socket),
        Err(e) => {
            eprintln!("Failed to bind UDP send socket: {}", e);
            return;
        }
    };

    let latest_sensor_data = Arc::new(Mutex::new(HashMap::<Uuid, Vec<u8>>::new()));

    tauri::async_runtime::block_on(async move {
        let ble_handler = match tauri_plugin_blec::get_handler() {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Failed to get BLE handler: {:?}", e);
                return;
            }
        };

        for &sensor_uuid in ALL_SENSORS_ORDERED.iter() {
            let data_store_clone = Arc::clone(&latest_sensor_data);
            let socket_clone = Arc::clone(&send_socket);
            let osc_target_addr_clone = target_socket_addr;

            let subscription_result = ble_handler
                .subscribe(sensor_uuid, move |new_data: Vec<u8>| {
                    let mut store_guard = data_store_clone
                        .lock()
                        .expect("Failed to lock data store for write");
                    store_guard.insert(sensor_uuid, new_data);
                    drop(store_guard);

                    let mut osc_args: Vec<OscType> = Vec::with_capacity(ALL_SENSORS_ORDERED.len());

                    let current_store_values = data_store_clone
                        .lock()
                        .expect("Failed to lock data store for read");
                    for &uuid_for_osc in ALL_SENSORS_ORDERED.iter() {
                        let blob_data = current_store_values
                            .get(&uuid_for_osc)
                            .cloned()
                            .unwrap_or_else(Vec::new);
                        osc_args.push(OscType::Blob(blob_data));
                    }
                    drop(current_store_values);

                    let osc_packet = OscPacket::Message(OscMessage {
                        addr: OSC_ADDRESS_PATH.to_string(),
                        args: osc_args,
                    });

                    match encoder::encode(&osc_packet) {
                        Ok(encoded_msg_buf) => {
                            if let Err(e) =
                                socket_clone.send_to(&encoded_msg_buf, osc_target_addr_clone)
                            {
                                eprintln!("Error sending OSC packet: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error encoding OSC packet: {}", e);
                        }
                    }
                })
                .await;

            match subscription_result {
                Ok(_) => println!("Successfully subscribed to UUID: {}", sensor_uuid),
                Err(e) => eprintln!("Failed to subscribe to UUID {}: {:?}", sensor_uuid, e),
            }
        }
        println!("All BLE subscriptions initiated. OSC streaming should be active.");
    });

    println!("stream_osc function has finished its setup. Background streaming will continue via callbacks.");
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_blec::init())
        .invoke_handler(tauri::generate_handler![stream_osc])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
