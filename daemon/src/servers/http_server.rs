use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::dev::ServerHandle;
use actix_web::http::header::ContentType;
use actix_web::middleware::Condition;
use actix_web::web::Data;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, get, post, web};
use actix_ws::{AggregatedMessage, CloseCode, CloseReason, Session};
use anyhow::{Result, anyhow};
use enum_map::EnumMap;
use futures_util::StreamExt;
use include_dir::{Dir, include_dir};
use jsonpath_rust::JsonPath;
use log::{debug, error, info, warn};
use mime_guess::MimeGuess;
use mime_guess::mime::IMAGE_PNG;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Component, PathBuf};
use std::{env, fs};
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::oneshot::Sender;
use tokio::sync::{RwLock, oneshot};

use crate::PatchEvent;
use crate::SettingsHandle;
use crate::button_actions::ButtonLayer;
use crate::files::{FilePaths, find_file_in_path};
use goxlr_ipc::{
    DaemonRequest, DaemonResponse, DaemonStatus, HttpSettings, Scribble, WebsocketRequest,
    WebsocketResponse,
};
use goxlr_scribbles::get_scribble_png;
use goxlr_types::FaderName;

use crate::primary_worker::{DeviceCommand, DeviceSender};
use crate::servers::server_packet::handle_packet;

const WEB_CONTENT: Dir = include_dir!("./daemon/web-content/");

struct AppData {
    usb_tx: DeviceSender,
    broadcast_tx: BroadcastSender<PatchEvent>,
    file_paths: FilePaths,
    settings_handle: SettingsHandle,

    scribble_state: EnumMap<FaderName, ScribbleState>,
}

#[derive(Debug, Default)]
struct ScribbleState {
    scribble_config: Option<Scribble>,
    png_data: Option<Vec<u8>>,
}

pub async fn spawn_http_server(
    usb_tx: DeviceSender,
    handle_tx: Sender<Result<Option<ServerHandle>>>,
    broadcast_tx: tokio::sync::broadcast::Sender<PatchEvent>,
    settings: HttpSettings,
    file_paths: FilePaths,
    settings_handle: SettingsHandle,
) {
    // Create the AppData ONCE, outside the closure
    let app_data = Data::new(RwLock::new(AppData {
        broadcast_tx: broadcast_tx.clone(),
        usb_tx: usb_tx.clone(),
        file_paths: file_paths.clone(),
        settings_handle,
        scribble_state: EnumMap::default(),
    }));

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://127.0.0.1")
                    || origin.as_bytes().starts_with(b"http://localhost")
            })
            .allow_any_method()
            .allow_any_header()
            .max_age(300);
        App::new()
            .wrap(Condition::new(settings.cors_enabled, cors))
            .app_data(app_data.clone())
            .service(websocket)
            .service(execute_command)
            .service(get_devices)
            .service(get_sample)
            .service(get_scribble)
            .service(get_path)
            .service(upload_firmware)
            .service(get_button_layer)
            .service(set_button_layer)
            .service(test_button_action)
            .service(get_application_controls)
            .service(set_application_controls)
            .service(get_diagnostics)
            .service(check_app_update)
            .service(install_app_update)
            .service(export_diagnostics_log)
            .default_service(web::to(default))
    })
    .bind((settings.bind_address.clone(), settings.port));

    if let Err(e) = server {
        // Log the Error Message..
        warn!("Unable to Start HTTP Server: {}", e);

        // Let 'Upstream' know an error has occurred
        let _ = handle_tx.send(Err(anyhow!(e)));

        // Give up :D
        return;
    }

    // Run the server..
    let server = server.unwrap().run();
    info!(
        "Started GoXLR configuration interface at http://{}:{}/",
        settings.bind_address.as_str(),
        settings.port,
    );

    // Let upstream know we're running...
    let _ = handle_tx.send(Ok(Some(server.handle())));

    // Wait for the server to exit with its reason
    let result = server.await;
    if result.is_err() {
        error!("HTTP Server Stopped with Error: {}", result.err().unwrap());
        return;
    }

    info!("HTTP Server Stopped.");
}

#[derive(Serialize)]
struct WsResponse(WebsocketResponse);

#[derive(Serialize)]
struct ApplicationControlsResponse {
    applications: Vec<crate::button_actions::ApplicationBinding>,
    running_applications: Vec<crate::button_actions::RunningApplication>,
    runtime: crate::button_actions::ApplicationModeStatus,
    discovery_available: bool,
    routes: Vec<crate::application_audio::ApplicationRouteStatus>,
}

#[get("/api/application-controls/{serial}")]
async fn get_application_controls(
    req: HttpRequest,
    serial: web::Path<String>,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    if !local_settings_request(&req) {
        return HttpResponse::Forbidden().finish();
    }
    let settings = app_data.read().await.settings_handle.clone();
    let applications = settings.get_application_bindings(&serial).await;
    let runtime = settings.get_application_runtime(&serial).await;
    let running = crate::application_audio::running_applications().await;
    let (running_applications, discovery_available) = match running {
        Ok(applications) => (applications, true),
        Err(error) => {
            warn!("Unable to discover Windows audio sessions: {error}");
            (Vec::new(), false)
        }
    };
    HttpResponse::Ok().json(ApplicationControlsResponse {
        routes: settings.get_application_routes(&serial).await,
        applications,
        running_applications,
        runtime,
        discovery_available,
    })
}

#[post("/api/application-controls/{serial}")]
async fn set_application_controls(
    req: HttpRequest,
    serial: web::Path<String>,
    applications: web::Json<Vec<crate::button_actions::ApplicationBinding>>,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    if !local_settings_request(&req) {
        return HttpResponse::Forbidden().finish();
    }
    let status = match get_status(app_data.clone()).await {
        Ok(status) => status,
        Err(error) => return HttpResponse::ServiceUnavailable().body(error.to_string()),
    };
    if !status
        .mixers
        .get(serial.as_str())
        .is_some_and(|mixer| mixer.hardware.device_type == goxlr_types::DeviceType::Mini)
    {
        return HttpResponse::BadRequest().body("Liga o GoXLR Mini para guardar as aplicações.");
    }
    let settings = app_data.read().await.settings_handle.clone();
    match settings
        .set_application_bindings(serial.into_inner(), applications.into_inner())
        .await
    {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"saved":true})),
        Err(error) => HttpResponse::BadRequest().body(error.to_string()),
    }
}

#[derive(Serialize)]
struct DiagnosticEvent {
    time: String,
    kind: &'static str,
    message: String,
}

#[derive(Serialize)]
struct DiagnosticSummary {
    mini_connected: Option<bool>,
    last_usb_issue: Option<DiagnosticEvent>,
    recovery: Option<DiagnosticEvent>,
    recovery_seconds: Option<u64>,
    events: Vec<DiagnosticEvent>,
    log_available: bool,
}

fn is_usb_issue(line: &str) -> bool {
    line.contains("TSTATUS_")
        || line.contains("Timeout Awaiting Response")
        || line.contains("No USB response")
        || line.contains("USB response channel has closed")
        || line.contains("USB event handle invalidated")
        || line.contains("Error Reading Event")
        || line.contains("USB session has ended")
        || (line.contains("USB command") && line.contains(" failed;"))
        || (line.contains("USB response for") && line.contains(" failed;"))
}

fn diagnostic_recovery(line: &str) -> Option<&'static str> {
    if line.contains("USB protocol counter resynchronized") {
        Some("Leitura recuperada após sincronizar a comunicação USB")
    } else if line.contains("] GoXLR ready, profiles applied in ") {
        Some("Mini ligado e perfil aplicado")
    } else {
        None
    }
}

fn diagnostic_event(line: &str, kind: &'static str) -> DiagnosticEvent {
    let time = line
        .get(..8)
        .filter(|time| time.as_bytes().get(2) == Some(&b':'))
        .unwrap_or("");
    let message = line
        .split_once("] ")
        .map(|(_, message)| message)
        .unwrap_or(line)
        .trim();
    DiagnosticEvent {
        time: time.to_owned(),
        kind,
        message: message.to_owned(),
    }
}

fn diagnostic_elapsed_seconds(start: &str, end: &str) -> Option<u64> {
    let parse = |time: &str| {
        let mut parts = time.split(':').map(str::parse::<u64>);
        Some(parts.next()?.ok()? * 3600 + parts.next()?.ok()? * 60 + parts.next()?.ok()?)
    };
    let start = parse(start)?;
    let end = parse(end)?;
    let seconds = if end >= start {
        end - start
    } else {
        end + 24 * 3600 - start
    };
    (seconds <= 30 * 60).then_some(seconds)
}

#[get("/api/diagnostics")]
async fn get_diagnostics(req: HttpRequest, app_data: Data<RwLock<AppData>>) -> HttpResponse {
    if !local_settings_request(&req) {
        return HttpResponse::Forbidden().finish();
    }
    let settings = app_data.read().await.settings_handle.clone();
    let log_directory = settings.get_log_directory().await;
    let log = fs::read_to_string(log_directory.join("goxlr-daemon.log")).ok();
    let lines: Vec<&str> = log.as_deref().unwrap_or_default().lines().collect();
    let issue_index = lines.iter().rposition(|line| is_usb_issue(line));
    let last_usb_issue = issue_index.map(|index| diagnostic_event(lines[index], "issue"));
    let recovery_index = issue_index.and_then(|issue| {
        lines
            .iter()
            .enumerate()
            .skip(issue + 1)
            .find_map(|(index, line)| diagnostic_recovery(line).map(|_| index))
    });
    let recovery = recovery_index.map(|index| {
        diagnostic_event(
            lines[index],
            diagnostic_recovery(lines[index]).unwrap_or("recovery"),
        )
    });
    let recovery_seconds = issue_index
        .zip(recovery_index)
        .and_then(|(issue, recovery)| {
            diagnostic_elapsed_seconds(
                &diagnostic_event(lines[issue], "issue").time,
                &diagnostic_event(lines[recovery], "recovery").time,
            )
        });
    let events = lines
        .iter()
        .filter_map(|line| {
            if is_usb_issue(line) {
                Some(diagnostic_event(line, "issue"))
            } else if line.contains("] Device Disconnected") {
                Some(diagnostic_event(line, "disconnect"))
            } else {
                diagnostic_recovery(line).map(|kind| diagnostic_event(line, kind))
            }
        })
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let status = get_status(app_data).await;
    let mini_connected = status.ok().map(|status| {
        status
            .mixers
            .values()
            .any(|mixer| mixer.hardware.device_type == goxlr_types::DeviceType::Mini)
    });
    HttpResponse::Ok().json(DiagnosticSummary {
        mini_connected,
        last_usb_issue,
        recovery,
        recovery_seconds,
        events,
        log_available: log.is_some(),
    })
}

#[get("/api/diagnostics/log")]
async fn export_diagnostics_log(req: HttpRequest, app_data: Data<RwLock<AppData>>) -> HttpResponse {
    if !local_settings_request(&req) {
        return HttpResponse::Forbidden().finish();
    }
    let settings = app_data.read().await.settings_handle.clone();
    let log_directory = settings.get_log_directory().await;
    match fs::read(log_directory.join("goxlr-daemon.log")) {
        Ok(log) => HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .insert_header((
                "Content-Disposition",
                "attachment; filename=\"goxlr-daemon.log\"",
            ))
            .body(log),
        Err(_) => HttpResponse::NotFound().body("O registo atual ainda não está disponível."),
    }
}

fn local_settings_request(req: &HttpRequest) -> bool {
    let local_peer = req.peer_addr().is_some_and(|peer| peer.ip().is_loopback());
    let host = req.connection_info().host().to_owned();
    let local_host = format!("http://{host}")
        .parse::<actix_web::http::Uri>()
        .ok()
        .and_then(|uri| uri.host().map(str::to_owned))
        .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]"));
    let same_origin = req.headers().get("Origin").is_none_or(|origin| {
        origin
            .to_str()
            .is_ok_and(|origin| origin == format!("http://{host}"))
    });
    local_peer && local_host && same_origin
}

#[get("/api/button-actions/{serial}")]
async fn get_button_layer(
    req: HttpRequest,
    serial: web::Path<String>,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    if !local_settings_request(&req) {
        return HttpResponse::Forbidden()
            .body("Abre as ações neste computador através de localhost.");
    }
    let settings = app_data.read().await.settings_handle.clone();
    let mut layer = serde_json::to_value(settings.get_button_layer(&serial).await).unwrap();
    layer["runtime"] = serde_json::to_value(settings.get_button_runtime(&serial).await).unwrap();
    layer["last_result"] = serde_json::to_value(settings.get_action_report(&serial).await).unwrap();
    HttpResponse::Ok().json(layer)
}

#[post("/api/button-actions/{serial}")]
async fn set_button_layer(
    req: HttpRequest,
    serial: web::Path<String>,
    layer: web::Json<ButtonLayer>,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    if !local_settings_request(&req) {
        return HttpResponse::Forbidden()
            .body("Abre as ações neste computador através de localhost.");
    }
    if let Err(error) = layer.validate() {
        return HttpResponse::BadRequest().body(error.to_string());
    }
    let status = match get_status(app_data.clone()).await {
        Ok(status) => status,
        Err(error) => return HttpResponse::ServiceUnavailable().body(error.to_string()),
    };
    if !status
        .mixers
        .get(serial.as_str())
        .is_some_and(|mixer| mixer.hardware.device_type == goxlr_types::DeviceType::Mini)
    {
        return HttpResponse::BadRequest().body("Liga o GoXLR Mini para guardar as ações.");
    }
    for action in layer.pages.iter().flat_map(|page| &page.actions) {
        if let crate::button_actions::ButtonAction::LoadProfile { profile } = action
            && !status.files.profiles.contains(profile)
        {
            return HttpResponse::BadRequest()
                .body("O perfil escolhido já não está disponível. Escolhe outro perfil.");
        }
    }
    let settings = app_data.read().await.settings_handle.clone();
    match settings
        .set_button_layer(serial.into_inner(), layer.into_inner())
        .await
    {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"saved": true})),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}

#[post("/api/button-actions/{serial}/test")]
async fn test_button_action(
    req: HttpRequest,
    serial: web::Path<String>,
    action: web::Json<crate::button_actions::ButtonAction>,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    if !local_settings_request(&req) {
        return HttpResponse::Forbidden().finish();
    }
    if let Err(error) = action.validate() {
        return HttpResponse::BadRequest().body(error.to_string());
    }
    let tx = app_data.read().await.usb_tx.clone();
    let (sender, receiver) = oneshot::channel();
    if tx
        .send(DeviceCommand::RunButtonAction(
            serial.into_inner(),
            action.into_inner(),
            sender,
        ))
        .await
        .is_err()
    {
        return HttpResponse::ServiceUnavailable().body("O serviço não respondeu.");
    }
    match tokio::time::timeout(std::time::Duration::from_secs(5), receiver).await {
        Ok(Ok(Ok(()))) => HttpResponse::Ok().json(serde_json::json!({"executed":true})),
        Ok(Ok(Err(error))) => HttpResponse::BadRequest().body(error.to_string()),
        _ => HttpResponse::ServiceUnavailable()
            .body("Não foi possível confirmar a execução. Não repitas até verificar o resultado."),
    }
}

#[get("/api/websocket")]
async fn websocket(
    app_data: Data<RwLock<AppData>>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, msg_stream) = actix_ws::handle(&req, body)?;

    let data = app_data.read().await;
    let mut usb_tx = data.usb_tx.clone();
    let mut broadcast_rx = data.broadcast_tx.subscribe();
    let mut control_rx = data.settings_handle.subscribe_control_updates();

    // Spawn the handler (this is now where we do stuff)
    actix_web::rt::spawn(async move {
        let mut msg_stream = msg_stream.aggregate_continuations();

        let close_reason = loop {
            tokio::select! {
                Ok(()) = control_rx.changed() => {
                    // An empty JSON patch is compatible with existing clients;
                    // it tells the redesign to refresh its separate control endpoints.
                    let message = WsResponse(WebsocketResponse {
                        id: u64::MAX,
                        data: DaemonResponse::Patch(json_patch::Patch(vec![])),
                    });
                    if let Err(e) = send_response(message, &mut session).await {
                        break e;
                    }
                }
                Ok(patch) = broadcast_rx.recv() => {
                    let message = WsResponse(WebsocketResponse {
                        id: u64::MAX,
                        data: DaemonResponse::Patch(patch.data),
                    });
                    if let Err(e) = send_response(message, &mut session).await {
                        break e;
                    }
                }
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        AggregatedMessage::Ping(msg) => {
                            if let Err(e) = session.pong(&msg).await {
                                error!("Failed to send Pong: {}", e);
                                break Some(CloseReason {
                                    code: CloseCode::Error,
                                    description: Some(format!("Failed to Send Pong: {}", e)),
                                });
                            };
                        }
                        AggregatedMessage::Text(msg) => {
                            match serde_json::from_slice::<WebsocketRequest>(msg.as_ref()) {
                                Ok(request) => {
                                    let request_id = request.id;
                                    let result = handle_packet(request.data, &mut usb_tx).await;
                                    let response = match result {
                                        Ok(resp) => {
                                            match resp {
                                                DaemonResponse::Ok => {
                                                    WsResponse(WebsocketResponse {
                                                        id: request_id,
                                                        data: DaemonResponse::Ok,
                                                    })
                                                }
                                                DaemonResponse::Error(error) => {
                                                    WsResponse(WebsocketResponse {
                                                        id: request_id,
                                                        data: DaemonResponse::Error(error),
                                                    })
                                                }
                                                DaemonResponse::Status(status) => {
                                                    WsResponse(WebsocketResponse {
                                                        id: request_id,
                                                        data: DaemonResponse::Status(status),
                                                    })
                                                }
                                                DaemonResponse::MicLevel(level) => {
                                                    WsResponse(WebsocketResponse {
                                                        id: request_id,
                                                        data: DaemonResponse::MicLevel(level),
                                                    })
                                                }
                                                _ => {
                                                    // This should never fucking happen
                                                    break Some(CloseReason {
                                                        code: CloseCode::Abnormal,
                                                        description: Some("Unexpected Message Type".to_string()),
                                                    });
                                                }
                                            }
                                        },
                                        Err(error) => {
                                            WsResponse(WebsocketResponse {
                                                id: request_id,
                                                data: DaemonResponse::Error(error.to_string()),
                                            })
                                        }
                                    };
                                    if let Err(e) = send_response(response, &mut session).await {
                                        break e;
                                    }
                                }
                                Err(error) => {
                                    // Ok, we weren't able to deserialise the request into a proper object, we
                                    // now need to confirm whether it was at least valid JSON with a request id
                                    warn!("Error Deserialising Request to Object: {}", error);
                                    warn!("Original Request: {}", msg);

                                    debug!("Attempting Low Level request Id Extraction..");
                                    let request: serde_json::Result<Value> = serde_json::from_str(msg.as_ref());
                                    match request {
                                        Ok(value) => {
                                            if let Some(request_id) = value["id"].as_u64() {
                                                let response = WsResponse(WebsocketResponse {
                                                    id: request_id,
                                                    data: DaemonResponse::Error(error.to_string()),
                                                });
                                                if let Err(e) = send_response(response, &mut session).await {
                                                    break e;
                                                }
                                            } else {
                                                warn!("id missing, Cannot continue. Closing connection");
                                                let error = CloseReason {
                                                    code: CloseCode::Invalid,
                                                    description: Some(String::from(
                                                        "Missing or invalid Request ID",
                                                    )),
                                                };
                                                break Some(error);
                                            }
                                        }
                                        Err(error) => {
                                            warn!("JSON structure is invalid, closing connection.");
                                            let error = CloseReason {
                                                code: CloseCode::Invalid,
                                                description: Some(error.to_string()),
                                            };
                                            break Some(error);
                                        }
                                    }
                                }
                            }
                        }
                        AggregatedMessage::Binary(_) => {
                            error!("Received Binary Message, aborting!");
                            break Some(CloseReason {
                                code: CloseCode::Unsupported,
                                description: Some("Binary is not Supported".to_string()),
                            });
                        }
                        AggregatedMessage::Pong(_) => {}
                        AggregatedMessage::Close(reason) => {
                            break reason;
                        }
                    }
                }
                else => {
                    break None;
                }
            }
        };

        let _ = session.close(close_reason).await;
    });

    Ok(response)
}

/// Serialises and sends a WsResponse to a Session
async fn send_response(res: WsResponse, session: &mut Session) -> Result<(), Option<CloseReason>> {
    match serde_json::to_string(&res) {
        Ok(text) => {
            if let Err(e) = session.text(text).await {
                error!("Failed to send response: {}", e);
                return Err(Some(CloseReason {
                    code: CloseCode::Error,
                    description: Some(e.to_string()),
                }));
            }
        }
        Err(e) => {
            error!("Failed to serialize response: {}", e);
            return Err(Some(CloseReason {
                code: CloseCode::Error,
                description: Some(format!("Serialization Error: {}", e)),
            }));
        }
    }
    Ok(())
}

// So, fun note, according to the actix manual, web::Json uses serde_json to deserialise, good
// news everybody! So do we.. :)
#[post("/api/command")]
async fn execute_command(
    request: web::Json<DaemonRequest>,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    let mut data = app_data.write().await;

    // Errors propagate weirdly in the javascript world, so send all as OK, and handle there.
    match handle_packet(request.0, &mut data.usb_tx).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(error) => HttpResponse::Ok().json(DaemonResponse::Error(error.to_string())),
    }
}

#[get("/api/get-devices")]
async fn get_devices(app_data: Data<RwLock<AppData>>) -> HttpResponse {
    if let Ok(response) = get_status(app_data).await {
        return HttpResponse::Ok().json(&response);
    }
    HttpResponse::InternalServerError().finish()
}

#[get("/api/path")]
async fn get_path(app_data: Data<RwLock<AppData>>, req: HttpRequest) -> HttpResponse {
    let params = web::Query::<HashMap<String, String>>::from_query(req.query_string());
    if let Ok(params) = params {
        if let Some(path) = params.get("path") {
            if let Ok(status) = get_status(app_data).await {
                if let Ok(value) = serde_json::to_value(status) {
                    if let Ok(result) = value.query(path) {
                        return HttpResponse::Ok().json(result);
                    } else {
                        warn!("Invalid Path Provided..");
                    }
                } else {
                    warn!("Unable to Parse DaemonStatus..");
                }
            } else {
                warn!("Unable to Fetch Daemon Status..");
            }
        } else {
            warn!("Path Parameter Not Found..");
        }
    } else {
        warn!("Unable to Parse Parameters..");
    }

    HttpResponse::InternalServerError().finish()
}

#[get("/files/scribble/{serial}/{fader}.png")]
async fn get_scribble(
    path: web::Path<(String, FaderName)>,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    let serial = &path.0;
    let fader = path.1;

    let final_width = 128;
    let final_height = 64;

    // Now we need to grab the DaemonResponse to get the layout of the scribble
    let mut data = app_data.write().await;
    let request = DaemonRequest::GetStatus;

    // Pull out the scribble configuration from the Daemon Status
    let scribble = handle_packet(request, &mut data.usb_tx)
        .await
        .ok()
        .and_then(|response| match response {
            DaemonResponse::Status(status) => status
                .mixers
                .get(serial)
                .and_then(|mixer| mixer.fader_status[fader].scribble.clone()),
            _ => None,
        });

    if scribble.is_none() {
        return HttpResponse::NotFound().finish();
    }

    let state = &data.scribble_state[fader];
    if state.scribble_config == scribble && state.png_data.is_some() {
        // If we already have the PNG data, return it.
        if let Some(png_data) = &data.scribble_state[fader].png_data {
            debug!("Returning Cached Scribble Image for {} - {}", serial, fader);
            let mime_type = ContentType(IMAGE_PNG);
            let mut builder = HttpResponse::Ok();
            builder.insert_header(mime_type);
            return builder.body(png_data.clone());
        }
    } else if let Some(scribble) = scribble {
        debug!("Building Scribble Image for {} - {}", serial, fader);
        let scribble_path = data.file_paths.icons.clone();

        let mut icon_path = None;
        if let Some(file) = &scribble.file_name {
            icon_path = Some(scribble_path.join(file));
        }

        let png = get_scribble_png(
            icon_path,
            scribble.bottom_text.clone(),
            scribble.left_text.clone(),
            scribble.inverted,
            final_width,
            final_height,
        );

        if let Ok(png) = png {
            data.scribble_state[fader] = ScribbleState {
                scribble_config: Some(scribble.clone()),
                png_data: Some(png.clone()),
            };

            debug!("Creating Image {}x{}", final_width, final_height);

            let mime_type = ContentType(IMAGE_PNG);
            let mut builder = HttpResponse::Ok();
            builder.insert_header(mime_type);
            return builder.body(png);
        }
    }

    debug!("Unable to Build Image: {} - {}", serial, fader);
    HttpResponse::NotFound().finish()
}

#[get("/files/samples/{sample:.*}")]
async fn get_sample(sample: web::Path<String>, app_data: Data<RwLock<AppData>>) -> HttpResponse {
    // Get the Base Samples Path..
    let sample_path = {
        let data = app_data.read().await;
        data.file_paths.samples.clone()
    };

    let sample = sample.into_inner();
    let path = PathBuf::from(sample);

    if path.components().any(|part| part == Component::ParentDir) {
        // The path provided attempts to leave the samples dir, reject it.
        return HttpResponse::Forbidden().finish();
    }

    debug!("Attempting to Find {:?} in {:?}", path, sample_path);
    let file = find_file_in_path(sample_path, path);
    if let Some(path) = file {
        debug!("Found at {:?}", path);
        let mime_type = MimeGuess::from_path(path.clone()).first_or_octet_stream();
        let mut builder = HttpResponse::Ok();
        builder.insert_header(ContentType(mime_type));
        return builder.body(fs::read(path).unwrap());
    }

    HttpResponse::NotFound().finish()
}

#[post("/firmware-upload/{serial}")]
async fn upload_firmware(
    path: web::Path<String>,
    mut payload: Multipart,
    app_data: Data<RwLock<AppData>>,
) -> HttpResponse {
    let serial = path.into_inner();
    let file_path = env::temp_dir().join(format!("{serial}.bin"));

    let mut field = match payload.next().await {
        Some(Ok(field)) => field,
        Some(Err(e)) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error processing multipart: {e}"));
        }
        None => return HttpResponse::BadRequest().body("No file found in request"),
    };

    let mut file = match File::create(&file_path) {
        Ok(f) => f,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to create file: {e}"));
        }
    };

    while let Some(chunk) = field.next().await {
        match chunk {
            Ok(data) => {
                if let Err(e) = file.write_all(&data) {
                    return HttpResponse::InternalServerError()
                        .body(format!("Failed to write file: {e}"));
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Error reading file chunk: {e}"));
            }
        }
    }

    // When we get here, the file has been uploaded successfully...
    let data = app_data.read().await;
    let (tx, rx) = oneshot::channel();

    let _ = data
        .usb_tx
        .send(DeviceCommand::RunFirmwareUpdate(
            serial,
            Some(file_path),
            false,
            tx,
        ))
        .await;
    let result = rx.await;
    match result {
        Ok(_) => HttpResponse::Ok().body(serde_json::to_string(&DaemonResponse::Ok).unwrap()),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error Occurred: {e}")),
    }
}

async fn default(req: HttpRequest) -> HttpResponse {
    let path = if req.path() == "/" || req.path() == "" {
        "/redesign.html"
    } else {
        req.path()
    };
    let path_part = &path[1..path.len()];
    let file = WEB_CONTENT.get_file(path_part);
    if let Some(file) = file {
        let mime_type = MimeGuess::from_path(path).first_or_octet_stream();
        let mut builder = HttpResponse::Ok();
        builder.insert_header(ContentType(mime_type));
        builder.body(file.contents())
    } else {
        HttpResponse::NotFound().finish()
    }
}

async fn get_status(app_data: Data<RwLock<AppData>>) -> Result<DaemonStatus> {
    let mut data = app_data.write().await;
    let request = DaemonRequest::GetStatus;

    let result = handle_packet(request, &mut data.usb_tx).await?;
    match result {
        DaemonResponse::Status(status) => Ok(status),
        _ => Err(anyhow!("Unexpected Daemon Status Result: {:?}", result)),
    }
}

#[cfg(test)]
mod button_layer_request_tests {
    use super::local_settings_request;
    use actix_web::test::TestRequest;

    #[test]
    fn local_same_origin_is_allowed_but_remote_and_lookalike_hosts_are_rejected() {
        let make = |host: &str, origin: &str, peer: &str| {
            TestRequest::default()
                .insert_header(("Host", host))
                .insert_header(("Origin", origin))
                .peer_addr(peer.parse().unwrap())
                .to_http_request()
        };
        assert!(local_settings_request(&make(
            "localhost:14564",
            "http://localhost:14564",
            "127.0.0.1:40000"
        )));
        assert!(local_settings_request(&make(
            "127.0.0.1:14564",
            "http://127.0.0.1:14564",
            "127.0.0.1:40000"
        )));
        assert!(!local_settings_request(&make(
            "localhost:14564",
            "http://localhost.evil:14564",
            "127.0.0.1:40000"
        )));
        assert!(!local_settings_request(&make(
            "localhost.evil:14564",
            "http://localhost.evil:14564",
            "127.0.0.1:40000"
        )));
        assert!(!local_settings_request(&make(
            "localhost:14564",
            "http://localhost:14564",
            "192.168.1.2:40000"
        )));
    }
}

#[get("/api/app-update")]
async fn check_app_update(req: HttpRequest) -> HttpResponse {
    if !local_settings_request(&req) { return HttpResponse::Forbidden().finish(); }
    HttpResponse::Ok().json(crate::updates::check().await)
}
#[post("/api/app-update")]
async fn install_app_update(req: HttpRequest, body: web::Json<Value>) -> HttpResponse {
    if !local_settings_request(&req) { return HttpResponse::Forbidden().finish(); }
    let Some(version) = body.get("version").and_then(Value::as_str) else {
        return HttpResponse::BadRequest().json(serde_json::json!({"Error":"Missing version"}));
    };
    match crate::updates::install(version) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"started":true})),
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({"Error":error.to_string()}))
    }
}
