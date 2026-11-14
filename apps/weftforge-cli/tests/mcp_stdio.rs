use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command as Process, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use weftforge_core::{Command, Editor, Project, recovery_path};

struct Server {
    child: Child,
    input: Option<ChildStdin>,
    lines: Receiver<String>,
    next_id: u64,
}

impl Server {
    fn start() -> Self {
        let mut child = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let output = child.stdout.take().unwrap();
        let (sender, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                if sender.send(line.unwrap()).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            lines,
            next_id: 1,
        }
    }

    fn send(&mut self, line: &str) {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{line}").unwrap();
        input.flush().unwrap();
    }

    fn receive(&self) -> Value {
        let line = self
            .lines
            .recv_timeout(Duration::from_secs(10))
            .expect("MCP response timed out");
        serde_json::from_str(&line)
            .unwrap_or_else(|error| panic!("Non-protocol stdout: {line}: {error}"))
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}).to_string());
        let response = self.receive();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], id);
        response
    }

    fn initialize(&mut self, version: &str) {
        let response = self.request("initialize", json!({"protocolVersion":version,"capabilities":{},"clientInfo":{"name":"stdio-test","version":"1"}}));
        assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
        assert_eq!(response["result"]["capabilities"], json!({"tools":{}}));
        self.send(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
    }

    fn call(&mut self, name: &str, arguments: Value) -> Value {
        self.request("tools/call", json!({"name":name,"arguments":arguments}))
    }

    fn finish(&mut self) {
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline, "MCP did not exit on EOF");
            thread::sleep(Duration::from_millis(10));
        }
        let mut stderr = String::new();
        std::io::Read::read_to_string(&mut self.child.stderr.take().unwrap(), &mut stderr).unwrap();
        assert!(stderr.is_empty(), "Unexpected stderr: {stderr}");
        assert!(
            self.lines.recv_timeout(Duration::from_secs(1)).is_err(),
            "Stray stdout after final response"
        );
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weftforge-mcp-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn success(response: &Value) -> &Value {
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["isError"], false, "{response}");
    let data = &response["result"]["structuredContent"];
    assert_eq!(
        serde_json::from_str::<Value>(response["result"]["content"][0]["text"].as_str().unwrap())
            .unwrap(),
        *data
    );
    data
}

fn tool_error(response: &Value) {
    assert_eq!(response["result"]["isError"], true, "{response}");
    assert!(
        response["result"]["structuredContent"]["error"]
            .as_str()
            .is_some_and(|message| !message.is_empty())
    );
}

fn attribute<'a>(element: &'a str, name: &str) -> &'a str {
    element
        .split_once(&format!(" {name}=\""))
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
}

fn numbers(line: &str) -> Vec<f64> {
    line.split_whitespace()
        .map(|number| {
            let number = number.parse::<f64>().unwrap();
            assert!(number.is_finite());
            number
        })
        .collect()
}

fn span(points: &[Vec<f64>], axis: usize) -> f64 {
    points
        .iter()
        .map(|point| point[axis])
        .reduce(f64::max)
        .unwrap()
        - points
            .iter()
            .map(|point| point[axis])
            .reduce(f64::min)
            .unwrap()
}

// Independent strict token parsers for this original, straight-edged rectangle fixture.
fn check_rectangle_exports(obj: &Path, svg: &Path) {
    let svg = fs::read_to_string(svg).unwrap();
    let root = svg.lines().find(|line| line.starts_with("<svg ")).unwrap();
    assert_eq!(attribute(root, "xmlns"), "http://www.w3.org/2000/svg");
    let viewbox = numbers(attribute(root, "viewBox"));
    assert_eq!(viewbox.len(), 4);
    for (name, extent) in [("width", viewbox[2]), ("height", viewbox[3])] {
        assert!(extent > 0.0);
        assert_eq!(
            attribute(root, name)
                .strip_suffix("mm")
                .unwrap()
                .parse::<f64>()
                .unwrap(),
            extent
        );
    }
    let paths: Vec<_> = svg
        .lines()
        .filter(|line| line.starts_with("<path "))
        .collect();
    assert_eq!(paths.len(), 1);
    assert_eq!(
        attribute(paths[0], "data-name"),
        "Original &lt;Front &amp; panel&gt;"
    );
    let mut data = attribute(paths[0], "d").split_whitespace();
    let mut points = Vec::new();
    for command in ["M", "L", "L", "L", "L"] {
        assert_eq!(data.next(), Some(command));
        points.push(numbers(&format!(
            "{} {}",
            data.next().unwrap(),
            data.next().unwrap()
        )));
    }
    assert_eq!(data.next(), Some("Z"));
    assert!(data.next().is_none());
    assert!((span(&points, 0) - 600.).abs() < 1e-8);
    assert!((span(&points, 1) - 800.).abs() < 1e-8);
    assert!(svg.ends_with("</svg>\n"));
    let obj = fs::read_to_string(obj).unwrap();
    assert!(obj.starts_with("# WeftForge OBJ: metres; Y up;"));
    let mut vertices = Vec::new();
    let mut uv = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();
    let mut groups = 0;
    for line in obj.lines().filter(|line| !line.starts_with('#')) {
        let (kind, data) = line.split_once(' ').unwrap();
        match kind {
            "v" => vertices.push(numbers(data)),
            "vt" => uv.push(numbers(data)),
            "vn" => normals.push(numbers(data)),
            "g" => {
                assert!(data.starts_with("piece_"));
                groups += 1;
            }
            "f" => faces.push(
                data.split_whitespace()
                    .map(|vertex| {
                        vertex
                            .split('/')
                            .map(|index| index.parse::<usize>().unwrap())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => panic!("Unexpected OBJ row: {line}"),
        }
    }
    assert!(!vertices.is_empty() && !uv.is_empty() && !normals.is_empty() && !faces.is_empty());
    assert_eq!(groups, 1);
    assert!(
        vertices
            .iter()
            .all(|point| point.len() == 3 && point[2] == 0.)
    );
    assert!(uv.iter().all(|point| point.len() == 2));
    assert!((span(&vertices, 0) - 0.6).abs() < 1e-8);
    assert!((span(&vertices, 1) - 0.8).abs() < 1e-8);
    assert!(
        normals.iter().all(|point| point.len() == 3
            && (point.iter().map(|v| v * v).sum::<f64>() - 1.).abs() < 1e-8)
    );
    for face in faces {
        assert_eq!(face.len(), 3);
        for indices in face {
            assert_eq!(indices.len(), 3);
            assert!((1..=vertices.len()).contains(&indices[0]));
            assert!((1..=uv.len()).contains(&indices[1]));
            assert!((1..=normals.len()).contains(&indices[2]));
        }
    }
}

#[test]
fn stdio_authors_atomic_shared_commands_and_exports_physical_units() {
    let directory = Directory::new();
    let project_path = directory.path("original.weftforge");
    let obj = directory.path("rest.obj");
    let svg = directory.path("patterns.svg");
    let dxf = directory.path("patterns.dxf");
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 6);
    assert!(
        tools
            .iter()
            .all(|tool| tool["inputSchema"]["additionalProperties"] == false)
    );
    let execute = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap();
    assert_eq!(
        execute["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
            .as_array()
            .unwrap()
            .len(),
        18
    );
    success(&server.call("project_create", json!({"output":project_path})));
    let initial = server.call("project_inspect", json!({"path":project_path}));
    assert_eq!(success(&initial)["project"], json!(Project::default()));
    let piece = initial["result"]["structuredContent"]["project"]["next_id"]
        .as_u64()
        .unwrap();
    let commands = vec![
        Command::AddRectangle {
            name: "Original <Front & panel>".into(),
            origin: [0.1, 0.2],
            width: 0.6,
            height: 0.8,
        },
        Command::SetGrain { piece, angle: 0.37 },
    ];
    let result = server.call(
        "project_execute",
        json!({"input":project_path,"output":project_path,"commands":commands}),
    );
    assert_eq!(success(&result)["revision"], 1);
    assert_eq!(success(&result)["commands_applied"], 2);
    let mut expected = Editor::default();
    expected.execute(Command::Transaction(commands)).unwrap();
    assert_eq!(Project::load(&project_path).unwrap(), expected.project);
    assert_eq!(Project::recover(&project_path).unwrap(), Project::default());
    let reopened = server.call("project_inspect", json!({"path":project_path}));
    assert_eq!(success(&reopened)["project"], json!(expected.project));
    assert_eq!(
        success(&reopened)["units"],
        json!({"length":"m","mass":"kg","time":"s"})
    );
    assert_eq!(
        success(&server.call(
            "project_export_obj",
            json!({"input":project_path,"output":obj})
        ))["source_revision"],
        1
    );
    assert_eq!(
        success(&server.call(
            "project_export_svg",
            json!({"input":project_path,"output":svg})
        ))["length_unit"],
        "mm"
    );
    check_rectangle_exports(&obj, &svg);
    let source_bytes = fs::read(&project_path).unwrap();
    let exported = server.call(
        "project_export_dxf",
        json!({"input":project_path,"output":dxf}),
    );
    let exported = success(&exported);
    assert_eq!(exported["source_revision"], 1);
    assert_eq!(exported["kind"], "authored_dxf");
    assert_eq!(exported["length_unit"], "mm");
    assert_eq!(exported["up"], "pattern U = +X, V = +Y; Z = 0");
    assert!(
        exported["scope"]
            .as_str()
            .unwrap()
            .contains("native project")
    );
    let cli_dxf = directory.path("cli-patterns.dxf");
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("export-dxf")
        .arg(&project_path)
        .arg(&cli_dxf)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let receipt: Value = serde_json::from_slice(&cli.stdout).unwrap();
    assert_eq!(receipt["source_revision"], 1);
    assert_eq!(receipt["kind"], "authored_dxf");
    assert_eq!(receipt["length_unit"], "mm");
    assert_eq!(fs::read(&cli_dxf).unwrap(), fs::read(&dxf).unwrap());
    assert_eq!(fs::read(&project_path).unwrap(), source_bytes);
    server.finish();
}

#[test]
fn stdio_advertises_and_executes_join_atomically_without_overwriting_failed_output() {
    use weftforge_core::CurveInterval;
    let directory = Directory::new();
    let input = directory.path("split.weftforge");
    let output = directory.path("joined.weftforge");
    let mut expected = Editor::default();
    for x in [0.0, 1.0] {
        expected
            .execute(Command::AddRectangle {
                name: "Panel".into(),
                origin: [x, 0.0],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
    }
    let curve = expected.project.pieces[0].boundary[2];
    let other = expected.project.pieces[1].boundary[2];
    expected
        .execute(Command::Transaction(vec![
            Command::SetCurveControls {
                curve,
                controls: Some([[0.3, 0.8], [0.1, 0.8]]),
            },
            Command::AddSeam {
                name: "Sewn cubic".into(),
                a: CurveInterval::full(curve),
                b: CurveInterval::reversed(other),
                correspondence: vec![[0.0, 0.0], [0.4, 0.65], [1.0, 1.0]],
            },
            Command::SplitCurve { curve, t: 0.4 },
        ]))
        .unwrap();
    expected.project.save(&input).unwrap();
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let schema = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "join_curve")
        .unwrap();
    assert_eq!(schema["properties"]["data"]["required"], json!(["curve"]));
    assert_eq!(schema["properties"]["data"]["additionalProperties"], false);
    assert_eq!(
        schema["properties"]["data"]["properties"]
            .as_object()
            .unwrap()
            .len(),
        1
    );
    let command = Command::JoinCurve { curve };
    let before_revision = expected.project.revision;
    let response = server.call(
        "project_execute",
        json!({"input":input,"output":output,"commands":[command]}),
    );
    assert_eq!(success(&response)["revision"], before_revision + 1);
    expected
        .execute(Command::Transaction(vec![command]))
        .unwrap();
    assert_eq!(Project::load(&output).unwrap(), expected.project);
    let valid = fs::read(&output).unwrap();
    for commands in [
        json!([{"type":"join_curve","data":{"curve":curve,"next":other}}]),
        json!([{"type":"join_curve","data":{"curve":other}}]),
        json!([{"type":"join_curve","data":{"curve":curve,"t":0.4}}]),
    ] {
        let response = server.call(
            "project_execute",
            json!({"input":output,"output":output,"commands":commands}),
        );
        assert!(response.get("error").is_some() || response["result"]["isError"] == true);
        assert_eq!(fs::read(&output).unwrap(), valid);
    }
    server.finish();
}

#[test]
fn stdio_errors_preserve_input_output_and_recovery_and_allow_retry() {
    let directory = Directory::new();
    let input = directory.path("input.weftforge");
    let output = directory.path("output.weftforge");
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "Original".into(),
            origin: [0., 0.],
            width: 0.3,
            height: 0.4,
        })
        .unwrap();
    editor.project.save(&input).unwrap();
    Project::default().save(&output).unwrap();
    editor.project.save(&output).unwrap();
    let original = fs::read(&input).unwrap();
    let previous_output = fs::read(&output).unwrap();
    let previous_recovery = fs::read(recovery_path(&output)).unwrap();
    let piece = editor.project.pieces[0].id;
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let valid = json!(Command::SetGrain { piece, angle: 0.5 });
    let invalid = json!(Command::ResizePiece {
        piece: u64::MAX,
        width: 0.2,
        height: 0.4
    });
    let failures = [
        json!([]),
        json!([valid.clone(), invalid]),
        json!([{"type":"unknown","data":{}}]),
        json!([{"type":"transaction","data":[valid.clone()]}]),
        json!([{"type":"set_grain","data":{"piece":piece,"angle":0.5,"unknown":true}}]),
        json!(vec![valid.clone(); 1025]),
    ];
    for commands in failures {
        tool_error(&server.call(
            "project_execute",
            json!({"input":input,"output":output,"commands":commands}),
        ));
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&output).unwrap(), previous_output);
        assert_eq!(fs::read(recovery_path(&output)).unwrap(), previous_recovery);
    }
    for duplicate in ["\"angle\":0.5,\"angle\":0.8", "\"angle\":0.5,\"piece\":999"] {
        let request = json!({"jsonrpc":"2.0","id":"duplicate-command","method":"tools/call","params":{"name":"project_execute","arguments":{"input":input,"output":output,"commands":[valid.clone()]}}}).to_string().replace("\"angle\":0.5", duplicate);
        server.send(&request);
        tool_error(&server.receive());
        assert_eq!(fs::read(&output).unwrap(), previous_output);
    }
    let duplicate_tag = json!({"jsonrpc":"2.0","id":"duplicate-tag","method":"tools/call","params":{"name":"project_execute","arguments":{"input":input,"output":output,"commands":[valid]}}}).to_string().replace("\"type\":\"set_grain\"", "\"type\":\"set_grain\",\"type\":\"remove_piece\"");
    server.send(&duplicate_tag);
    tool_error(&server.receive());
    tool_error(&server.call("project_create", json!({"output":output})));
    for destination in [&input, &output] {
        for tool in [
            "project_export_svg",
            "project_export_obj",
            "project_export_dxf",
        ] {
            tool_error(&server.call(tool, json!({"input":input,"output":destination})));
        }
    }
    let future = directory.path("future.weftforge");
    let mut future_document: Value = serde_json::from_slice(&original).unwrap();
    future_document["version"] = json!(u32::MAX);
    fs::write(&future, future_document.to_string()).unwrap();
    tool_error(&server.call("project_execute", json!({"input":future,"output":output,"commands":[Command::SetResolution { metres: 0.04 }]})));
    tool_error(&server.call(
        "project_export_svg",
        json!({"input":future,"output":output}),
    ));
    let duplicate_native = directory.path("duplicate.weftforge");
    fs::write(
        &duplicate_native,
        String::from_utf8(original.clone())
            .unwrap()
            .replace("\"revision\": 1", "\"revision\": 1, \"revision\": 2"),
    )
    .unwrap();
    tool_error(&server.call("project_inspect", json!({"path":duplicate_native})));
    let invalid_target = directory.path("keep.weftforge");
    fs::write(&invalid_target, b"user data").unwrap();
    tool_error(&server.call("project_execute", json!({"input":input,"output":invalid_target,"commands":[Command::SetResolution { metres: 0.04 }]})));
    assert_eq!(fs::read(&invalid_target).unwrap(), b"user data");
    let missing_parent = directory.path("missing/out.weftforge");
    tool_error(&server.call("project_execute", json!({"input":input,"output":missing_parent,"commands":[Command::SetResolution { metres: 0.04 }]})));
    let folder = directory.path("folder.weftforge");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("retained"), b"folder contents").unwrap();
    fs::write(recovery_path(&folder), &previous_recovery).unwrap();
    tool_error(&server.call(
        "project_execute",
        json!({"input":input,"output":folder,"commands":[Command::SetResolution { metres: 0.04 }]}),
    ));
    assert_eq!(
        fs::read(folder.join("retained")).unwrap(),
        b"folder contents"
    );
    assert_eq!(fs::read(recovery_path(&folder)).unwrap(), previous_recovery);
    #[cfg(unix)]
    for (name, target) in [
        ("valid.weftforge", input.clone()),
        ("dangling.weftforge", directory.path("absent")),
    ] {
        let destination = directory.path(name);
        std::os::unix::fs::symlink(&target, &destination).unwrap();
        fs::write(recovery_path(&destination), &previous_recovery).unwrap();
        tool_error(&server.call("project_execute", json!({"input":input,"output":destination,"commands":[Command::SetResolution { metres: 0.04 }]})));
        assert!(
            fs::symlink_metadata(&destination)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_link(&destination).unwrap(), target);
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(
            fs::read(recovery_path(&destination)).unwrap(),
            previous_recovery
        );
    }
    assert!(!directory.path("absent").exists());
    tool_error(&server.call(
        "project_export_obj",
        json!({"input":input,"output":directory.0}),
    ));
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read(&output).unwrap(), previous_output);
    assert_eq!(fs::read(recovery_path(&output)).unwrap(), previous_recovery);
    assert_eq!(
        success(&server.call("project_inspect", json!({"path":input})))["project"],
        json!(editor.project)
    );
    server.finish();
}

fn modern_metadata(version: &str) -> Value {
    json!({"io.modelcontextprotocol/protocolVersion":version,"io.modelcontextprotocol/clientCapabilities":{}})
}

#[test]
fn stdio_validates_protocol_lifecycle_modern_metadata_and_line_bounds() {
    let directory = Directory::new();
    let path = directory.path("modern.weftforge");
    let mut server = Server::start();
    assert_eq!(
        server.request("tools/list", json!({}))["error"]["code"],
        -32602
    );
    let wrong = server.request(
        "server/discover",
        json!({"_meta":modern_metadata("2099-01-01")}),
    );
    assert_eq!(wrong["error"]["code"], -32022);
    assert_eq!(wrong["error"]["data"]["requested"], "2099-01-01");
    let discovered = server.request(
        "server/discover",
        json!({"_meta":modern_metadata("2026-07-28")}),
    );
    assert_eq!(
        discovered["result"]["supportedVersions"],
        json!(["2026-07-28"])
    );
    assert_eq!(discovered["result"]["capabilities"], json!({"tools":{}}));
    assert_eq!(discovered["result"]["resultType"], "complete");
    assert_eq!(
        server.request(
            "server/discover",
            json!({"_meta":modern_metadata("2025-11-25")})
        )["error"]["data"]["supported"],
        json!(["2026-07-28"])
    );
    let listed = server.request("tools/list", json!({"_meta":modern_metadata("2026-07-28")}));
    assert_eq!(listed["result"]["tools"].as_array().unwrap().len(), 6);
    assert_eq!(listed["result"]["cacheScope"], "public");
    assert_eq!(listed["result"]["ttlMs"], 0);
    success(&server.request("tools/call", json!({"_meta":modern_metadata("2026-07-28"),"name":"project_create","arguments":{"output":path}})));
    let modern = server.request("tools/call", json!({"_meta":modern_metadata("2026-07-28"),"name":"project_inspect","arguments":{"path":path}}));
    assert_eq!(success(&modern)["project"], json!(Project::default()));
    assert_eq!(modern["result"]["resultType"], "complete");
    assert_eq!(
        server.request("tools/list", json!({}))["error"]["code"],
        -32602
    );
    assert_eq!(
        server.request(
            "tools/list",
            json!({"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}})
        )["error"]["code"],
        -32602
    );
    assert_eq!(
        server.request(
            "tools/list",
            json!({"_meta":{"io.modelcontextprotocol/clientCapabilities":{}}})
        )["error"]["code"],
        -32602
    );
    server.initialize("2099-01-01");
    assert_eq!(server.request("ping", json!({}))["result"], json!({}));
    assert_eq!(
        server.request("resources/list", json!({}))["error"]["code"],
        -32601
    );
    assert_eq!(
        server.request("tools/list", json!({"cursor":"unknown"}))["error"]["code"],
        -32602
    );
    assert_eq!(
        server.request("tools/list", json!({"typo":true}))["error"]["code"],
        -32602
    );
    assert_eq!(
        server.request("tools/call", json!({"name":"missing","arguments":{}}))["error"]["code"],
        -32602
    );
    assert_eq!(
        server.request(
            "tools/call",
            json!({"name":"project_inspect","arguments":{},"typo":true})
        )["error"]["code"],
        -32602
    );
    tool_error(&server.call("project_inspect", json!({"path":"missing","typo":true})));
    for invalid in [
        r#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#,
        r#"{"jsonrpc":"2.0","id":1.5,"method":"ping"}"#,
        r#"{"jsonrpc":"2.0","id":true,"method":"ping"}"#,
        r#"{"jsonrpc":"2.0","id":90,"id":91,"method":"ping"}"#,
        r#"{"jsonrpc":"2.0","id":90,"method":"ping","extra":true}"#,
        "[]",
    ] {
        server.send(invalid);
        assert_eq!(server.receive()["error"]["code"], -32600, "{invalid}");
    }
    server.send(r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":null}"#);
    assert_eq!(server.receive()["error"]["code"], -32602);
    server.send("{");
    assert_eq!(server.receive()["error"]["code"], -32700);
    server.send(&"x".repeat(1024 * 1024 + 1));
    assert_eq!(server.receive()["error"]["code"], -32600);
    let ping = r#"{"jsonrpc":"2.0","id":"boundary","method":"ping"}"#;
    server.send(&format!(
        "{ping}{}",
        " ".repeat(1024 * 1024 - ping.len() - 1)
    ));
    assert_eq!(server.receive()["id"], "boundary");
    server.send(r#"{"jsonrpc":"2.0","method":"notifications/unknown"}"#);
    server.send(
        r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"finished"}}"#,
    );
    assert_eq!(server.request("ping", json!({}))["result"], json!({}));
    server
        .input
        .as_mut()
        .unwrap()
        .write_all(br#"{"jsonrpc":"2.0","id":"last","method":"ping"}"#)
        .unwrap();
    server.input.take();
    assert_eq!(server.receive()["id"], "last");
    server.finish();
}
