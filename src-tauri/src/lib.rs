mod peers;

use std::sync::{Arc, OnceLock};

use tauri::Manager;
use tracing::{info, Level};
use veilid_core::{
    api_startup_json, VeilidAPI, VeilidAPIError, VeilidAPIResult, VeilidConfigBlockStore,
    VeilidConfigDHT, VeilidConfigInner, VeilidConfigNetwork, VeilidConfigProtectedStore,
    VeilidConfigProtocol, VeilidConfigRPC, VeilidConfigRoutingTable, VeilidConfigTCP,
    VeilidConfigTLS, VeilidConfigTableStore, VeilidConfigUDP, VeilidUpdate,
};

use serde_json::to_string;

use peers::PeerState;

static MAIN_WINDOW: OnceLock<tauri::Window> = OnceLock::new();
static VEILID_API: OnceLock<VeilidAPI> = OnceLock::new();

#[tauri::command]
async fn connect_veilid() -> Result<(), VeilidAPIError> {
    if let Some(veilid) = VEILID_API.get() {
        veilid.attach().await?;
    }
    Ok(())
}

#[tauri::command]
async fn get_node_id() -> Result<String, VeilidAPIError> {
    match VEILID_API.get() {
        None => Ok("".into()),
        Some(veilid) => {
            let id = veilid
                .get_state()
                .await?
                .config
                .config
                .network
                .routing_table
                .node_id;
            let id = id.first().map_or("".into(), |ct| ct.to_string());
            Ok(id)
        }
    }
}

fn update_callback(change: VeilidUpdate) {
    if let Some(win) = MAIN_WINDOW.get() {
        match change {
            VeilidUpdate::AppMessage(msg) => {
                if let Ok(m) = std::str::from_utf8(msg.message()) {
                    info!("AppMessage event: {}", m);
                    let _ = win.emit("app-message", m.to_string());
                }
            }
            VeilidUpdate::Attachment(attachment) => {
                info!("Attachment event: {:?}", attachment);
                // Handle the attachment here
            }
            VeilidUpdate::Network(network) => {
                let network = *network;
                let result = PeerState::upsert(network.peers);

                match result {
                    Ok(changes) => {
                        let json = to_string(&changes).unwrap();
                        let _ = win.emit("peers", json);
                    }
                    Err(e) => {
                        info!("Error upserting peers: {:?}", e);
                    }
                }
            }
            VeilidUpdate::RouteChange(route_change) => {
                info!("RouteChange event: {:?}", route_change);
                // Handle the route change here
            }
            _ => {
                info!("Something else happened");
            }
        };
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![connect_veilid, get_node_id])
        .setup(|app| {
            info!("Setting up tauri app");
            tauri::async_runtime::block_on(async {
                let api = init_veilid().await.expect("failed to initialize Veilid");
                VEILID_API.get_or_init(|| api.clone());

                let main_window = MAIN_WINDOW.get_or_init(|| app.get_window("main").unwrap());
                main_window.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::Destroyed) {
                        let veilid = api.clone();
                        tauri::async_runtime::block_on(async {
                            veilid.shutdown().await;
                        });
                    }
                });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn init_veilid() -> VeilidAPIResult<VeilidAPI> {
    let config = VeilidConfigInner {
        program_name: "tauri-veilid-example".into(),
        protected_store: VeilidConfigProtectedStore {
            allow_insecure_fallback: true,
            always_use_insecure_storage: true,
            directory: "../.store/protected".into(),
            delete: false,
            ..Default::default()
        },
        block_store: VeilidConfigBlockStore {
            directory: "../.store/block".into(),
            delete: false,
        },
        table_store: VeilidConfigTableStore {
            directory: "../.store/table".into(),
            delete: false,
        },
        network: VeilidConfigNetwork {
            connection_initial_timeout_ms: 2000,
            connection_inactivity_timeout_ms: 60000,
            max_connections_per_ip4: 32,
            max_connections_per_ip6_prefix: 32,
            max_connections_per_ip6_prefix_size: 56,
            max_connection_frequency_per_min: 128,
            client_allowlist_timeout_ms: 300000,
            reverse_connection_receipt_time_ms: 5000,
            hole_punch_receipt_time_ms: 5000,
            routing_table: VeilidConfigRoutingTable {
                bootstrap: vec!["bootstrap.veilid.net".into()],
                limit_over_attached: 64,
                limit_fully_attached: 32,
                limit_attached_strong: 16,
                limit_attached_good: 8,
                limit_attached_weak: 4,
                ..Default::default()
            },
            rpc: VeilidConfigRPC {
                concurrency: 0,
                queue_size: 1024,
                max_timestamp_behind_ms: Some(10000),
                max_timestamp_ahead_ms: Some(10000),
                timeout_ms: 5000,
                max_route_hop_count: 4,
                default_route_hop_count: 1,
            },
            dht: VeilidConfigDHT {
                max_find_node_count: 20,
                resolve_node_timeout_ms: 10000,
                resolve_node_count: 1,
                resolve_node_fanout: 4,
                get_value_timeout_ms: 10000,
                get_value_count: 3,
                get_value_fanout: 4,
                set_value_timeout_ms: 10000,
                set_value_count: 5,
                set_value_fanout: 4,
                min_peer_count: 20,
                min_peer_refresh_time_ms: 60000,
                validate_dial_info_receipt_time_ms: 2000,
                ..Default::default()
            },
            upnp: true,
            detect_address_changes: true,
            restricted_nat_retries: 0,
            tls: VeilidConfigTLS {
                certificate_path: "server.pem".into(),
                private_key_path: "key.pem".into(),
                connection_initial_timeout_ms: 2000,
            },
            protocol: VeilidConfigProtocol {
                udp: VeilidConfigUDP {
                    enabled: true,
                    ..Default::default()
                },
                tcp: VeilidConfigTCP {
                    connect: true,
                    listen: true,
                    max_connections: 32,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let json_config = serde_json::to_string(&config).unwrap();
    api_startup_json(Arc::new(update_callback), json_config).await
}
