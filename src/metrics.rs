use opentelemetry::metrics::{Counter, Meter};

#[derive(Clone, Debug)]
pub struct Metrics {
    pub rpc_call_counter: Counter<u64>,
    pub http_call_counter: Counter<u64>,
    pub http_latency_tracker: Counter<f64>,
    pub rejected_project_counter: Counter<u64>,
}

impl Metrics {
    pub fn new(meter: &Meter) -> Self {
        let rpc_call_counter = meter
            .u64_counter("rpc_call_counter")
            .with_description("The number of rpc calls served")
            .init();

        let http_call_counter = meter
            .u64_counter("http_call_counter")
            .with_description("The number of http calls served")
            .init();

        let http_latency_tracker = meter
            .f64_counter("http_latency_tracker")
            .with_description("The http call latency")
            .init();

        let rejected_project_counter = meter
            .u64_counter("rejected_project_counter")
            .with_description("The number of calls for invalid project ids")
            .init();

        Metrics {
            rpc_call_counter,
            http_call_counter,
            http_latency_tracker,
            rejected_project_counter,
        }
    }
}

impl Metrics {
    pub fn add_rpc_call(&self, chain_id: &str) {
        self.rpc_call_counter.add(
            1,
            &[opentelemetry::KeyValue::new(
                "chain.id",
                chain_id.to_owned(),
            )],
        );
    }

    pub fn add_http_call(&self, code: u16, route: &str) {
        self.http_call_counter.add(
            1,
            &[
                opentelemetry::KeyValue::new("code", i64::from(code)),
                opentelemetry::KeyValue::new("route", route.to_owned()),
            ],
        );
    }

    pub fn add_http_latency(&self, code: u16, route: &str, latency: f64) {
        self.http_latency_tracker.add(
            latency,
            &[
                opentelemetry::KeyValue::new("code", i64::from(code)),
                opentelemetry::KeyValue::new("route", route.to_owned()),
            ],
        )
    }

    pub fn add_rejected_project(&self, project_id: &str) {
        self.rejected_project_counter.add(
            1,
            &[opentelemetry::KeyValue::new(
                "project_id",
                project_id.to_owned(),
            )],
        )
    }
}
