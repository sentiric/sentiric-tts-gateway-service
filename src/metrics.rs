use hyper::{service::{make_service_fn, service_fn}, Body, Request, Response, Server as HyperServer, StatusCode};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};
use crate::clients::coqui::CoquiClient;
use crate::clients::mms::MmsClient;

#[allow(dead_code)]
pub const GRPC_REQUESTS_TOTAL: &str = "sentiric_tts_gateway_grpc_requests_total";

async fn health_handler(coqui: Arc<CoquiClient>, mms: Arc<MmsClient>) -> Result<Response<Body>, Infallible> {
    let coqui_ready = coqui.is_ready();
    let mms_ready = mms.is_ready();

    if coqui_ready || mms_ready {
         let json = format!(r#"{{"status":"ok", "coqui":{}, "mms":{}}}"#, coqui_ready, mms_ready);
         Ok(Response::new(Body::from(json)))
    } else {
        let mut resp = Response::new(Body::from(r#"{"status":"error", "upstream":"all_disconnected"}"#));
        *resp.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
        Ok(resp)
    }
}

async fn route_handler(
    req: Request<Body>, 
    recorder_handle: PrometheusHandle,
    coqui: Arc<CoquiClient>,
    mms: Arc<MmsClient>
) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/metrics") => {
            let metrics = recorder_handle.render();
            Ok(Response::new(Body::from(metrics)))
        },
        (&hyper::Method::GET, "/health") | (&hyper::Method::GET, "/healthz") => {
            health_handler(coqui, mms).await
        },
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub fn start_metrics_server(addr: SocketAddr, coqui: CoquiClient, mms: MmsClient) {
    let recorder_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Prometheus recorder kurulumu başarısız oldu");

    let coqui = Arc::new(coqui);
    let mms = Arc::new(mms);

    tokio::spawn(async move {
        let make_svc = make_service_fn(move |_conn| {
            let recorder_handle = recorder_handle.clone();
            let coqui = coqui.clone();
            let mms = mms.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    route_handler(req, recorder_handle.clone(), coqui.clone(), mms.clone())
                }))
            }
        });

        let server = HyperServer::bind(&addr).serve(make_svc);
        
        info!(address = %addr, "Prometheus & Health sunucusu dinleniyor...");
        if let Err(e) = server.await {
            error!(error = %e, "Metrik sunucusu hatası.");
        }
    });
}