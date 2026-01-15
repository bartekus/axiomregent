use crate::tools::encore_ts::schemas::{ApiInfo, MetaSnapshotV1, ServiceInfo};
use anyhow::{Context, Result};
use encore_tsparser::parser::resources::Resource;
use encore_tsparser::parser::{ParseContext, Parser, resourceparser::PassOneParser};
use std::path::{Path, PathBuf};
use swc_common::{
    FilePathMapping, SourceMap,
    errors::{DiagnosticBuilder, Emitter, Handler},
    sync::Lrc,
};

// A silent emitter to capture errors if needed, or we can just log them.
// For now, we use a simple emitter that logs to stderr if we want, or just ignore for the tool output?
// We probably want to fail if there are errors.

struct CapturingEmitter;

impl Emitter for CapturingEmitter {
    fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
        // In a real implementation we might want to collect these.
        // For now, let's just log them as warnings.
        log::warn!("Encore Parser: {}", db.message());
    }
}

pub fn parse(root: &Path) -> Result<MetaSnapshotV1> {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let handler = Lrc::new(Handler::with_emitter(
        true,
        false,
        Box::new(CapturingEmitter),
    ));

    // ParseContext expects app_root.
    let ctx = ParseContext::new(
        root.to_path_buf(),
        None, // js_runtime_path
        cm.clone(),
        handler.clone(),
    )
    .context("Failed to create ParseContext")?;

    let pass1 = PassOneParser::new(&ctx);
    let parser = Parser::new(&ctx, pass1);
    let result = parser.parse();

    // Map ParseResult to MetaSnapshotV1
    let mut service_infos = Vec::new();

    // The result.services contains Service structs which have the binds.
    // But result.resources contains all resources flattened.
    // Iterating services seems cleaner to group APIs.

    for service in result.services {
        let mut apis = Vec::new();

        for bind in service.binds {
            if let Resource::APIEndpoint(endpoint_rc) = &bind.resource {
                let endpoint = &*endpoint_rc;
                // Extract method
                let method = endpoint.encoding.default_method.as_str().to_string();

                // Extract path
                // Path implements Display
                let path_str = endpoint.encoding.path.to_string();

                let access = if endpoint.require_auth {
                    "auth".to_string()
                } else if endpoint.expose {
                    "public".to_string()
                } else {
                    "private".to_string()
                };

                apis.push(ApiInfo {
                    name: endpoint.name.clone(),
                    path: path_str,
                    method,
                    access,
                });
            }
        }

        service_infos.push(ServiceInfo {
            name: service.name.clone(),
            description: service.doc.clone(),
            apis,
        });
    }

    Ok(MetaSnapshotV1 {
        services: service_infos,
    })
}
