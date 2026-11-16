//! Local, synchronous file authoring over MCP stdio; no live desktop session.

use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned, de::IgnoredAny};
use serde_json::{Map, Value, json};
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use weftforge_core::{Command, Project};

const MODERN_VERSION: &str = "2026-07-28";
const LEGACY_VERSION: &str = "2025-11-25";
const MAX_LINE_BYTES: usize = 1024 * 1024;
use super::MAX_COMMANDS;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum RequestId {
    Text(String),
    Signed(i64),
    Unsigned(u64),
}

// Missing is optional; explicit null must still fail typed validation.
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "T: Deserialize<'de>"))]
struct Request<T> {
    jsonrpc: String,
    #[serde(default, deserialize_with = "present")]
    id: Option<RequestId>,
    method: String,
    #[serde(default, deserialize_with = "present")]
    params: Option<T>,
}

#[derive(Default, Deserialize)]
struct Metadata {
    #[serde(
        rename = "io.modelcontextprotocol/protocolVersion",
        default,
        deserialize_with = "present"
    )]
    version: Option<String>,
    #[serde(
        rename = "io.modelcontextprotocol/clientCapabilities",
        default,
        deserialize_with = "present"
    )]
    capabilities: Option<Map<String, Value>>,
    #[serde(
        rename = "io.modelcontextprotocol/clientInfo",
        default,
        deserialize_with = "present"
    )]
    _client_info: Option<Implementation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Implementation {
    #[serde(rename = "name")]
    _name: String,
    #[serde(rename = "version")]
    _version: String,
    #[serde(rename = "title")]
    _title: Option<String>,
    #[serde(rename = "description")]
    _description: Option<String>,
    #[serde(rename = "websiteUrl")]
    _website_url: Option<String>,
    #[serde(rename = "icons")]
    _icons: Option<Vec<Icon>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Icon {
    #[serde(rename = "src")]
    _src: String,
    #[serde(rename = "mimeType")]
    _mime_type: Option<String>,
    #[serde(rename = "sizes")]
    _sizes: Option<Vec<String>>,
    #[serde(rename = "theme")]
    _theme: Option<Theme>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Theme {
    Light,
    Dark,
}

#[derive(Default, Deserialize)]
struct MetadataProbe {
    #[serde(rename = "_meta", default, deserialize_with = "present")]
    metadata: Option<Metadata>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyParams {
    #[serde(rename = "_meta")]
    _metadata: Option<Metadata>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InitializeParams {
    protocol_version: String,
    #[serde(rename = "capabilities")]
    _capabilities: Map<String, Value>,
    #[serde(rename = "clientInfo")]
    _client_info: Implementation,
    #[serde(rename = "_meta")]
    _metadata: Option<Metadata>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListParams {
    cursor: Option<String>,
    #[serde(rename = "_meta")]
    _metadata: Option<Metadata>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "T: Deserialize<'de>"))]
struct CallParams<T> {
    name: String,
    #[serde(default, deserialize_with = "present")]
    arguments: Option<T>,
    #[serde(rename = "_meta")]
    _metadata: Option<Metadata>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InspectArgs {
    path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LengthArgs {
    path: String,
    query: super::LengthQuery,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateArgs {
    output: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileArgs {
    input: String,
    output: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecuteArgs {
    input: String,
    output: String,
    commands: Vec<Command>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportSvgArgs {
    input: String,
    output: String,
    svg: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportDxfArgs {
    input: String,
    output: String,
    dxf: String,
}

#[derive(Default)]
enum LegacyState {
    #[default]
    New,
    Initializing,
    Ready,
}

pub(super) fn serve() -> super::Result<()> {
    serve_stream(&mut io::stdin().lock(), &mut io::stdout().lock())?;
    Ok(())
}

fn serve_stream(input: &mut impl BufRead, output: &mut impl Write) -> io::Result<()> {
    let mut legacy = LegacyState::New;
    loop {
        let mut line = Vec::new();
        let read = input
            .by_ref()
            .take((MAX_LINE_BYTES + 1) as u64)
            .read_until(b'\n', &mut line)?;
        if read == 0 {
            return Ok(());
        }
        let response = if line.len() > MAX_LINE_BYTES {
            if line.last() != Some(&b'\n') {
                loop {
                    let bytes = input.fill_buf()?;
                    if bytes.is_empty() {
                        break;
                    }
                    let end = bytes.iter().position(|byte| *byte == b'\n');
                    let consumed = end.map_or(bytes.len(), |index| index + 1);
                    input.consume(consumed);
                    if end.is_some() {
                        break;
                    }
                }
            }
            Some(error(None, -32600, "MCP line exceeds the 1 MiB limit"))
        } else {
            handle(&line, &mut legacy)
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut *output, &response)?;
            output.write_all(b"\n")?;
            output.flush()?;
        }
    }
}

fn error(id: Option<&RequestId>, code: i32, message: impl AsRef<str>) -> Value {
    let mut response = json!({"jsonrpc":"2.0","error":{"code":code,"message":message.as_ref()}});
    if let Some(id) = id {
        response["id"] = json!(id);
    }
    response
}

fn parameters<T: DeserializeOwned>(line: &[u8]) -> Result<Option<T>, String> {
    // Parse original bytes again: a Value round-trip would erase duplicate command fields.
    serde_json::from_slice::<Request<T>>(line)
        .map(|request| request.params)
        .map_err(|error| error.to_string())
}

fn required_parameters<T: DeserializeOwned>(line: &[u8]) -> Result<T, String> {
    parameters(line)?.ok_or_else(|| "params object is required".into())
}

fn handle(line: &[u8], legacy: &mut LegacyState) -> Option<Value> {
    let request = match serde_json::from_slice::<Request<IgnoredAny>>(line) {
        Ok(request) => request,
        Err(cause) => {
            let code = match cause.classify() {
                serde_json::error::Category::Syntax | serde_json::error::Category::Eof => -32700,
                _ => -32600,
            };
            return Some(error(None, code, cause.to_string()));
        }
    };
    if request.jsonrpc != "2.0" {
        return Some(error(request.id.as_ref(), -32600, "jsonrpc must be 2.0"));
    }
    let Some(id) = request.id.as_ref() else {
        if request.method == "notifications/initialized"
            && matches!(legacy, LegacyState::Initializing)
            && parameters::<EmptyParams>(line).is_ok()
        {
            *legacy = LegacyState::Ready;
        }
        // Unknown notifications, including cancellations of already finished calls, have no response.
        return None;
    };
    let params = match parameters::<MetadataProbe>(line) {
        Ok(params) => params,
        Err(cause) => return Some(error(Some(id), -32602, cause)),
    };
    let metadata = params.as_ref().and_then(|params| params.metadata.as_ref());
    let modern = metadata.is_some_and(|metadata| {
        metadata.version.is_some()
            || metadata.capabilities.is_some()
            || metadata._client_info.is_some()
    });
    if modern {
        let metadata = metadata.expect("checked above");
        if metadata.version.is_none() {
            return Some(error(
                Some(id),
                -32602,
                "Modern requests require _meta protocolVersion",
            ));
        }
        if metadata.version.as_deref() != Some(MODERN_VERSION) {
            let mut response = error(Some(id), -32022, "Unsupported protocol version");
            response["error"]["data"] = json!({
                "supported":[MODERN_VERSION],"requested":metadata.version,
            });
            return Some(response);
        }
        if metadata.capabilities.is_none() {
            return Some(error(
                Some(id),
                -32602,
                "Modern requests require _meta clientCapabilities",
            ));
        }
    }
    let result = match request.method.as_str() {
        "initialize" if !modern => {
            if !matches!(legacy, LegacyState::New) {
                return Some(error(Some(id), -32600, "Legacy session is already initialized"));
            }
            match required_parameters::<InitializeParams>(line) {
                Ok(params) => {
                    // Legacy negotiation returns our supported revision when the requested one differs.
                    let _requested_version = params.protocol_version;
                    *legacy = LegacyState::Initializing;
                    Ok(json!({"protocolVersion":LEGACY_VERSION,"capabilities":{"tools":{}},
                        "serverInfo":server_info(),"instructions":instructions()}))
                }
                Err(cause) => Err(cause),
            }
        }
        "ping" if !modern => parameters::<EmptyParams>(line).map(|_| json!({})),
        "server/discover" if modern => parameters::<EmptyParams>(line).map(|_| json!({
            "supportedVersions":[MODERN_VERSION],"capabilities":{"tools":{}},
            "_meta":{"io.modelcontextprotocol/serverInfo":server_info()},"instructions":instructions(),
            "ttlMs":0,"cacheScope":"public",
        })),
        "server/discover" => Err("server/discover requires modern per-request _meta".into()),
        "tools/list" | "tools/call" => {
            if !modern && !matches!(legacy, LegacyState::Ready) {
                Err("Send initialize then notifications/initialized, or supply modern per-request _meta".into())
            } else if request.method == "tools/list" {
                match parameters::<ListParams>(line) {
                    Ok(params) if params.as_ref().is_none_or(|params| params.cursor.is_none()) => {
                        let mut result = json!({"tools":tools()});
                        if modern {
                            result["ttlMs"] = json!(0);
                            result["cacheScope"] = json!("public");
                        }
                        Ok(result)
                    }
                    Ok(_) => Err("Invalid cursor; the complete tool list is returned in one page".into()),
                    Err(cause) => Err(cause),
                }
            } else {
                call(line)
            }
        }
        _ => return Some(error(Some(id), -32601, format!("Unknown method: {}", request.method))),
    };
    Some(match result {
        Ok(mut result) => {
            if modern {
                result["resultType"] = json!("complete");
            }
            json!({"jsonrpc":"2.0","id":id,"result":result})
        }
        Err(cause) => error(Some(id), -32602, cause),
    })
}

fn server_info() -> Value {
    json!({"name":"weftforge-cli","version":env!("CARGO_PKG_VERSION")})
}

fn instructions() -> &'static str {
    "Local stateless file authoring in SI units. Inspect stable IDs before edits. Each command array commits as one transaction. No active GUI control, simulation jobs, or persistent undo."
}

fn arguments<T: DeserializeOwned>(line: &[u8]) -> super::Result<T> {
    required_parameters::<CallParams<T>>(line)?
        .arguments
        .ok_or_else(|| "arguments object is required".into())
}

fn call(line: &[u8]) -> Result<Value, String> {
    let header = required_parameters::<CallParams<IgnoredAny>>(line)?;
    if !matches!(
        header.name.as_str(),
        "project_inspect"
            | "project_lengths"
            | "project_create"
            | "project_execute"
            | "project_import_svg"
            | "project_import_dxf"
            | "project_export_svg"
            | "project_export_obj"
            | "project_export_dxf"
    ) {
        return Err(format!("Unknown tool: {}", header.name));
    }
    let operation = || -> super::Result<Value> {
        match header.name.as_str() {
            "project_lengths" => {
                let args: LengthArgs = arguments(line)?;
                let project = Project::load(Path::new(&args.path))?;
                super::length_report(&project, args.query)
            }
            "project_inspect" => {
                let args: InspectArgs = arguments(line)?;
                let project = Project::load(Path::new(&args.path))?;
                // Preserve native f32 JSON formatting instead of promoting colors to f64.
                let project: Value = serde_json::from_slice(&serde_json::to_vec(&project)?)?;
                Ok(
                    json!({"project":project,"units":{"length":"m","mass":"kg","time":"s"},"up":"Y"}),
                )
            }
            "project_create" => {
                let args: CreateArgs = arguments(line)?;
                match std::fs::symlink_metadata(&args.output) {
                    Ok(_) => {
                        return Err("New project output already exists; choose a new path".into());
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                let project = Project::default();
                project.save(Path::new(&args.output))?;
                Ok(json!({"output":args.output,"revision":project.revision}))
            }
            "project_execute" => {
                let args: ExecuteArgs = arguments(line)?;
                let (revision, count) = super::execute_commands(
                    Path::new(&args.input),
                    Path::new(&args.output),
                    args.commands,
                )?;
                Ok(
                    json!({"input":args.input,"output":args.output,"revision":revision,"commands_applied":count}),
                )
            }
            "project_import_svg" => {
                let args: ImportSvgArgs = arguments(line)?;
                super::import_svg(
                    Path::new(&args.input),
                    Path::new(&args.output),
                    Path::new(&args.svg),
                )
            }
            "project_import_dxf" => {
                let args: ImportDxfArgs = arguments(line)?;
                super::import_dxf(
                    Path::new(&args.input),
                    Path::new(&args.output),
                    Path::new(&args.dxf),
                )
            }
            "project_export_svg" | "project_export_obj" | "project_export_dxf" => {
                let args: FileArgs = arguments(line)?;
                super::distinct(&args.input, &args.output)?;
                let project = Project::load(Path::new(&args.input))?;
                let (kind, unit, up) = match header.name.as_str() {
                    "project_export_svg" => {
                        project.export_svg(Path::new(&args.output))?;
                        ("authored_svg", "mm", "pattern V up; SVG Y inverted")
                    }
                    "project_export_dxf" => {
                        project.export_dxf(Path::new(&args.output))?;
                        ("authored_dxf", "mm", "pattern U = +X, V = +Y; Z = 0")
                    }
                    _ => {
                        project.mesh()?.export_obj(Path::new(&args.output))?;
                        ("rest_obj", "m", "Y")
                    }
                };
                let mut result = json!({"input":args.input,"output":args.output,"source_revision":project.revision,
                    "kind":kind,"length_unit":unit,"up":up});
                if header.name == "project_export_dxf" {
                    result["scope"] = json!(
                        "Exterior pattern boundaries only; sewing, materials, grain and 3D placements require the native project."
                    );
                }
                Ok(result)
            }
            _ => unreachable!("tool name was checked above"),
        }
    };
    Ok(match operation() {
        Ok(data) => {
            json!({"content":[{"type":"text","text":data.to_string()}],"structuredContent":data,"isError":false})
        }
        Err(cause) => {
            let data = json!({"error":cause.to_string()});
            json!({"content":[{"type":"text","text":data.to_string()}],"structuredContent":data,"isError":true})
        }
    })
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}

fn array(items: Value, length: usize) -> Value {
    json!({"type":"array","items":items,"minItems":length,"maxItems":length})
}

fn length_query_schema() -> Value {
    let id = json!({"type":"integer","minimum":1,"maximum":u64::MAX});
    let parameter = json!({"type":"number","minimum":0,"maximum":1});
    let variants = [
        (
            "interval",
            object(
                json!({"curve":id,"t0":parameter,"t1":parameter}),
                &["curve", "t0", "t1"],
            ),
        ),
        ("seam", object(json!({"id":id}), &["id"])),
        ("sewing_group", object(json!({"id":id}), &["id"])),
    ];
    json!({"oneOf":variants.into_iter().map(|(name,data)|object(json!({"type":{"const":name},"data":data}), &["type","data"])).collect::<Vec<_>>()})
}

fn command_schema() -> Value {
    let number = json!({"type":"number"});
    let id = json!({"type":"integer","minimum":0,"maximum":u64::MAX});
    let text = json!({"type":"string"});
    let xy = array(number.clone(), 2);
    let xyz = array(number.clone(), 3);
    let interval = object(
        json!({"curve":id,"t0":number,"t1":number}),
        &["curve", "t0", "t1"],
    );
    let side_by_length = object(
        json!({"curve":id,"start_t":{"type":"number","minimum":0,"maximum":1},
            "direction":{"type":"string","enum":["forward","reverse"]},
            "length_m":{"type":"number","exclusiveMinimum":0,"description":"Positive authored arc length in metres, within the remaining oriented curve."}}),
        &["curve", "start_t", "direction", "length_m"],
    );
    let placement = object(
        json!({"translation":xyz,"rotation":xyz}),
        &["translation", "rotation"],
    );
    let material = object(
        json!({"id":id,"name":text,"areal_density":number,"warp_stiffness":number,
        "weft_stiffness":number,"shear_stiffness":number,"bending_stiffness":number,"damping":number,
        "friction":number,"collision_thickness":number,"color":xyz}),
        &[
            "id",
            "name",
            "areal_density",
            "warp_stiffness",
            "weft_stiffness",
            "shear_stiffness",
            "bending_stiffness",
            "damping",
            "friction",
            "collision_thickness",
            "color",
        ],
    );
    let correspondence = json!({"type":"array","items":xy,"minItems":2,"maxItems":1024});
    let seam = object(
        json!({"id":id,"name":text,"a":interval,"b":interval,"correspondence":correspondence}),
        &["id", "name", "a", "b", "correspondence"],
    );
    let path = json!({"type":"array","items":interval,"minItems":1,"maxItems":1024});
    let sewing_group = object(
        json!({"id":id,"name":text,"a":path,"b":path,"correspondence":correspondence}),
        &["id", "name", "a", "b", "correspondence"],
    );
    let mannequin = object(
        json!({"capsules":{"type":"array","items":object(json!({"a":xyz,"b":xyz,"radius":number}), &["a","b","radius"])}}),
        &["capsules"],
    );
    let cut = object(
        json!({
            "piece":{"type":"integer","minimum":1,"maximum":u64::MAX},
            "start_curve":{"type":"integer","minimum":1,"maximum":u64::MAX},
            "start_t":{"type":"number","exclusiveMinimum":1e-6,"exclusiveMaximum":1.-1e-6},
            "end_curve":{"type":"integer","minimum":1,"maximum":u64::MAX},
            "end_t":{"type":"number","exclusiveMinimum":1e-6,"exclusiveMaximum":1.-1e-6}
        }),
        &["piece", "start_curve", "start_t", "end_curve", "end_t"],
    );
    let variants = [
        (
            "import_dxf_patterns",
            object(
                json!({"dxf":{"type":"string","maxLength":weftforge_core::MAX_DXF_BYTES}}),
                &["dxf"],
            ),
        ),
        (
            "import_svg_patterns",
            object(
                json!({"svg":{"type":"string","maxLength":weftforge_core::MAX_SVG_BYTES}}),
                &["svg"],
            ),
        ),
        (
            "add_polygon",
            object(
                json!({"name":text,"vertices":{"type":"array","items":xy}}),
                &["name", "vertices"],
            ),
        ),
        (
            "add_rectangle",
            object(
                json!({"name":text,"origin":xy,"width":number,"height":number}),
                &["name", "origin", "width", "height"],
            ),
        ),
        (
            "add_ellipse",
            object(
                json!({"name":text,"origin":xy,"width":number,"height":number}),
                &["name", "origin", "width", "height"],
            ),
        ),
        (
            "add_hole",
            object(
                json!({"piece":{"type":"integer","minimum":1,"maximum":u64::MAX},
                    "vertices":{"type":"array","minItems":3,"maxItems":4096,
                        "description":"Finite authored U/V metres; one simple straight hole strictly inside an existing straight piece. Either winding is accepted; the first vertex is retained.",
                        "items":array(json!({"type":"number","minimum":-1000,"maximum":1000}),2)}}),
                &["piece", "vertices"],
            ),
        ),
        (
            "remove_hole",
            object(
                json!({"hole":{"type":"integer","minimum":1,"maximum":u64::MAX,
                    "description":"Stable Hole ID; fills its opening and removes only the hole's owned geometry."}}),
                &["hole"],
            ),
        ),
        (
            "add_internal_segment",
            object(
                json!({"piece":{"type":"integer","minimum":1,"maximum":u64::MAX},
                    "start":array(json!({"type":"number","minimum":-1000,"maximum":1000}),2),
                    "end":array(json!({"type":"number","minimum":-1000,"maximum":1000}),2)}),
                &["piece", "start", "end"],
            ),
        ),
        (
            "remove_internal_curve",
            object(
                json!({"curve":{"type":"integer","minimum":1,"maximum":u64::MAX}}),
                &["curve"],
            ),
        ),
        (
            "move_point",
            object(json!({"point":id,"position":xy}), &["point", "position"]),
        ),
        (
            "set_curve_controls",
            object(
                json!({"curve":id,"controls":{"anyOf":[{"type":"null"},array(xy.clone(),2)]}}),
                &["curve"],
            ),
        ),
        (
            "split_curve",
            object(json!({"curve":id,"t":number}), &["curve", "t"]),
        ),
        ("cut_piece", cut.clone()),
        ("cut_and_sew", cut),
        (
            "offset_outline",
            object(
                json!({
                    "piece":{"type":"integer","minimum":1,"maximum":u64::MAX},
                    "width":{"type":"number","exclusiveMinimum":0,"description":"Positive offset in metres; changes the actual material outline, rest mesh and cloth mass."},
                    "outward":{"type":"boolean","description":"True extends the material outline; false retracts it. Unsupported attached construction and invalid or unproved geometry refuse atomically."}
                }),
                &["piece", "width", "outward"],
            ),
        ),
        ("join_curve", object(json!({"curve":id}), &["curve"])),
        (
            "resize_piece",
            object(
                json!({"piece":id,"width":number,"height":number}),
                &["piece", "width", "height"],
            ),
        ),
        (
            "transform_piece",
            object(
                json!({"piece":{"type":"integer","minimum":1,"maximum":u64::MAX},
                    "translation":array(json!({"type":"number","minimum":-1000,"maximum":1000,"description":"Metres in authored U/V; rotation is applied before translation."}),2),
                    "angle":{"type":"number","minimum":-1e6,"maximum":1e6,"description":"Counterclockwise radians; placement is compensated and grain rotates to preserve world arrangement."},
                    "pivot":array(json!({"type":"number","minimum":-1000,"maximum":1000,"description":"Explicit pivot in original authored U/V metres."}),2)}),
                &["piece", "translation", "angle", "pivot"],
            ),
        ),
        (
            "set_placement",
            object(
                json!({"piece":id,"placement":placement}),
                &["piece", "placement"],
            ),
        ),
        (
            "set_grain",
            object(json!({"piece":id,"angle":number}), &["piece", "angle"]),
        ),
        (
            "set_material",
            object(json!({"piece":id,"material":id}), &["piece", "material"]),
        ),
        (
            "add_material",
            object(json!({"material":material}), &["material"]),
        ),
        (
            "update_material",
            object(json!({"material":material}), &["material"]),
        ),
        (
            "add_seam",
            object(
                json!({"name":text,"a":interval,"b":interval,"correspondence":correspondence}),
                &["name", "a", "b", "correspondence"],
            ),
        ),
        ("update_seam", object(json!({"seam":seam}), &["seam"])),
        ("remove_seam", object(json!({"seam":id}), &["seam"])),
        (
            "add_sewing_group",
            object(
                json!({"name":text,"a":path,"b":path,"correspondence":correspondence}),
                &["name", "a", "b", "correspondence"],
            ),
        ),
        (
            "add_sewing_by_length",
            object(
                json!({"name":text,"a":side_by_length,"b":side_by_length}),
                &["name", "a", "b"],
            ),
        ),
        (
            "update_sewing_group",
            object(json!({"group":sewing_group}), &["group"]),
        ),
        (
            "remove_sewing_group",
            object(json!({"group":id}), &["group"]),
        ),
        ("remove_piece", object(json!({"piece":id}), &["piece"])),
        (
            "set_resolution",
            object(json!({"metres":number}), &["metres"]),
        ),
        (
            "set_mannequin",
            object(json!({"mannequin":mannequin}), &["mannequin"]),
        ),
    ];
    json!({"oneOf":variants.into_iter().map(|(name,data)|object(json!({"type":{"const":name},"data":data}), &["type","data"])).collect::<Vec<_>>()})
}

fn tools() -> Vec<Value> {
    let path = json!({"type":"string","minLength":1});
    let files = object(json!({"input":path,"output":path}), &["input", "output"]);
    [
        ("project_inspect", "Load a validated native project; return every authored object and stable relationship in SI units, without building a mesh.", object(json!({"path":path}), &["path"]), true),
        ("project_lengths", "Read authored/rest interval, legacy seam or ordered sewing-group lengths in unrounded SI metres, with source revision and signed B minus A. No mesh, simulation, history or file writes.", object(json!({"path":path,"query":length_query_schema()}), &["path","query"]), true),
        ("project_create", "Save an empty native project to a new path. Existing files are refused.", object(json!({"output":path}), &["output"]), false),
        ("project_execute", "Load input, execute 1–1024 shared typed commands atomically as one document revision, then save output with native atomic/recovery handling. Input and output may be the same native file. Inspect stable IDs before edits; nested transactions are rejected.", object(json!({"input":path,"output":path,"commands":{"type":"array","minItems":1,"maxItems":MAX_COMMANDS,"items":command_schema()}}), &["input","output","commands"]), false),
        ("project_import_svg", "Add closed straight/cubic SVG exterior patterns as one editable transaction. Compound paths with holes are unsupported and refused. Require physical mm width/height and a uniform viewBox scale; retain the SVG user-space origin and invert Y. Import names with first material/default 3D placement; omit appearance, sewing, grain and simulation. Unsupported content fails the entire import; source SVG is preserved.", object(json!({"input":path,"output":path,"svg":path}), &["input","output","svg"]), false),
        ("project_import_dxf", "Add the bounded FILE-005-v1 R2000 ASCII DXF LINE/clamped cubic SPLINE exterior patterns as one editable transaction. Require millimetres and exact closed ordered WF_PIECE layer contours; preserve WCS X/Y layout as U/V without inversion. Layer names become names with fresh IDs, first material and default 3D placement. Unsupported drawing content fails the entire import; source DXF is preserved.", object(json!({"input":path,"output":path,"dxf":path}), &["input","output","dxf"]), false),
        ("project_export_svg", "Export one compound evenodd path per piece with closed exterior and hole subpaths at physical millimetre scale, with inverted SVG Y; omit internal annotations. Compound paths with holes cannot yet be reimported. Refuse replacement of the source or native project destinations.", files.clone(), false),
        ("project_export_obj", "Build and export the actual rest surface, including holes, in metres, Y-up, with material-space metre UVs and source revision; no simulation or annotation line primitives. Refuse replacement of the source or native project destinations.", files.clone(), false),
        ("project_export_dxf", "Export authored exterior pattern boundaries as R2000 DXF LINE/SPLINE geometry in millimetres, without 3D placement. DXF v1 refuses any project with holes before writing output. Internal annotations are omitted; sewing, materials, grain and editable annotations require the native project. Refuse replacement of source or native project destinations.", files, false),
    ].into_iter().map(|(name,description,input,read_only)|json!({"name":name,"description":description,"inputSchema":input,"annotations":{"readOnlyHint":read_only,"destructiveHint":!read_only,"openWorldHint":false}})).collect()
}
