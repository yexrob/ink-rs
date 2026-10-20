// Ink examples/incremental-rendering.

use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::rc::Rc;
use std::time::Duration;

use ink_rs::{
    AnimationOptions, BorderStyle, Color, Element, FlexDirection, Scope, Text, TextWrap, View,
    component, rsx,
};

const SERVICES: [&str; 30] = [
    "Server Authentication Module - Handles JWT token validation, OAuth2 flows, and session management across distributed systems",
    "Database Connection Pool - Maintains persistent connections to PostgreSQL cluster with automatic failover and load balancing",
    "API Gateway Service - Routes incoming HTTP requests to microservices with rate limiting and request transformation",
    "User Profile Manager - Caches user data in Redis with write-through policy and invalidation strategies",
    "Payment Processing Engine - Integrates with Stripe, PayPal, and Square APIs for transaction processing",
    "Email Notification Queue - Processes outbound emails through SendGrid with retry logic and delivery tracking",
    "File Storage Handler - Manages S3 bucket operations with multipart uploads and CDN integration",
    "Search Indexer Service - Maintains Elasticsearch indices with real-time document updates and reindexing",
    "Metrics Aggregation Pipeline - Collects and processes telemetry data for Prometheus and Grafana dashboards",
    "WebSocket Connection Manager - Handles real-time bidirectional communication for chat and notifications",
    "Cache Invalidation Service - Coordinates distributed cache updates across Redis cluster nodes",
    "Background Job Processor - Executes async tasks via RabbitMQ with dead letter queue handling",
    "Session Store Manager - Persists user sessions in DynamoDB with TTL and cross-region replication",
    "Rate Limiter Module - Enforces API quotas using token bucket algorithm with Redis backend",
    "Content Delivery Network - Serves static assets through Cloudflare with edge caching and GZIP compression",
    "Logging Aggregator - Streams application logs to ELK stack with structured JSON formatting",
    "Health Check Monitor - Performs periodic service health checks with circuit breaker pattern implementation",
    "Configuration Manager - Loads environment-specific settings from Consul with hot reload capability",
    "Security Scanner Service - Runs automated vulnerability scans and dependency checks on deployed applications",
    "Backup Orchestrator - Schedules and executes automated database backups with encryption and versioning",
    "Load Balancer Controller - Manages NGINX upstream servers with health-based traffic distribution",
    "Container Orchestration - Coordinates Docker container lifecycle via Kubernetes with auto-scaling policies",
    "Message Bus Coordinator - Routes events through Apache Kafka topics with guaranteed delivery semantics",
    "Analytics Data Warehouse - Aggregates business metrics in Snowflake with incremental ETL processes",
    "API Documentation Service - Generates and serves OpenAPI specs with interactive Swagger UI",
    "Feature Flag Manager - Controls feature rollouts using LaunchDarkly with user targeting and percentage rollouts",
    "Audit Trail Logger - Records all user actions and system events for compliance and security analysis",
    "Image Processing Pipeline - Resizes and optimizes uploaded images using Sharp with multiple format outputs",
    "Geolocation Service - Resolves IP addresses to geographic coordinates using MaxMind GeoIP2 database",
    "Recommendation Engine - Generates personalized content suggestions using collaborative filtering algorithms",
];

fn random(seed: &Cell<u64>) -> u64 {
    let mut x = seed.get();
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    seed.set(x);
    x
}

fn log_line(index: usize, batch: u64, seconds: u64, seed: &Cell<u64>) -> String {
    let actions = [
        "PROCESSING",
        "COMPLETED",
        "UPDATING",
        "SYNCING",
        "VALIDATING",
        "EXECUTING",
    ];
    format!(
        "[{seconds:06}] Worker-{index} {}: Batch={batch} Throughput={}req/s Memory={}.{}MB CPU={}.{}%",
        actions[random(seed) as usize % actions.len()],
        random(seed) % 1000,
        random(seed) % 512,
        random(seed) % 10,
        random(seed) % 100,
        random(seed) % 10
    )
}

pub fn app() -> Element {
    component(monitor, ())
}

fn monitor(cx: &mut Scope, _: &()) -> Element {
    let height = cx.use_window_size().rows;
    let available = usize::from(height.saturating_sub(15).max(10));
    let log_count = (available * 3 / 10).max(3);
    let service_count = (available * 7 / 10).clamp(5, SERVICES.len());
    let selected = cx.use_state(|| 0_usize);
    let seed = cx.use_memo((), || {
        Rc::new(Cell::new(RandomState::new().build_hasher().finish().max(1)))
    });
    let logs = cx.use_state(|| {
        (0..log_count)
            .map(|i| log_line(i, 0, 0, &seed))
            .collect::<Vec<_>>()
    });
    let fast = cx.use_animation(AnimationOptions {
        interval: Duration::from_millis(16),
        ..AnimationOptions::default()
    });
    let seconds = cx.use_animation(AnimationOptions {
        interval: Duration::from_secs(1),
        ..AnimationOptions::default()
    });
    // Counters/progress derive from frame: late wakes skip to the elapsed count.
    // Logs take one mutation per delivered fast frame, without replaying gaps.
    let noise = cx.use_state(|| 0_u64);
    let effect_seed = seed.clone();
    cx.use_effect((fast.frame, log_count), move || {
        logs.update(|logs| {
            logs.truncate(log_count);
            while logs.len() < log_count {
                logs.push(log_line(logs.len(), 0, seconds.frame, &effect_seed));
            }
            if fast.frame > 0 {
                let index = random(&effect_seed) as usize % logs.len();
                logs[index] = log_line(
                    index,
                    random(&effect_seed) % 1000,
                    seconds.frame,
                    &effect_seed,
                );
            }
        });
        if fast.frame > 0 {
            noise.set(random(&effect_seed) % 1000);
        }
    });
    let app = cx.use_app();
    cx.use_input(true, move |input, key| {
        if input == "q" {
            app.exit();
        }
        selected.update(|index| {
            *index = (*index).min(service_count - 1);
            if key.up_arrow {
                *index = (*index + service_count - 1) % service_count;
            }
            if key.down_arrow {
                *index = (*index + 1) % service_count;
            }
        });
    });
    // Derive the resize clamp; writing State in render is not allowed.
    let selected = selected.get().min(service_count - 1);
    // ink-rs diffs cells in every mode, so there is no incrementalRendering flag.
    let fps = if fast.delta.is_zero() {
        0.0
    } else {
        1.0 / fast.delta.as_secs_f64()
    };
    let progress_bars = [Color::Green, Color::Yellow, Color::Red]
        .into_iter()
        .enumerate()
        .map(|(index, color)| {
            let progress = (fast.frame * (index as u64 + 1) % 101) as usize;
            (index, color, progress)
        });
    let header = rsx! {
        View(flex_direction: FlexDirection::Column) {
            Text(bold: true, color: Color::Cyan) {
                "Incremental Rendering Demo - cell diff"
            }
            Text(dim_color: true) {
                "Use up/down arrows to navigate | Press q to quit | FPS: {fps:.0}"
            }
            Text {
                {format!(
                    "Time: {}s | Updates: {} | Random: {}",
                    seconds.frame,
                    seconds.frame,
                    noise.get()
                )}
            }
            for (index, color, progress) in progress_bars {
                Text {
                    {format!("Progress {}: ", index + 1)}
                    Text(color: color) {
                        {format!(
                            "{}{}",
                            "\u{2588}".repeat(progress / 5),
                            "\u{2591}".repeat(20 - progress / 5)
                        )}
                    }
                    " {progress}%"
                }
            }
        }
    };
    // Truncate the long service/log rows to keep both panels and selection visible.
    rsx! {
        View(flex_direction: FlexDirection::Column) {
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Cyan,
                padding_x: 2,
                padding_y: 1,
            ) {
                {header}
            }
            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Single,
                border_color: Color::Yellow,
                padding_x: 2,
                padding_y: 1,
                margin_top: 1,
            ) {
                Text(bold: true, color: Color::Yellow) {
                    "Live Logs (one line updates per frame):"
                }
                for (i, line) in logs.get().into_iter().enumerate() {
                    Text(wrap: TextWrap::TruncateEnd, color: Color::Green, key: i) { {line} }
                }
            }
            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Single,
                border_color: Color::Gray,
                padding_x: 2,
                padding_y: 1,
                margin_top: 1,
            ) {
                Text(bold: true, color: Color::Magenta) {
                    "System Services Monitor ({service_count} of 30 services):"
                }
                for (i, row) in SERVICES[..service_count].iter().enumerate() {
                    Text(
                        wrap: TextWrap::TruncateEnd,
                        color: if i == selected { Color::Blue } else { Color::White },
                        key: i,
                    ) {
                        {format!("{} {row}", if i == selected { ">" } else { " " })}
                    }
                }
            }
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Magenta,
                padding_x: 2,
                margin_top: 1,
            ) {
                Text(wrap: TextWrap::TruncateEnd) {
                    "Selected: "
                    Text(bold: true, color: Color::Magenta) { {SERVICES[selected]} }
                }
            }
        }
    }
    .into()
}
