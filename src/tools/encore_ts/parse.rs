// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Bartek Kus
// Feature: ENCORE_TS_INTEGRATION
// Spec: spec/core/encore_ts.md

use crate::tools::encore_ts::schemas::{ApiInfo, MetaSnapshotV1, ServiceInfo};
use anyhow::{Context, Result};
use encore_tsparser::parser::parser::{ParseContext, Parser};
use encore_tsparser::parser::resourceparser::PassOneParser;
use encore_tsparser::parser::resources::Resource;
use std::path::Path;
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
        eprintln!("Encore Parser Error: {}", db.message());
        log::warn!("Encore Parser: {}", db.message());
    }
}

pub fn parse(root: &Path) -> Result<MetaSnapshotV1> {
    let globals = swc_common::Globals::new();
    swc_common::GLOBALS.set(&globals, || {
        let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        let handler = Lrc::new(Handler::with_emitter(
            true,
            false,
            Box::new(CapturingEmitter),
        ));

        // Set the global handler for the duration of the parsing
        swc_common::errors::HANDLER.set(&handler, || {
            // ParseContext expects app_root.
            let ctx = ParseContext::new(
                root.to_path_buf(),
                None, // js_runtime_path
                cm.clone(),
                handler.clone(),
            )
            .context("Failed to create ParseContext")?;

            let pass1 = PassOneParser::new(
                ctx.file_set.clone(),
                ctx.type_checker.clone(),
                Default::default(),
            );
            let parser = Parser::new(&ctx, pass1);
            let result = parser.parse();

            if handler.has_errors() {
                anyhow::bail!("Encore TS parsing failed with errors");
            }

            // Map ParseResult to MetaSnapshotV1
            let mut service_infos = Vec::new();

            // The result.services contains Service structs which have the binds.
            // But result.resources contains all resources flattened.
            // Iterating services seems cleaner to group APIs.

            for service in result.services {
                let mut apis = Vec::new();

                for bind in service.binds {
                    if let Resource::APIEndpoint(endpoint_rc) = &bind.resource {
                        let endpoint = &**endpoint_rc;
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
        })
    })
}
