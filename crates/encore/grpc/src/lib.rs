mod client;
mod server;

pub mod encore {
    pub mod daemon {
        tonic::include_proto!("encore.daemon");
    }
    pub mod engine {
        pub mod trace {
            tonic::include_proto!("encore.engine.trace");
        }
        pub mod trace2 {
            tonic::include_proto!("encore.engine.trace2");
        }
    }
    pub mod parser {
        pub mod schema {
            pub mod v1 {
                tonic::include_proto!("encore.parser.schema.v1");
            }
        }
        pub mod meta {
            pub mod v1 {
                tonic::include_proto!("encore.parser.meta.v1");
            }
        }
    }
    pub mod runtime {
        pub mod v1 {
            tonic::include_proto!("encore.runtime.v1");
        }
    }
}

pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    // does nothing

    Ok(())
}
