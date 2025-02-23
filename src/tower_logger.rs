use std::net::IpAddr;

use hyper::{
    body::{Body, Bytes},
    Request,
};
use tower_http::trace::{MakeSpan, OnBodyChunk, OnRequest, OnResponse};
use tracing::{span, Span};

use crate::log_utils::{
    log_body_frame, log_headers, log_latency, log_request_uri, log_response_uri, Direction,
    HttpLogLevel,
};

#[derive(Debug, Clone)]
pub struct OnRequestLogger {
    log_level: HttpLogLevel,
}

impl OnRequestLogger {
    pub fn new(log_level: HttpLogLevel) -> Self {
        Self { log_level }
    }
}

impl<B> OnRequest<B> for OnRequestLogger
where
    B: Body,
{
    fn on_request(&mut self, request: &Request<B>, _span: &Span) {
        match &self.log_level {
            HttpLogLevel::None => {}
            HttpLogLevel::Uri => {
                log_request_uri(request);
            }
            HttpLogLevel::UriHeaders | HttpLogLevel::UriHeadersBody => {
                log_request_uri(request);
                log_headers(request.headers(), Direction::Incoming);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OnResponseLogger {
    log_level: HttpLogLevel,
}

impl OnResponseLogger {
    pub fn new(log_level: HttpLogLevel) -> Self {
        Self { log_level }
    }
}

impl<B> OnResponse<B> for OnResponseLogger
where
    B: Body,
{
    fn on_response(
        self,
        response: &hyper::Response<B>,
        latency: std::time::Duration,
        _span: &Span,
    ) {
        match self.log_level {
            HttpLogLevel::None => {
                return;
            }
            HttpLogLevel::Uri => {
                log_response_uri(response);
            }
            HttpLogLevel::UriHeaders | HttpLogLevel::UriHeadersBody => {
                log_response_uri(response);
                log_headers(response.headers(), Direction::Outgoing);
            }
        }
        log_latency(latency);
    }
}

#[derive(Debug, Clone)]
pub struct BodyLogger {
    log_level: HttpLogLevel,
}

impl BodyLogger {
    pub fn new(log_level: HttpLogLevel) -> Self {
        Self { log_level }
    }
}

impl OnBodyChunk<Bytes> for BodyLogger {
    fn on_body_chunk(&mut self, chunk: &Bytes, _latency: std::time::Duration, span: &Span) {
        if self.log_level == HttpLogLevel::UriHeadersBody {
            log_body_frame(chunk, span);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpanMaker {
    client_ip: IpAddr,
    id: u64,
}

impl SpanMaker {
    pub fn new(client_ip: IpAddr, id: u64) -> Self {
        Self { client_ip, id }
    }
}

impl<B> MakeSpan<B> for SpanMaker {
    fn make_span(&mut self, _: &Request<B>) -> Span {
        span!(
            tracing::Level::INFO,
            "client",
            ip = ?self.client_ip,
            id = self.id
        )
    }
}
