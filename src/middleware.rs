use std::sync::Arc;

use my_app_insights::AppInsightsTelemetry;
use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware, HttpServerRequestFlow,
};
use rust_extensions::StopWatch;

pub struct AppInsightsMiddleware {
    telemetry: Arc<AppInsightsTelemetry>,
}

impl AppInsightsMiddleware {
    pub fn new(telemetry: Arc<AppInsightsTelemetry>) -> Self {
        Self { telemetry }
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for AppInsightsMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let mut sw = StopWatch::new();
        sw.start();

        let result = get_next.next(ctx).await;

        sw.pause();

        match &result {
            Ok(ok_result) => {
                if ok_result.write_telemetry {
                    self.telemetry.write_http_request_duration(
                        ctx.request.uri.clone(),
                        ctx.request.method.clone(),
                        ok_result.get_status_code(),
                        sw.duration(),
                    );
                }
            }
            Err(fail_result) => {
                if fail_result.write_telemetry {
                    self.telemetry.write_http_request_duration(
                        ctx.request.uri.clone(),
                        ctx.request.method.clone(),
                        fail_result.status_code,
                        sw.duration(),
                    );
                }
            }
        }

        result
    }
}
