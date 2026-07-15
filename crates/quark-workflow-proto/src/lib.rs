//! Generated protobuf types and gRPC service stubs for the Temporal API.
//!
//! This crate is produced by `prost` + `tonic` codegen from the vendored
//! `.proto` files under `crates/api/protos/`. It provides:
//!
//! - **Message types** (request/response, enums, common types) for both the
//!   public SDK API (`temporal.api.*`) and the server-internal API
//!   (`temporal.server.api.*`).
//! - **gRPC client/server stubs** for the WorkflowService, HistoryService,
//!   MatchingService, AdminService, and OperatorService.
//!
//! # Module layout
//!
//! ```text
//! workflow_api::temporal::api::workflowservice::v1   // SDK-facing service
//! workflow_api::temporal::api::common::v1             // shared SDK types
//! workflow_api::temporal::api::enums::v1              // SDK enums
//! workflow_api::temporal::server::api::historyservice::v1  // history RPCs
//! workflow_api::temporal::server::api::matchingservice::v1 // matching RPCs
//! ```

// Allow all clippy lints on auto-generated prost/tonic code.
#![allow(clippy::all)]

// ---------------------------------------------------------------------------
// Re-export prost and tonic so downstream crates don't need direct deps.
// ---------------------------------------------------------------------------

pub use prost;
pub use prost_types;
pub use tonic;

// ---------------------------------------------------------------------------
// Generated SDK API types (public-facing Temporal API).
// ---------------------------------------------------------------------------

pub mod temporal {
    pub mod api {
        pub mod activity {
            pub mod v1 {
                tonic::include_proto!("temporal.api.activity.v1");
            }
        }
        pub mod batch {
            pub mod v1 {
                tonic::include_proto!("temporal.api.batch.v1");
            }
        }
        pub mod callback {
            pub mod v1 {
                tonic::include_proto!("temporal.api.callback.v1");
            }
        }
        pub mod command {
            pub mod v1 {
                tonic::include_proto!("temporal.api.command.v1");
            }
        }
        pub mod common {
            pub mod v1 {
                tonic::include_proto!("temporal.api.common.v1");
            }
        }
        pub mod compute {
            pub mod v1 {
                tonic::include_proto!("temporal.api.compute.v1");
            }
        }
        pub mod deployment {
            pub mod v1 {
                tonic::include_proto!("temporal.api.deployment.v1");
            }
        }
        pub mod enums {
            pub mod v1 {
                tonic::include_proto!("temporal.api.enums.v1");
            }
        }
        pub mod errordetails {
            pub mod v1 {
                tonic::include_proto!("temporal.api.errordetails.v1");
            }
        }
        pub mod export {
            pub mod v1 {
                tonic::include_proto!("temporal.api.export.v1");
            }
        }
        pub mod failure {
            pub mod v1 {
                tonic::include_proto!("temporal.api.failure.v1");
            }
        }
        pub mod filter {
            pub mod v1 {
                tonic::include_proto!("temporal.api.filter.v1");
            }
        }
        pub mod history {
            pub mod v1 {
                tonic::include_proto!("temporal.api.history.v1");
            }
        }
        pub mod namespace {
            pub mod v1 {
                tonic::include_proto!("temporal.api.namespace.v1");
            }
        }
        pub mod nexus {
            pub mod v1 {
                tonic::include_proto!("temporal.api.nexus.v1");
            }
        }
        pub mod nexusservices {
            pub mod workerservice {
                pub mod v1 {
                    tonic::include_proto!("temporal.api.nexusservices.workerservice.v1");
                }
            }
        }
        pub mod operatorservice {
            pub mod v1 {
                tonic::include_proto!("temporal.api.operatorservice.v1");
            }
        }
        pub mod protocol {
            pub mod v1 {
                tonic::include_proto!("temporal.api.protocol.v1");
            }
        }
        pub mod protometa {
            pub mod v1 {
                tonic::include_proto!("temporal.api.protometa.v1");
            }
        }
        pub mod query {
            pub mod v1 {
                tonic::include_proto!("temporal.api.query.v1");
            }
        }
        pub mod replication {
            pub mod v1 {
                tonic::include_proto!("temporal.api.replication.v1");
            }
        }
        pub mod rules {
            pub mod v1 {
                tonic::include_proto!("temporal.api.rules.v1");
            }
        }
        pub mod schedule {
            pub mod v1 {
                tonic::include_proto!("temporal.api.schedule.v1");
            }
        }
        pub mod sdk {
            pub mod v1 {
                tonic::include_proto!("temporal.api.sdk.v1");
            }
        }
        pub mod taskqueue {
            pub mod v1 {
                tonic::include_proto!("temporal.api.taskqueue.v1");
            }
        }
        pub mod update {
            pub mod v1 {
                tonic::include_proto!("temporal.api.update.v1");
            }
        }
        pub mod version {
            pub mod v1 {
                tonic::include_proto!("temporal.api.version.v1");
            }
        }
        pub mod worker {
            pub mod v1 {
                tonic::include_proto!("temporal.api.worker.v1");
            }
        }
        pub mod workflow {
            pub mod v1 {
                tonic::include_proto!("temporal.api.workflow.v1");
            }
        }
        pub mod workflowservice {
            pub mod v1 {
                tonic::include_proto!("temporal.api.workflowservice.v1");
            }
        }
    }

    // -----------------------------------------------------------------------
    // Generated server-internal API types.
    // -----------------------------------------------------------------------

    pub mod server {
        pub mod api {
            pub mod adminservice {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.adminservice.v1");
                }
            }
            pub mod archiver {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.archiver.v1");
                }
            }
            pub mod batch {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.batch.v1");
                }
            }
            pub mod chasm {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.chasm.v1");
                }
            }
            pub mod checksum {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.checksum.v1");
                }
            }
            pub mod cli {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.cli.v1");
                }
            }
            pub mod clock {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.clock.v1");
                }
            }
            pub mod cluster {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.cluster.v1");
                }
            }
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.common.v1");
                }
            }
            pub mod contextpropagation {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.contextpropagation.v1");
                }
            }
            pub mod deployment {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.deployment.v1");
                }
            }
            pub mod enums {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.enums.v1");
                }
            }
            pub mod errordetails {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.errordetails.v1");
                }
            }
            pub mod health {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.health.v1");
                }
            }
            pub mod history {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.history.v1");
                }
            }
            pub mod historyservice {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.historyservice.v1");
                }
            }
            pub mod matchingservice {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.matchingservice.v1");
                }
            }
            pub mod metrics {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.metrics.v1");
                }
            }
            pub mod namespace {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.namespace.v1");
                }
            }
            pub mod persistence {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.persistence.v1");
                }
            }
            pub mod replication {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.replication.v1");
                }
            }
            pub mod routing {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.routing.v1");
                }
            }
            pub mod schedule {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.schedule.v1");
                }
            }
            pub mod taskqueue {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.taskqueue.v1");
                }
            }
            pub mod testservice {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.testservice.v1");
                }
            }
            pub mod token {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.token.v1");
                }
            }
            pub mod visibilityservice {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.visibilityservice.v1");
                }
            }
            pub mod workflow {
                pub mod v1 {
                    tonic::include_proto!("temporal.server.api.workflow.v1");
                }
            }
        }
    }
}
