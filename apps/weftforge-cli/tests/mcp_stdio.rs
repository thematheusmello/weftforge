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

fn native_project_json(project: &Project) -> Value {
    serde_json::from_slice::<Value>(&project.to_json().unwrap()).unwrap()["project"].clone()
}

#[test]
fn straight_cut_commands_share_closed_schemas_profiles_and_atomic_saves() {
    for (kind, modern) in [
        ("cut_piece", false),
        ("cut_and_sew", false),
        ("cut_and_sew", true),
    ] {
        check_cut_command(kind, modern);
    }
}

fn check_cut_command(kind: &str, modern: bool) {
    let dir = Directory::new();
    let original =
        include_bytes!("../../../crates/weftforge-core/tests/fixtures/original-cut-r240.weftforge");
    let cli_path = dir.path("cut-cli.weftforge");
    let mcp_path = dir.path("cut-mcp.weftforge");
    let failed_cli = dir.path("refuse-cli.weftforge");
    let failed_mcp = dir.path("refuse-mcp.weftforge");
    let separate_cli = dir.path("separate-cli.weftforge");
    let separate_mcp = dir.path("separate-mcp.weftforge");
    let commands_file = dir.path("commands.json");
    for path in [&cli_path, &mcp_path, &failed_cli, &failed_mcp] {
        fs::write(path, original).unwrap();
    }
    let backup = Project::default().to_json().unwrap();
    for path in [&failed_cli, &failed_mcp] {
        fs::write(recovery_path(path), &backup).unwrap();
    }
    let command = json!({"type":kind,"data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}});
    let run = |input: &Path, output: &Path, raw: &str| {
        fs::write(&commands_file, raw).unwrap();
        Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("execute")
            .arg(input)
            .arg(output)
            .arg(&commands_file)
            .output()
            .unwrap()
    };
    let mut server = Server::start();
    if !modern {
        server.initialize("2025-11-25");
    }
    let profile_params = |mut params: Value| {
        if modern {
            params["_meta"] = modern_metadata("2026-07-28");
        }
        params
    };
    let call = |server: &mut Server, name: &str, arguments: Value| {
        server.request(
            "tools/call",
            profile_params(json!({"name":name,"arguments":arguments})),
        )
    };
    let listed = server.request("tools/list", profile_params(json!({})));
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9);
    let variants = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap();
    assert_eq!(variants.len(), 32);
    let execute = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap();
    assert_eq!(
        execute["inputSchema"]["properties"]["commands"]["minItems"],
        1
    );
    assert_eq!(
        execute["inputSchema"]["properties"]["commands"]["maxItems"],
        1024
    );
    let cut = variants
        .iter()
        .filter(|variant| variant["properties"]["type"]["const"] == kind)
        .collect::<Vec<_>>();
    assert_eq!(cut.len(), 1);
    assert_eq!(cut[0]["required"], json!(["type", "data"]));
    assert_eq!(cut[0]["additionalProperties"], false);
    let schema = &cut[0]["properties"]["data"];
    assert_eq!(
        schema["required"],
        json!(["piece", "start_curve", "start_t", "end_curve", "end_t"])
    );
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"].as_object().unwrap().len(), 5);
    for id in ["piece", "start_curve", "end_curve"] {
        assert_eq!(schema["properties"][id]["type"], "integer");
        assert_eq!(schema["properties"][id]["minimum"], 1);
        assert_eq!(schema["properties"][id]["maximum"], u64::MAX);
    }
    for t in ["start_t", "end_t"] {
        assert_eq!(schema["properties"][t]["type"], "number");
        assert_eq!(schema["properties"][t]["exclusiveMinimum"], 1e-6);
        assert_eq!(schema["properties"][t]["exclusiveMaximum"], 1. - 1e-6);
    }
    let result = run(&cli_path, &cli_path, &json!([command.clone()]).to_string());
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap(),
        json!({"revision":2,"commands_applied":1})
    );
    let response = call(
        &mut server,
        "project_execute",
        json!({"input":mcp_path,"output":mcp_path,"commands":[command.clone()]}),
    );
    assert_eq!(success(&response)["revision"], 2);
    assert_eq!(fs::read(&cli_path).unwrap(), fs::read(&mcp_path).unwrap());
    for path in [&cli_path, &mcp_path] {
        assert_eq!(fs::read(recovery_path(path)).unwrap(), original);
    }
    let saved = Project::load(&mcp_path).unwrap();
    assert_eq!(
        saved
            .pieces
            .iter()
            .map(|piece| piece.id)
            .collect::<Vec<_>>(),
        [2, 15]
    );
    assert_eq!(saved.pieces[0].boundary, [7, 18, 14, 10]);
    assert_eq!(saved.pieces[1].boundary, [12, 8, 9, 19]);
    let saved_bytes = fs::read(&mcp_path).unwrap();
    let sewn = kind == "cut_and_sew";
    let saved_value: Value = serde_json::from_slice(&saved_bytes).unwrap();
    assert_eq!(
        saved_value["project"]["next_id"],
        if sewn { 21 } else { 20 }
    );
    assert_eq!(saved_value["version"], weftforge_core::FORMAT_VERSION);
    let seams = if sewn {
        json!([{"id":20,"name":"Cut seam #20",
            "a":{"curve":18,"t0":0.,"t1":1.},
            "b":{"curve":19,"t0":1.,"t1":0.},
            "correspondence":[[0.,0.],[1.,1.]]}])
    } else {
        json!([])
    };
    assert_eq!(native_project_json(&saved)["seams"], seams);
    if sewn {
        for (id, start, end) in [(18, 11, 13), (19, 17, 16)] {
            let curve = saved.curves.iter().find(|curve| curve.id == id).unwrap();
            assert_eq!((curve.start, curve.end), (start, end));
        }
        for (a, b) in [(11, 16), (13, 17)] {
            let position = |id| {
                saved
                    .points
                    .iter()
                    .find(|point| point.id == id)
                    .unwrap()
                    .position
            };
            assert_eq!(position(a).map(f64::to_bits), position(b).map(f64::to_bits));
        }
    }
    for path in [&separate_cli, &separate_mcp] {
        fs::write(path, &backup).unwrap();
        fs::write(recovery_path(path), b"earlier recovery").unwrap();
    }
    assert!(
        run(
            &failed_cli,
            &separate_cli,
            &json!([command.clone()]).to_string()
        )
        .status
        .success()
    );
    let separate = call(
        &mut server,
        "project_execute",
        json!({"input":failed_mcp,"output":separate_mcp,"commands":[command.clone()]}),
    );
    assert_eq!(success(&separate)["revision"], 2);
    for path in [&separate_cli, &separate_mcp] {
        assert_eq!(fs::read(path).unwrap(), saved_bytes);
        assert_eq!(fs::read(recovery_path(path)).unwrap(), backup);
    }
    let inspected = call(&mut server, "project_inspect", json!({"path":mcp_path}));
    assert_eq!(success(&inspected)["project"], native_project_json(&saved));
    let lengths = call(
        &mut server,
        "project_lengths",
        json!({"path":mcp_path,"query":{"type":"interval","data":{"curve":18,"t0":0.,"t1":1.}}}),
    );
    assert!((success(&lengths)["length_m"].as_f64().unwrap() - 0.36).abs() < 1e-12);

    let mut failures = Vec::new();
    for field in ["piece", "start_curve", "start_t", "end_curve", "end_t"] {
        let mut missing = command.clone();
        missing["data"].as_object_mut().unwrap().remove(field);
        failures.push(json!([missing]).to_string());
        let needle = format!("\"{field}\":{}", command["data"][field]);
        failures.push(
            json!([command.clone()])
                .to_string()
                .replace(&needle, &format!("{needle},{needle}")),
        );
    }
    for field in ["piece", "start_curve", "end_curve"] {
        for value in [
            json!(0),
            json!(-1),
            json!(2.5),
            json!("2"),
            Value::Null,
            json!(true),
            json!(u64::MAX),
        ] {
            let mut bad = command.clone();
            bad["data"][field] = value;
            failures.push(json!([bad]).to_string());
        }
        failures.push(json!([command.clone()]).to_string().replace(
            &format!("\"{field}\":{}", command["data"][field]),
            &format!("\"{field}\":18446744073709551616"),
        ));
    }
    for field in ["start_t", "end_t"] {
        for value in [
            json!(-1),
            json!(0),
            json!(1e-6),
            json!(1. - 1e-6),
            json!(1),
            json!(2),
            Value::Null,
            json!("0.375"),
            json!(true),
        ] {
            let mut bad = command.clone();
            bad["data"][field] = value;
            failures.push(json!([bad]).to_string());
        }
        failures.push(json!([command.clone()]).to_string().replace(
            &format!("\"{field}\":{}", command["data"][field]),
            &format!("\"{field}\":1e309"),
        ));
    }
    let mut same_edge = command.clone();
    same_edge["data"]["end_curve"] = json!(7);
    failures.push(json!([same_edge]).to_string());
    let mut unknown_data = command.clone();
    unknown_data["data"]["unexpected"] = json!(true);
    failures.push(json!([unknown_data]).to_string());
    for field in ["type", "data"] {
        let mut missing = command.clone();
        missing.as_object_mut().unwrap().remove(field);
        failures.push(json!([missing]).to_string());
    }
    let mut outer = command.clone();
    outer["unknown"] = json!(true);
    failures.push(json!([outer]).to_string());
    failures
        .push(json!([command.clone(),{"type":"set_resolution","data":{"metres":0}}]).to_string());
    failures.push(json!([{"type":kind,"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}]).to_string());
    for field in ["type", "data"] {
        let needle = format!("\"{field}\":{}", command[field]);
        failures.push(
            json!([command.clone()])
                .to_string()
                .replace(&needle, &format!("{needle},{needle}")),
        );
    }
    failures.push("[]".into());
    failures.push(json!([{"type":"transaction","data":[command.clone()]}]).to_string());
    failures.push(json!(vec![command.clone(); 1025]).to_string());
    failures.push(format!("[{}]", " ".repeat(1024 * 1024)));
    for raw in failures {
        for output in [&failed_cli, &separate_cli] {
            let result = run(&failed_cli, output, &raw);
            assert!(!result.status.success(), "accepted CLI {raw}");
        }
        for output in [&failed_mcp, &separate_mcp] {
            let request = json!({"jsonrpc":"2.0","id":"raw-cut","method":"tools/call","params":profile_params(json!({"name":"project_execute","arguments":{"input":failed_mcp,"output":output,"commands":"CUT_RAW_COMMANDS"}}))}).to_string().replace("\"CUT_RAW_COMMANDS\"", &raw);
            server.send(&request);
            let response = server.receive();
            if request.len() > 1024 * 1024 {
                assert_eq!(response["error"]["code"], -32600);
            } else {
                tool_error(&response);
            }
        }
        for path in [&failed_cli, &failed_mcp] {
            assert_eq!(fs::read(path).unwrap(), original);
            assert_eq!(fs::read(recovery_path(path)).unwrap(), backup);
        }
        for path in [&separate_cli, &separate_mcp] {
            assert_eq!(fs::read(path).unwrap(), saved_bytes);
            assert_eq!(fs::read(recovery_path(path)).unwrap(), backup);
        }
    }
    if sewn {
        let mut bounded = vec![command.clone()];
        bounded.resize(
            1024,
            json!({"type":"set_resolution","data":{"metres":0.07}}),
        );
        let maximum_cli = dir.path("maximum-cli.weftforge");
        let maximum_mcp = dir.path("maximum-mcp.weftforge");
        let result = run(&failed_cli, &maximum_cli, &json!(bounded).to_string());
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            json!({"revision":2,"commands_applied":1024})
        );
        let result = call(
            &mut server,
            "project_execute",
            json!({"input":failed_mcp,"output":maximum_mcp,"commands":bounded}),
        );
        assert_eq!(success(&result)["commands_applied"], 1024);
        for path in [&maximum_cli, &maximum_mcp] {
            assert_eq!(fs::read(path).unwrap(), saved_bytes);
        }
        let recut = json!([{"type":"cut_and_sew","data":{"piece":2,"start_curve":7,"start_t":0.5,"end_curve":14,"end_t":0.5}}]);
        assert!(
            !run(&cli_path, &cli_path, &recut.to_string())
                .status
                .success()
        );
        tool_error(&call(
            &mut server,
            "project_execute",
            json!({"input":mcp_path,"output":mcp_path,"commands":recut}),
        ));
        for path in [&cli_path, &mcp_path] {
            assert_eq!(fs::read(path).unwrap(), saved_bytes);
            assert_eq!(fs::read(recovery_path(path)).unwrap(), original);
        }
        let remove = json!([{"type":"remove_seam","data":{"seam":20}}]);
        assert!(
            run(&cli_path, &cli_path, &remove.to_string())
                .status
                .success()
        );
        let result = call(
            &mut server,
            "project_execute",
            json!({"input":mcp_path,"output":mcp_path,"commands":remove}),
        );
        assert_eq!(success(&result)["revision"], 3);
        let mut removed = saved.clone();
        removed.seams.clear();
        removed.revision = 3;
        for path in [&cli_path, &mcp_path] {
            assert_eq!(Project::load(path).unwrap(), removed);
            assert_eq!(fs::read(recovery_path(path)).unwrap(), saved_bytes);
        }
    }
    for path in [&failed_cli, &failed_mcp, &separate_cli, &separate_mcp] {
        fs::remove_file(recovery_path(path)).unwrap();
        fs::create_dir(recovery_path(path)).unwrap();
    }
    for output in [&failed_cli, &separate_cli] {
        assert!(
            !run(&failed_cli, output, &json!([command.clone()]).to_string())
                .status
                .success()
        );
    }
    for output in [&failed_mcp, &separate_mcp] {
        tool_error(&call(
            &mut server,
            "project_execute",
            json!({"input":failed_mcp,"output":output,"commands":[command.clone()]}),
        ));
    }
    for path in [&failed_cli, &failed_mcp] {
        assert_eq!(fs::read(path).unwrap(), original);
        assert!(recovery_path(path).is_dir());
    }
    for path in [&separate_cli, &separate_mcp] {
        assert_eq!(fs::read(path).unwrap(), saved_bytes);
        assert!(recovery_path(path).is_dir());
    }
    let l_path = dir.path("concave.weftforge");
    let l_bytes =
        include_bytes!("../../../crates/weftforge-core/tests/fixtures/original-cut-l.weftforge");
    fs::write(&l_path, l_bytes).unwrap();
    let invalid = json!([{"type":kind,"data":{"piece":2,"start_curve":11,"start_t":1./3.,"end_curve":12,"end_t":2./3.}}]);
    assert!(!run(&l_path, &l_path, &invalid.to_string()).status.success());
    tool_error(&call(
        &mut server,
        "project_execute",
        json!({"input":l_path,"output":l_path,"commands":invalid}),
    ));
    assert_eq!(fs::read(&l_path).unwrap(), l_bytes);
    assert!(!recovery_path(&l_path).exists());
    server.finish();
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
    assert_eq!(tools.len(), 9);
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
        32
    );
    success(&server.call("project_create", json!({"output":project_path})));
    let initial = server.call("project_inspect", json!({"path":project_path}));
    assert_eq!(
        success(&initial)["project"],
        native_project_json(&Project::default())
    );
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
    assert_eq!(
        success(&reopened)["project"],
        native_project_json(&expected.project)
    );
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
fn sewing_groups_share_cli_and_mcp_transactions_and_inspect_actual_rest_constraints() {
    use weftforge_core::CurveInterval;
    let directory = Directory::new();
    let input = directory.path("original.weftforge");
    let created = directory.path("created.weftforge");
    let cli_created = directory.path("cli-created.weftforge");
    let updated = directory.path("updated.weftforge");
    let removed = directory.path("removed.weftforge");
    let commands_file = directory.path("commands.json");
    let mut editor = Editor::default();
    for (index, width) in [0.1, 0.3, 0.2, 0.1, 0.3].into_iter().enumerate() {
        editor
            .execute(Command::AddRectangle {
                name: format!("Panel {index}"),
                origin: [index as f64, 0.0],
                width,
                height: 0.2,
            })
            .unwrap();
    }
    editor.project.save(&input).unwrap();
    let source = fs::read(&input).unwrap();
    let intervals: Vec<_> = editor
        .project
        .pieces
        .iter()
        .map(|piece| CurveInterval::full(piece.boundary[0]))
        .collect();
    let command = Command::AddSewingGroup {
        name: "Ordered 2:3".into(),
        a: intervals[..2].to_vec(),
        b: intervals[2..].to_vec(),
        correspondence: vec![[0.0, 0.0], [1.0, 1.0]],
    };
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let variants = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap();
    assert_eq!(variants.len(), 32);
    for name in [
        "add_sewing_group",
        "update_sewing_group",
        "remove_sewing_group",
    ] {
        let variant = variants
            .iter()
            .find(|variant| variant["properties"]["type"]["const"] == name)
            .unwrap();
        assert_eq!(variant["properties"]["data"]["additionalProperties"], false);
    }
    let create_schema = variants
        .iter()
        .find(|v| v["properties"]["type"]["const"] == "add_sewing_group")
        .unwrap();
    for side in ["a", "b"] {
        let path = &create_schema["properties"]["data"]["properties"][side];
        assert_eq!(path["minItems"], 1);
        assert_eq!(path["maxItems"], 1024);
        assert_eq!(path["items"]["additionalProperties"], false);
    }
    let result = server.call(
        "project_execute",
        json!({"input":input,"output":created,"commands":[command]}),
    );
    assert_eq!(success(&result)["revision"], editor.project.revision + 1);
    assert_eq!(success(&result)["commands_applied"], 1);
    let project = Project::load(&created).unwrap();
    assert!(project.seams.is_empty());
    assert_eq!(project.sewing_groups.len(), 1);
    let group = project.sewing_groups[0].clone();
    assert_eq!(group.a, intervals[..2]);
    assert_eq!(group.b, intervals[2..]);
    let saved: Value = serde_json::from_slice(&fs::read(&created).unwrap()).unwrap();
    assert_eq!(saved["version"], weftforge_core::FORMAT_VERSION);

    fs::write(&commands_file, serde_json::to_vec(&vec![command]).unwrap()).unwrap();
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&input)
        .arg(&cli_created)
        .arg(&commands_file)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(fs::read(&created).unwrap(), fs::read(&cli_created).unwrap());
    assert_eq!(fs::read(&input).unwrap(), source);
    let inspect = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("inspect-sewing")
        .arg(&created)
        .output()
        .unwrap();
    assert!(
        inspect.status.success(),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let inspected: Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(inspected["kind"], "sewing_inspection");
    assert_eq!(inspected["source"], saved);
    assert_eq!(inspected["mesh"]["doc_revision"], project.revision);
    assert_eq!(inspected["sewing_groups"][0]["id"], group.id);
    for (actual, expected) in inspected["sewing_groups"][0]["lengths"]
        .as_array()
        .unwrap()
        .iter()
        .zip([0.4, 0.6])
    {
        assert!((actual.as_f64().unwrap() - expected).abs() < 1e-12);
    }
    let stitches = inspected["mesh"]["stitches"].as_array().unwrap();
    assert!(!stitches.is_empty());
    assert!(stitches.iter().all(|stitch| stitch["seam"] == group.id));
    let mut reversed = group.clone();
    reversed.b.reverse();
    for interval in &mut reversed.b {
        std::mem::swap(&mut interval.t0, &mut interval.t1);
    }
    let result = server.call("project_execute", json!({"input":created,"output":updated,"commands":[Command::UpdateSewingGroup{group:reversed.clone()}]}));
    assert_eq!(success(&result)["revision"], project.revision + 1);
    assert_eq!(
        Project::load(&updated).unwrap().sewing_groups,
        vec![reversed]
    );
    let result = server.call("project_execute", json!({"input":updated,"output":removed,"commands":[Command::RemoveSewingGroup{group:group.id}]}));
    assert_eq!(success(&result)["revision"], project.revision + 2);
    assert!(Project::load(&removed).unwrap().sewing_groups.is_empty());
    assert_eq!(Project::load(&removed).unwrap().pieces, project.pieces);

    let old_output = fs::read(&created).unwrap();
    let mut invalid = group;
    invalid.b[0] = invalid.a[0];
    tool_error(&server.call("project_execute", json!({"input":created,"output":created,"commands":[Command::UpdateSewingGroup{group:invalid}]})));
    assert_eq!(fs::read(&created).unwrap(), old_output);
    fs::write(
        &commands_file,
        serde_json::to_vec(&vec![
            Command::SetResolution { metres: 0.1 },
            Command::SetResolution { metres: -1.0 },
        ])
        .unwrap(),
    )
    .unwrap();
    let failed = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&created)
        .arg(&created)
        .arg(&commands_file)
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert_eq!(fs::read(&created).unwrap(), old_output);
    assert!(!recovery_path(&created).exists());
    server.finish();
}

#[test]
fn cli_command_files_are_bounded_strict_and_do_not_replace_failed_destinations() {
    let directory = Directory::new();
    let input = directory.path("source.weftforge");
    let output = directory.path("destination.weftforge");
    let commands = directory.path("commands.json");
    Project::default().save(&input).unwrap();
    let original = fs::read(&input).unwrap();
    fs::write(&output, &original).unwrap();
    let cases = [
        b"[]".to_vec(),
        br#"[{"type":"transaction","data":[]}]"#.to_vec(),
        br#"[{"type":"set_resolution","data":{"metres":0.1,"metres":0.2}}]"#.to_vec(),
        vec![b' '; 1024 * 1024 + 1],
        serde_json::to_vec(&vec![Command::SetResolution { metres: 0.1 }; 1025]).unwrap(),
    ];
    for bytes in cases {
        fs::write(&commands, bytes).unwrap();
        let result = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("execute")
            .arg(&input)
            .arg(&output)
            .arg(&commands)
            .output()
            .unwrap();
        assert!(!result.status.success(), "Invalid command file accepted");
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&output).unwrap(), original);
        assert!(!recovery_path(&output).exists());
    }
}

fn physical_sewing_input() -> (Editor, u64, u64, Command) {
    use weftforge_core::{SewingDirection, SewingSideByLength};
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Original physical-length curved panel".into(),
            vertices: vec![[0., 0.], [0.2, 0.1], [0.2, 0.3], [0., 0.3]],
        })
        .unwrap();
    editor
        .execute(Command::AddRectangle {
            name: "Original physical-length straight panel".into(),
            origin: [0.4, 0.],
            width: 0.2,
            height: 0.3,
        })
        .unwrap();
    let a = editor.project.pieces[0].boundary[0];
    let b = editor.project.pieces[1].boundary[0];
    let command = Command::AddSewingByLength {
        name: "Original physical 100 mm pair".into(),
        a: SewingSideByLength {
            curve: a,
            start_t: 0.,
            direction: SewingDirection::Forward,
            length_m: 0.1,
        },
        b: SewingSideByLength {
            curve: b,
            start_t: 0.75,
            direction: SewingDirection::Reverse,
            length_m: 0.1,
        },
    };
    (editor, a, b, command)
}

#[test]
fn physical_length_sewing_schema_and_cli_mcp_use_the_shared_candidate_transaction() {
    let directory = Directory::new();
    let input = directory.path("original.weftforge");
    let output = directory.path("mcp.weftforge");
    let cli_output = directory.path("cli.weftforge");
    let command_file = directory.path("commands.json");
    let (mut editor, a, _, command) = physical_sewing_input();
    editor.project.save(&input).unwrap();
    let source = fs::read(&input).unwrap();
    editor.project.save(&output).unwrap();
    editor.project.save(&cli_output).unwrap();
    let commands = vec![
        Command::SetCurveControls {
            curve: a,
            controls: Some([[0.2 / 3., 0.], [0.4 / 3., 0.1 / 3.]]),
        },
        command,
    ];
    let revision = editor.project.revision;
    editor
        .execute(Command::Transaction(commands.clone()))
        .unwrap();
    assert_eq!(editor.project.revision, revision + 1);
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9);
    let variants = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap();
    assert_eq!(variants.len(), 32);
    let variant = variants
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "add_sewing_by_length")
        .unwrap();
    assert_eq!(variant["additionalProperties"], false);
    let data = &variant["properties"]["data"];
    assert_eq!(data["additionalProperties"], false);
    assert_eq!(data["required"], json!(["name", "a", "b"]));
    for side in ["a", "b"] {
        let schema = &data["properties"][side];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(
            schema["required"],
            json!(["curve", "start_t", "direction", "length_m"])
        );
        assert_eq!(schema["properties"]["start_t"]["minimum"], 0);
        assert_eq!(schema["properties"]["start_t"]["maximum"], 1);
        assert_eq!(
            schema["properties"]["direction"]["enum"],
            json!(["forward", "reverse"])
        );
        assert_eq!(schema["properties"]["length_m"]["exclusiveMinimum"], 0);
    }
    let response = server.call(
        "project_execute",
        json!({"input":input,"output":output,"commands":commands}),
    );
    assert_eq!(success(&response)["commands_applied"], 2);
    assert_eq!(success(&response)["revision"], revision + 1);
    assert_eq!(Project::load(&output).unwrap(), editor.project);
    assert_eq!(
        fs::read(&output).unwrap(),
        editor.project.to_json().unwrap()
    );
    fs::write(&command_file, serde_json::to_vec(&commands).unwrap()).unwrap();
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&input)
        .arg(&cli_output)
        .arg(&command_file)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(fs::read(&output).unwrap(), fs::read(&cli_output).unwrap());
    assert_eq!(fs::read(recovery_path(&output)).unwrap(), source);
    assert_eq!(fs::read(recovery_path(&cli_output)).unwrap(), source);
    assert_eq!(fs::read(&input).unwrap(), source);
    let group = &editor.project.sewing_groups[0];
    assert!((group.a[0].t1 - 0.4819445564563284).abs() < 1e-8);
    assert!((group.b[0].t1 - 0.25).abs() < 1e-14);
    let inspected = server.call("project_inspect", json!({"path":output}));
    assert_eq!(
        success(&inspected)["project"],
        native_project_json(&editor.project)
    );
    let saved: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(saved["version"], weftforge_core::FORMAT_VERSION);
    for side in ["a", "b"] {
        assert_eq!(
            saved["project"]["sewing_groups"][0][side][0]
                .as_object()
                .unwrap()
                .len(),
            3
        );
    }
    let report = server.call(
        "project_lengths",
        json!({"path":output,"query":{"type":"sewing_group","data":{"id":group.id}}}),
    );
    for length in success(&report)["lengths_m"].as_array().unwrap() {
        assert!((length.as_f64().unwrap() - 0.1).abs() < 1e-8);
    }
    server.finish();
}

#[test]
fn physical_length_sewing_rejects_original_strict_payloads_and_preserves_same_file_bytes() {
    let directory = Directory::new();
    let path = directory.path("original.weftforge");
    let command_file = directory.path("commands.json");
    let (editor, _, _, command) = physical_sewing_input();
    editor.project.save(&path).unwrap();
    editor.project.save(&path).unwrap();
    let primary = fs::read(&path).unwrap();
    let recovery = fs::read(recovery_path(&path)).unwrap();
    let valid = json!(command);
    let mut invalid = Vec::new();
    for field in ["name", "a", "b"] {
        let mut missing = valid.clone();
        missing["data"].as_object_mut().unwrap().remove(field);
        invalid.push(missing);
    }
    for side in ["a", "b"] {
        for field in ["curve", "start_t", "direction", "length_m"] {
            let mut missing = valid.clone();
            missing["data"][side].as_object_mut().unwrap().remove(field);
            invalid.push(missing);
        }
        for (field, value) in [
            ("curve", json!(u64::MAX)),
            ("curve", json!(null)),
            ("start_t", json!(-0.1)),
            ("start_t", json!(1.1)),
            ("start_t", json!(null)),
            ("direction", json!("Forward")),
            ("direction", json!(1)),
            ("direction", json!(null)),
            ("length_m", json!(0)),
            ("length_m", json!(-0.1)),
            ("length_m", json!(1e300)),
            ("length_m", json!(1e-12)),
            ("length_m", json!("NaN")),
            ("extra", json!(true)),
        ] {
            let mut bad = valid.clone();
            bad["data"][side][field] = value;
            invalid.push(bad);
        }
    }
    let mut overlapping = valid.clone();
    overlapping["data"]["b"] = overlapping["data"]["a"].clone();
    invalid.push(overlapping);
    let mut invalid_name = valid.clone();
    invalid_name["data"]["name"] = json!("");
    invalid.push(invalid_name);
    let mut extra = valid.clone();
    extra["data"]["extra"] = json!(true);
    invalid.push(extra);
    let mut server = Server::start();
    server.initialize("2025-11-25");
    for bad in invalid {
        let commands = json!([{"type":"set_resolution","data":{"metres":0.09}}, bad]);
        tool_error(&server.call(
            "project_execute",
            json!({"input":path,"output":path,"commands":commands}),
        ));
        fs::write(&command_file, serde_json::to_vec(&commands).unwrap()).unwrap();
        let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("execute")
            .arg(&path)
            .arg(&path)
            .arg(&command_file)
            .output()
            .unwrap();
        assert!(!cli.status.success(), "{commands}");
        assert_eq!(fs::read(&path).unwrap(), primary);
        assert_eq!(fs::read(recovery_path(&path)).unwrap(), recovery);
    }
    let raw = json!({"jsonrpc":"2.0","id":"strict-physical-length","method":"tools/call","params":{"name":"project_execute","arguments":{"input":path,"output":path,"commands":[valid]}}}).to_string();
    for (token, replacement) in [
        ("\"length_m\":0.1", "\"length_m\":1e400"),
        ("\"length_m\":0.1", "\"length_m\":0.1,\"length_m\":0.2"),
        (
            "\"direction\":\"forward\"",
            "\"direction\":\"forward\",\"direction\":\"reverse\"",
        ),
        ("\"start_t\":0.0", "\"start_t\":0.0,\"start_t\":0.1"),
    ] {
        let request = raw.replace(token, replacement);
        assert_ne!(request, raw);
        server.send(&request);
        let result = server.receive();
        assert!(
            result.get("error").is_some() || result["result"]["isError"] == true,
            "{result}"
        );
        fs::write(
            &command_file,
            format!("[{}]", valid.to_string().replace(token, replacement)),
        )
        .unwrap();
        let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("execute")
            .arg(&path)
            .arg(&path)
            .arg(&command_file)
            .output()
            .unwrap();
        assert!(!cli.status.success());
        assert_eq!(fs::read(&path).unwrap(), primary);
        assert_eq!(fs::read(recovery_path(&path)).unwrap(), recovery);
    }
    let folder = directory.path("folder.weftforge");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("retained"), b"folder contents").unwrap();
    fs::write(recovery_path(&folder), &recovery).unwrap();
    tool_error(&server.call(
        "project_execute",
        json!({"input":path,"output":folder,"commands":[valid]}),
    ));
    assert_eq!(
        fs::read(folder.join("retained")).unwrap(),
        b"folder contents"
    );
    assert_eq!(fs::read(recovery_path(&folder)).unwrap(), recovery);
    assert_eq!(fs::read(&path).unwrap(), primary);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), recovery);
    success(&server.call(
        "project_execute",
        json!({"input":path,"output":path,"commands":[valid]}),
    ));
    assert_eq!(Project::load(&path).unwrap().sewing_groups.len(), 1);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), primary);
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
fn stdio_ellipse_discovery_creation_inspection_and_failure_preservation() {
    let directory = Directory::new();
    let input = directory.path("empty.weftforge");
    let output = directory.path("ellipse.weftforge");
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
        .find(|variant| variant["properties"]["type"]["const"] == "add_ellipse")
        .unwrap();
    assert_eq!(
        schema["properties"]["data"]["required"],
        json!(["name", "origin", "width", "height"])
    );
    assert_eq!(schema["properties"]["data"]["additionalProperties"], false);
    success(&server.call("project_create", json!({"output":input})));
    let original_input = fs::read(&input).unwrap();
    let command = json!({"type":"add_ellipse","data":{"name":"Original MCP ellipse","origin":[-0.4,0.2],"width":0.8,"height":0.6}});
    let response = server.call(
        "project_execute",
        json!({"input":input,"output":output,"commands":[command]}),
    );
    assert_eq!(success(&response)["revision"], 1);
    let saved = Project::load(&output).unwrap();
    assert_eq!(
        saved.piece_bounds(saved.pieces[0].id).unwrap(),
        [[-0.4, 0.2], [0.4, 0.8]]
    );
    assert!(saved.curves.iter().all(|c| c.controls.is_some()));
    let inspected = server.call("project_inspect", json!({"path":output}));
    assert_eq!(success(&inspected)["project"], native_project_json(&saved));
    success(&server.call(
        "project_execute",
        json!({"input":output,"output":output,"commands":[command]}),
    ));
    let previous_output = fs::read(&output).unwrap();
    let previous_recovery = fs::read(recovery_path(&output)).unwrap();
    for data in [
        json!({"name":"x","origin":[0,0],"width":0,"height":0.4}),
        json!({"name":"x","origin":[0,0],"width":-1,"height":0.4}),
        json!({"name":"x","origin":[999.9,0],"width":0.3,"height":0.4}),
        json!({"name":"x","origin":[0,0],"width":0.3,"height":0.4,"extra":1}),
        json!({"name":"x","origin":[0,0],"width":"NaN","height":0.4}),
        json!({"name":"x","origin":[0,0],"width":0.3}),
    ] {
        let result = server.call(
            "project_execute",
            json!({"input":input,"output":output,"commands":[{"type":"add_ellipse","data":data}]}),
        );
        assert!(
            result.get("error").is_some() || result["result"]["isError"] == true,
            "{result}"
        );
        assert_eq!(fs::read(&input).unwrap(), original_input);
        assert_eq!(fs::read(&output).unwrap(), previous_output);
        assert_eq!(fs::read(recovery_path(&output)).unwrap(), previous_recovery);
    }
    let malformed = directory.path("invalid.weftforge");
    fs::write(
        &malformed,
        r#"{"format":"weftforge","version":99,"project":{}}"#,
    )
    .unwrap();
    tool_error(&server.call(
        "project_execute",
        json!({"input":malformed,"output":output,"commands":[command]}),
    ));
    assert_eq!(fs::read(&output).unwrap(), previous_output);
    assert_eq!(fs::read(recovery_path(&output)).unwrap(), previous_recovery);
    server.finish();
}

#[test]
fn stdio_transform_schema_cli_equivalence_and_strict_atomic_failures() {
    use weftforge_core::Placement;
    let directory = Directory::new();
    let input = directory.path("transform-input.weftforge");
    let output = directory.path("transform-output.weftforge");
    let cli_output = directory.path("transform-cli.weftforge");
    let commands_file = directory.path("transform-commands.json");
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Asymmetric transform".into(),
            vertices: vec![[0., 0.], [0.75, 0.75], [-0.25, 1.25], [-0.5, 0.25]],
        })
        .unwrap();
    let piece = editor.project.pieces[0].id;
    editor
        .execute(Command::Transaction(vec![
            Command::SetCurveControls {
                curve: editor.project.pieces[0].boundary[0],
                controls: Some([[0.25, 0.], [0.5, 0.25]]),
            },
            Command::SetGrain { piece, angle: 0.37 },
            Command::SetPlacement {
                piece,
                placement: Placement {
                    translation: [0.2, 0.4, -0.3],
                    rotation: [0.31, -0.47, 0.21],
                },
            },
        ]))
        .unwrap();
    editor.project.save(&input).unwrap();
    Project::default().save(&output).unwrap();
    editor.project.save(&output).unwrap();
    let original_input = fs::read(&input).unwrap();
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9);
    let variant = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "transform_piece")
        .unwrap();
    assert_eq!(variant["additionalProperties"], false);
    let data = &variant["properties"]["data"];
    assert_eq!(
        data["required"],
        json!(["piece", "translation", "angle", "pivot"])
    );
    assert_eq!(data["additionalProperties"], false);
    let properties = &data["properties"];
    assert_eq!(properties["piece"]["minimum"], 1);
    assert_eq!(properties["angle"]["minimum"], -1e6);
    assert_eq!(properties["angle"]["maximum"], 1e6);
    for field in ["translation", "pivot"] {
        assert_eq!(properties[field]["minItems"], 2);
        assert_eq!(properties[field]["maxItems"], 2);
        assert_eq!(properties[field]["items"]["minimum"], -1000);
        assert_eq!(properties[field]["items"]["maximum"], 1000);
    }
    let command = Command::TransformPiece {
        piece,
        translation: [0.5, -0.25],
        angle: std::f64::consts::FRAC_PI_2,
        pivot: [0.25, 0.5],
    };
    let response = server.call(
        "project_execute",
        json!({"input":input,"output":output,"commands":[command]}),
    );
    assert_eq!(success(&response)["revision"], editor.project.revision + 1);
    assert_eq!(success(&response)["commands_applied"], 1);
    let saved = Project::load(&output).unwrap();
    for (old, new) in editor.project.points.iter().zip(&saved.points) {
        for (actual, expected) in new
            .position
            .into_iter()
            .zip([1.25 - old.position[1], old.position[0]])
        {
            assert!((actual - expected).abs() <= 1e-10);
        }
        let old_world = editor.project.pieces[0].placement.transform(old.position);
        let new_world = saved.pieces[0].placement.transform(new.position);
        assert!(
            old_world
                .into_iter()
                .zip(new_world)
                .all(|(a, b)| (a - b).abs() <= 1e-10)
        );
    }
    let inspected = server.call("project_inspect", json!({"path":output}));
    assert_eq!(success(&inspected)["project"], native_project_json(&saved));
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&output).unwrap()).unwrap()["version"],
        weftforge_core::FORMAT_VERSION
    );
    fs::write(
        &commands_file,
        serde_json::to_vec(&vec![command.clone()]).unwrap(),
    )
    .unwrap();
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&input)
        .arg(&cli_output)
        .arg(&commands_file)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&cli.stdout).unwrap()["revision"],
        saved.revision
    );
    assert_eq!(fs::read(&cli_output).unwrap(), fs::read(&output).unwrap());
    let previous_output = fs::read(&output).unwrap();
    let previous_recovery = fs::read(recovery_path(&output)).unwrap();
    let valid = json!(command);
    let mut failures = vec![
        json!([valid.clone(), {"type":"transform_piece","data":{"piece":0,"translation":[0,0],"angle":0,"pivot":[0,0]}}]),
    ];
    for field in ["piece", "translation", "angle", "pivot"] {
        let mut missing = valid.clone();
        missing["data"].as_object_mut().unwrap().remove(field);
        failures.push(json!([missing]));
    }
    for (field, value) in [
        ("piece", json!(0)),
        ("piece", json!(9999)),
        ("translation", json!([0])),
        ("translation", json!([0, 0, 0])),
        ("pivot", json!([0])),
        ("pivot", json!([0, 0, 0])),
        ("translation", json!([1000.1, 0])),
        ("translation", json!([1000, 0])),
        ("pivot", json!([0, -1000.1])),
        ("pivot", json!([1000, -1000])),
        ("angle", json!(1e6 + 1.)),
        ("angle", json!("NaN")),
        ("extra", json!(true)),
    ] {
        let mut invalid = valid.clone();
        invalid["data"][field] = value;
        failures.push(json!([invalid]));
    }
    for commands in failures {
        let result = server.call(
            "project_execute",
            json!({"input":input,"output":output,"commands":commands}),
        );
        assert!(
            result.get("error").is_some() || result["result"]["isError"] == true,
            "{result}"
        );
        fs::write(&commands_file, serde_json::to_vec(&commands).unwrap()).unwrap();
        let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("execute")
            .arg(&input)
            .arg(&output)
            .arg(&commands_file)
            .output()
            .unwrap();
        assert!(!cli.status.success());
        assert_eq!(fs::read(&input).unwrap(), original_input);
        assert_eq!(fs::read(&output).unwrap(), previous_output);
        assert_eq!(fs::read(recovery_path(&output)).unwrap(), previous_recovery);
    }
    let raw = json!({"jsonrpc":"2.0","id":"strict-transform","method":"tools/call","params":{"name":"project_execute","arguments":{"input":input,"output":output,"commands":[valid]}}}).to_string();
    let angle_token = format!("\"angle\":{}", std::f64::consts::FRAC_PI_2);
    for replacement in [
        "\"angle\":1e400",
        "\"angle\":0,\"angle\":1",
        "\"angle\":0,\"piece\":9999",
    ] {
        let request = raw.replace(&angle_token, replacement);
        assert_ne!(request, raw);
        server.send(&request);
        let result = server.receive();
        assert!(
            result.get("error").is_some() || result["result"]["isError"] == true,
            "{result}"
        );
        let commands = format!(
            "[{}]",
            serde_json::to_string(&command)
                .unwrap()
                .replace(&angle_token, replacement)
        );
        fs::write(&commands_file, commands).unwrap();
        let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("execute")
            .arg(&input)
            .arg(&output)
            .arg(&commands_file)
            .output()
            .unwrap();
        assert!(!cli.status.success());
        assert_eq!(fs::read(&input).unwrap(), original_input);
        assert_eq!(fs::read(&output).unwrap(), previous_output);
        assert_eq!(fs::read(recovery_path(&output)).unwrap(), previous_recovery);
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
        native_project_json(&editor.project)
    );
    server.finish();
}

fn modern_metadata(version: &str) -> Value {
    json!({"io.modelcontextprotocol/protocolVersion":version,"io.modelcontextprotocol/clientCapabilities":{}})
}

#[test]
fn authored_lengths_match_cli_core_and_both_mcp_profiles_without_writes() {
    use weftforge_core::CurveInterval;
    let directory = Directory::new();
    let path = directory.path("original.weftforge");
    let unrelated = directory.path("unrelated.weftforge");
    let invalid_project = directory.path("invalid.weftforge");
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Original analytic parabola".into(),
            vertices: vec![[0., 0.], [1., 1.], [0., 1.]],
        })
        .unwrap();
    let a = editor.project.pieces[0].boundary[0];
    let extra = editor.project.pieces[0].boundary[2];
    editor
        .execute(Command::SetCurveControls {
            curve: a,
            controls: Some([[1. / 3., 0.], [2. / 3., 1. / 3.]]),
        })
        .unwrap();
    editor
        .execute(Command::AddRectangle {
            name: "Original paired panel".into(),
            origin: [2., 0.],
            width: 1.7,
            height: 0.5,
        })
        .unwrap();
    let b = editor.project.pieces[1].boundary[0];
    let side_a = CurveInterval {
        curve: a,
        t0: 0.8,
        t1: 0.2,
    };
    let side_b = CurveInterval {
        curve: b,
        t0: 0.1,
        t1: 0.9,
    };
    editor
        .execute(Command::AddSeam {
            name: "Original legacy pair".into(),
            a: side_a,
            b: side_b,
            correspondence: vec![[0., 0.], [0.3, 0.7], [1., 1.]],
        })
        .unwrap();
    editor
        .execute(Command::AddSewingGroup {
            name: "Original disconnected pair".into(),
            a: vec![side_a, CurveInterval::full(extra)],
            b: vec![side_b],
            correspondence: vec![[0., 0.], [0.4, 0.8], [1., 1.]],
        })
        .unwrap();
    let project = &editor.project;
    Project::default().save(&path).unwrap();
    project.save(&path).unwrap();
    fs::write(&unrelated, b"retained unrelated destination").unwrap();
    fs::write(&invalid_project, b"{invalid native input").unwrap();
    let retained = fs::read(&path).unwrap();
    let recovery = fs::read(recovery_path(&path)).unwrap();
    let primitive = |t: f64| t * (1. + 4. * t * t).sqrt() / 2. + (2. * t).asinh() / 4.;
    let expected_a = primitive(0.8) - primitive(0.2);
    let requests = [
        (
            json!({"type":"interval","data":{"curve":a,"t0":0.8,"t1":0.2}}),
            vec![
                "interval".to_string(),
                a.to_string(),
                "0.8".into(),
                "0.2".into(),
            ],
            vec![project.interval_length(side_a).unwrap()],
            vec![expected_a],
        ),
        (
            json!({"type":"seam","data":{"id":project.seams[0].id}}),
            vec!["seam".to_string(), project.seams[0].id.to_string()],
            project.seam_lengths(project.seams[0].id).unwrap().to_vec(),
            vec![expected_a, 1.36],
        ),
        (
            json!({"type":"sewing_group","data":{"id":project.sewing_groups[0].id}}),
            vec![
                "sewing-group".to_string(),
                project.sewing_groups[0].id.to_string(),
            ],
            project
                .sewing_group_geometry(project.sewing_groups[0].id)
                .unwrap()
                .lengths
                .to_vec(),
            vec![expected_a + 1., 1.36],
        ),
    ];
    for modern in [false, true] {
        let mut server = Server::start();
        if !modern {
            server.initialize("2025-11-25");
        }
        let metadata = modern_metadata("2026-07-28");
        let profile_params = |mut params: Value| {
            if modern {
                params["_meta"] = metadata.clone();
            }
            params
        };
        let listed = server.request("tools/list", profile_params(json!({})));
        let tool = listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|tool| tool["name"] == "project_lengths")
            .unwrap();
        assert_eq!(
            tool["annotations"],
            json!({"readOnlyHint":true,"destructiveHint":false,"openWorldHint":false})
        );
        assert_eq!(tool["inputSchema"]["required"], json!(["path", "query"]));
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        let variants = tool["inputSchema"]["properties"]["query"]["oneOf"]
            .as_array()
            .unwrap();
        assert_eq!(variants.len(), 3);
        for variant in variants {
            assert_eq!(variant["additionalProperties"], false);
            assert_eq!(variant["properties"]["data"]["additionalProperties"], false);
            let properties = &variant["properties"]["data"]["properties"];
            if variant["properties"]["type"]["const"] == "interval" {
                assert_eq!(properties["curve"]["minimum"], 1);
                assert_eq!(
                    properties["t0"],
                    json!({"type":"number","minimum":0,"maximum":1})
                );
                assert_eq!(properties["t1"], properties["t0"]);
            } else {
                assert_eq!(properties["id"]["minimum"], 1);
            }
        }
        for (query, args, core, expected) in &requests {
            let output = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
                .arg("lengths")
                .arg(&path)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty());
            let cli: Value = serde_json::from_slice(&output.stdout).unwrap();
            let response = server.request(
                "tools/call",
                profile_params(
                    json!({"name":"project_lengths","arguments":{"path":path,"query":query}}),
                ),
            );
            assert_eq!(success(&response), &cli);
            assert_eq!(cli["source_revision"], project.revision);
            assert_eq!(cli["length_unit"], "m");
            assert_eq!(cli["query"], *query);
            let actual = if core.len() == 1 {
                vec![cli["length_m"].as_f64().unwrap()]
            } else {
                cli["lengths_m"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect()
            };
            assert_eq!(&actual, core);
            for (a, e) in actual.iter().zip(expected) {
                assert!((a - e).abs() <= 1e-8, "{a} != {e}");
            }
            if actual.len() == 2 {
                assert_eq!(
                    cli["mismatch_b_minus_a_m"].as_f64().unwrap(),
                    actual[1] - actual[0]
                );
                assert!(((actual[1] - actual[0]) - (expected[1] - expected[0])).abs() <= 2e-8);
            }
        }
        let invalid_queries = [
            json!({"type":"interval","data":{"curve":u64::MAX,"t0":0.,"t1":1.}}),
            json!({"type":"seam","data":{"id":u64::MAX}}),
            json!({"type":"sewing_group","data":{"id":u64::MAX}}),
            json!({"type":"interval","data":{"curve":a,"t0":0.2,"t1":0.2}}),
            json!({"type":"interval","data":{"curve":a,"t0":0.,"t1":1e-8}}),
            json!({"type":"interval","data":{"curve":a,"t0":-0.1,"t1":1.}}),
            json!({"type":"interval","data":{"curve":a,"t0":0.,"t1":1.1}}),
            json!({"type":"interval","data":{"curve":a,"t0":0.,"t1":null}}),
            json!({"type":"interval","data":{"curve":a,"t0":0.}}),
            json!({"type":"seam","data":{"id":project.seams[0].id,"unknown":true}}),
            json!({"type":"seam","data":{"id":project.seams[0].id},"unknown":true}),
            json!({"type":"seam","data":null}),
            json!({"type":"unknown","data":{}}),
            Value::Null,
        ];
        for query in invalid_queries {
            tool_error(&server.request(
                "tools/call",
                profile_params(
                    json!({"name":"project_lengths","arguments":{"path":path,"query":query}}),
                ),
            ));
        }
        for replacement in ["\"t0\":0.8,\"t0\":0.6", "\"t0\":null", "\"t0\":NaN"] {
            let raw=json!({"jsonrpc":"2.0","id":"raw-query","method":"tools/call","params":profile_params(json!({"name":"project_lengths","arguments":{"path":path,"query":requests[0].0}}))}).to_string().replace("\"t0\":0.8",replacement);
            server.send(&raw);
            let response = server.receive();
            if replacement.contains("NaN") {
                assert_eq!(response["error"]["code"], -32700);
            } else {
                tool_error(&response);
            }
        }
        let raw=json!({"jsonrpc":"2.0","id":"duplicate-tag","method":"tools/call","params":profile_params(json!({"name":"project_lengths","arguments":{"path":path,"query":requests[0].0}}))}).to_string().replace("\"type\":\"interval\"","\"type\":\"interval\",\"type\":\"seam\"");
        server.send(&raw);
        tool_error(&server.receive());
        tool_error(&server.request("tools/call",profile_params(json!({"name":"project_lengths","arguments":{"path":invalid_project,"query":requests[0].0}}))));
        success(&server.request(
            "tools/call",
            profile_params(
                json!({"name":"project_lengths","arguments":{"path":path,"query":requests[0].0}}),
            ),
        ));
        server.finish();
    }
    for args in [
        vec!["interval".into(), a.to_string(), "NaN".into(), "1".into()],
        vec!["interval".into(), a.to_string(), "0".into(), "inf".into()],
        vec!["seam".into()],
        vec![
            "seam".into(),
            project.seams[0].id.to_string(),
            "extra".into(),
        ],
        vec!["unknown".into()],
    ] {
        let output = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("lengths")
            .arg(&path)
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    assert_eq!(fs::read(&path).unwrap(), retained);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), recovery);
    assert_eq!(
        fs::read(&unrelated).unwrap(),
        b"retained unrelated destination"
    );
    assert_eq!(
        fs::read(&invalid_project).unwrap(),
        b"{invalid native input"
    );
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
    assert_eq!(listed["result"]["tools"].as_array().unwrap().len(), 9);
    assert_eq!(listed["result"]["cacheScope"], "public");
    assert_eq!(listed["result"]["ttlMs"], 0);
    success(&server.request("tools/call", json!({"_meta":modern_metadata("2026-07-28"),"name":"project_create","arguments":{"output":path}})));
    let modern = server.request("tools/call", json!({"_meta":modern_metadata("2026-07-28"),"name":"project_inspect","arguments":{"path":path}}));
    assert_eq!(
        success(&modern)["project"],
        native_project_json(&Project::default())
    );
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

#[test]
fn svg_import_stdio_discovers_shared_command_reports_physical_scale_and_is_atomic() {
    let directory = Directory::new();
    let input = directory.path("input.weftforge");
    let output = directory.path("output.weftforge");
    let svg = directory.path("original.svg");
    let original = include_bytes!(
        "../../../crates/weftforge-core/tests/fixtures/original-editable-patterns.svg"
    );
    fs::write(&svg, original).unwrap();
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "project_import_svg")
        .unwrap();
    assert_eq!(
        tool["inputSchema"]["required"],
        json!(["input", "output", "svg"])
    );
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    let command = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "import_svg_patterns")
        .unwrap();
    assert_eq!(command["properties"]["data"]["required"], json!(["svg"]));
    assert_eq!(command["properties"]["data"]["additionalProperties"], false);
    assert_eq!(
        command["properties"]["data"]["properties"]["svg"]["maxLength"],
        weftforge_core::MAX_SVG_BYTES
    );
    success(&server.call("project_create", json!({"output":input})));
    let result = server.call(
        "project_import_svg",
        json!({"input":input,"output":output,"svg":svg}),
    );
    let receipt = success(&result);
    assert_eq!(receipt["revision"], 1);
    assert_eq!(receipt["pieces_imported"], 2);
    assert_eq!(receipt["metres_per_user_unit"], 0.002);
    assert!(
        receipt["scope"]
            .as_str()
            .unwrap()
            .contains("exterior boundaries")
    );
    let saved = Project::load(&output).unwrap();
    let mut expected = Editor::default();
    expected
        .execute(Command::ImportSvgPatterns {
            svg: String::from_utf8(original.to_vec()).unwrap(),
        })
        .unwrap();
    assert_eq!(saved, expected.project);
    let before = fs::read(&output).unwrap();
    let input_before = fs::read(&input).unwrap();
    for arguments in [
        json!({"input":input,"output":output,"svg":svg,"extra":true}),
        json!({"input":input,"output":output}),
        json!({"input":input,"output":output,"svg":null}),
        json!({"input":input,"output":svg,"svg":svg}),
    ] {
        tool_error(&server.call("project_import_svg", arguments));
        assert_eq!(fs::read(&output).unwrap(), before);
        assert_eq!(fs::read(&input).unwrap(), input_before);
        assert_eq!(fs::read(&svg).unwrap(), original);
    }
    fs::write(&svg, b"<svg>").unwrap();
    tool_error(&server.call(
        "project_import_svg",
        json!({"input":input,"output":output,"svg":svg}),
    ));
    assert_eq!(fs::read(&output).unwrap(), before);
    assert!(!recovery_path(&output).exists());
    tool_error(&server.call("project_execute", json!({"input":input,"output":output,"commands":[{"type":"import_svg_patterns","data":{"svg":String::from_utf8(original.to_vec()).unwrap()}},{"type":"move_point","data":{"point":0,"position":[0,0]}}]})));
    assert_eq!(fs::read(&output).unwrap(), before);
    success(&server.call("project_execute", json!({"input":input,"output":output,"commands":[{"type":"import_svg_patterns","data":{"svg":String::from_utf8(original.to_vec()).unwrap()}}]})));
    assert_eq!(Project::load(&output).unwrap(), saved);
    assert_eq!(fs::read(recovery_path(&output)).unwrap(), before);
    server.finish();
}

#[test]
fn dxf_import_stdio_discovers_shared_command_reports_physical_scale_and_is_atomic() {
    let directory = Directory::new();
    let input = directory.path("input.weftforge");
    let output = directory.path("output.weftforge");
    let dxf = directory.path("original.dxf");
    let original = include_bytes!(
        "../../../crates/weftforge-core/tests/fixtures/original-editable-patterns.dxf"
    );
    fs::write(&dxf, original).unwrap();
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "project_import_dxf")
        .unwrap();
    assert_eq!(
        tool["inputSchema"]["required"],
        json!(["input", "output", "dxf"])
    );
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    let command = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variant| variant["properties"]["type"]["const"] == "import_dxf_patterns")
        .unwrap();
    assert_eq!(command["properties"]["data"]["required"], json!(["dxf"]));
    assert_eq!(command["properties"]["data"]["additionalProperties"], false);
    assert_eq!(
        command["properties"]["data"]["properties"]["dxf"]["maxLength"],
        weftforge_core::MAX_DXF_BYTES
    );
    success(&server.call("project_create", json!({"output":input})));
    let result = server.call(
        "project_import_dxf",
        json!({"input":input,"output":output,"dxf":dxf}),
    );
    let receipt = success(&result);
    assert_eq!(receipt["revision"], 1);
    assert_eq!(receipt["pieces_imported"], 2);
    assert_eq!(receipt["metres_per_user_unit"], 0.001);
    assert!(
        receipt["scope"]
            .as_str()
            .unwrap()
            .contains("exterior boundaries")
    );
    let saved = Project::load(&output).unwrap();
    let mut expected = Editor::default();
    expected
        .execute(Command::ImportDxfPatterns {
            dxf: String::from_utf8(original.to_vec()).unwrap(),
        })
        .unwrap();
    assert_eq!(saved, expected.project);
    let before = fs::read(&output).unwrap();
    let input_before = fs::read(&input).unwrap();
    for arguments in [
        json!({"input":input,"output":output,"dxf":dxf,"extra":true}),
        json!({"input":input,"output":output}),
        json!({"input":input,"output":output,"dxf":null}),
        json!({"input":input,"output":dxf,"dxf":dxf}),
    ] {
        tool_error(&server.call("project_import_dxf", arguments));
        assert_eq!(fs::read(&output).unwrap(), before);
        assert_eq!(fs::read(&input).unwrap(), input_before);
        assert_eq!(fs::read(&dxf).unwrap(), original);
    }
    fs::write(&dxf, b"0\nSECTION").unwrap();
    tool_error(&server.call(
        "project_import_dxf",
        json!({"input":input,"output":output,"dxf":dxf}),
    ));
    assert_eq!(fs::read(&output).unwrap(), before);
    assert!(!recovery_path(&output).exists());
    tool_error(&server.call("project_execute", json!({"input":input,"output":output,"commands":[{"type":"import_dxf_patterns","data":{"dxf":String::from_utf8(original.to_vec()).unwrap()}},{"type":"move_point","data":{"point":0,"position":[0,0]}}]})));
    assert_eq!(fs::read(&output).unwrap(), before);
    success(&server.call("project_execute", json!({"input":input,"output":output,"commands":[{"type":"import_dxf_patterns","data":{"dxf":String::from_utf8(original.to_vec()).unwrap()}}]})));
    assert_eq!(Project::load(&output).unwrap(), saved);
    assert_eq!(fs::read(recovery_path(&output)).unwrap(), before);
    server.finish();
}

#[test]
fn inspection_matches_saved_native_json_without_changing_f32_materials_or_f64_geometry() {
    let directory = Directory::new();
    let path = directory.path("canonical-inspection.weftforge");
    let mut editor = Editor::default();
    editor
        .execute(Command::AddEllipse {
            name: "Original decimal geometry".into(),
            origin: [-0.12345678901234566, 0.2718281828459045],
            width: 0.403210987654321,
            height: 0.6543210987654321,
        })
        .unwrap();
    let piece = editor.project.pieces[0].id;
    editor
        .execute(Command::SetPlacement {
            piece,
            placement: weftforge_core::Placement {
                translation: [-0.0, 0.23456789012345678, -0.3456789012345679],
                rotation: [
                    0.12345678901234566,
                    -0.23456789012345678,
                    0.3456789012345679,
                ],
            },
        })
        .unwrap();
    editor.project.materials[0].color = [0.22, 0.55, 0.76];
    editor.project.materials[0].areal_density = 0.18000000000000002;
    editor.project.save(&path).unwrap();
    let original = fs::read(&path).unwrap();
    let native: Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(
        native["project"]["materials"][0]["color"],
        json!([0.22, 0.55, 0.76])
    );
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let response = server.call("project_inspect", json!({"path":path}));
    // success also checks that the text and structured MCP results agree exactly.
    let inspected = &success(&response)["project"];
    assert_eq!(*inspected, native["project"]);
    let typed: Project = serde_json::from_value(inspected.clone()).unwrap();
    assert_eq!(typed, editor.project);
    assert_eq!(typed.materials, editor.project.materials);
    assert_eq!(
        typed.materials[0].color.map(f32::to_bits),
        editor.project.materials[0].color.map(f32::to_bits)
    );
    for (actual, expected) in typed.points.iter().zip(&editor.project.points) {
        assert_eq!(
            actual.position.map(f64::to_bits),
            expected.position.map(f64::to_bits)
        );
    }
    for (actual, expected) in typed.curves.iter().zip(&editor.project.curves) {
        assert_eq!(
            actual
                .controls
                .map(|controls| controls.map(|point| point.map(f64::to_bits))),
            expected
                .controls
                .map(|controls| controls.map(|point| point.map(f64::to_bits)))
        );
    }
    assert_eq!(
        typed.pieces[0].placement.translation.map(f64::to_bits),
        editor.project.pieces[0]
            .placement
            .translation
            .map(f64::to_bits)
    );
    assert_eq!(
        typed.pieces[0].placement.rotation.map(f64::to_bits),
        editor.project.pieces[0]
            .placement
            .rotation
            .map(f64::to_bits)
    );
    assert_eq!(fs::read(&path).unwrap(), original);
    server.finish();
}

#[test]
fn internal_annotations_share_closed_schema_cli_and_mcp_transactions_without_new_tools() {
    let directory = Directory::new();
    let empty = directory.path("empty.weftforge");
    let cli_path = directory.path("cli.weftforge");
    let mcp_path = directory.path("mcp.weftforge");
    let commands_file = directory.path("commands.json");
    Project::default().save(&empty).unwrap();
    let commands = vec![
        Command::AddRectangle {
            name: "Original R".into(),
            origin: [0., 0.],
            width: 1.,
            height: 1.,
        },
        Command::AddInternalSegment {
            piece: 2,
            start: [0.125, 0.25],
            end: [0.875, 0.75],
        },
    ];
    fs::write(&commands_file, serde_json::to_vec(&commands).unwrap()).unwrap();
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&empty)
        .arg(&cli_path)
        .arg(&commands_file)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&cli.stdout).unwrap()["revision"],
        1
    );
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9);
    let variants = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap()["inputSchema"]["properties"]["commands"]["items"]["oneOf"]
        .as_array()
        .unwrap();
    assert_eq!(variants.len(), 32);
    for (name, required, id_field) in [
        (
            "add_internal_segment",
            json!(["piece", "start", "end"]),
            "piece",
        ),
        ("remove_internal_curve", json!(["curve"]), "curve"),
    ] {
        let variant = variants
            .iter()
            .find(|variant| variant["properties"]["type"]["const"] == name)
            .unwrap();
        assert_eq!(variant["additionalProperties"], false);
        assert_eq!(variant["required"], json!(["type", "data"]));
        let data = &variant["properties"]["data"];
        assert_eq!(data["required"], required);
        assert_eq!(data["additionalProperties"], false);
        assert_eq!(data["properties"][id_field]["minimum"], 1);
        assert_eq!(data["properties"][id_field]["maximum"], u64::MAX);
        if name == "add_internal_segment" {
            for field in ["start", "end"] {
                assert_eq!(data["properties"][field]["minItems"], 2);
                assert_eq!(data["properties"][field]["maxItems"], 2);
                assert_eq!(data["properties"][field]["items"]["type"], "number");
            }
        }
    }
    let executed = server.call(
        "project_execute",
        json!({"input":empty,"output":mcp_path,"commands":commands}),
    );
    assert_eq!(success(&executed)["revision"], 1);
    assert_eq!(fs::read(&cli_path).unwrap(), fs::read(&mcp_path).unwrap());
    let saved = Project::load(&mcp_path).unwrap();
    assert_eq!(saved.pieces[0].boundary, [7, 8, 9, 10]);
    assert_eq!(saved.pieces[0].internal_curves, [13]);
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&mcp_path).unwrap()).unwrap()["version"],
        weftforge_core::FORMAT_VERSION
    );
    let inspected = server.call("project_inspect", json!({"path":mcp_path}));
    assert_eq!(success(&inspected)["project"], native_project_json(&saved));
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("inspect")
        .arg(&cli_path)
        .output()
        .unwrap();
    assert!(cli.status.success());
    let inspected: Value = serde_json::from_slice(&cli.stdout).unwrap();
    assert_eq!(inspected["patterns"][0]["boundary"], json!([7, 8, 9, 10]));
    assert_eq!(inspected["patterns"][0]["internal_curves"], json!([13]));
    let length = server.call(
        "project_lengths",
        json!({"path":mcp_path,"query":{"type":"interval","data":{"curve":13,"t0":0.,"t1":1.}}}),
    );
    assert!((success(&length)["length_m"].as_f64().unwrap() - 13_f64.sqrt() / 4.).abs() < 1e-12);
    let edits = vec![
        Command::MovePoint {
            point: 11,
            position: [0.875, 0.75],
        },
        Command::MovePoint {
            point: 12,
            position: [0.125, 0.25],
        },
    ];
    let edited = server.call(
        "project_execute",
        json!({"input":mcp_path,"output":mcp_path,"commands":edits}),
    );
    assert_eq!(success(&edited)["revision"], 2);
    fs::write(&commands_file, serde_json::to_vec(&edits).unwrap()).unwrap();
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&cli_path)
        .arg(&cli_path)
        .arg(&commands_file)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(fs::read(&cli_path).unwrap(), fs::read(&mcp_path).unwrap());
    let primary = fs::read(&mcp_path).unwrap();
    let recovery = fs::read(recovery_path(&mcp_path)).unwrap();
    let failures = [
        json!([{"type":"add_internal_segment","data":{"piece":2,"start":[0.2,0.2],"end":[0.7,0.7]}},{"type":"move_point","data":{"point":12,"position":[2.,2.]}}]),
        json!([{"type":"add_internal_segment","data":{"piece":2,"start":[0.2],"end":[0.7,0.7]}}]),
        json!([{"type":"add_internal_segment","data":{"piece":2,"start":[0.2,0.2],"end":[0.7,0.7],"unknown":true}}]),
        json!([{"type":"remove_internal_curve","data":{"curve":7}}]),
        json!([{"type":"remove_internal_curve","data":{"curve":0}}]),
    ];
    for commands in failures {
        let response = server.call(
            "project_execute",
            json!({"input":mcp_path,"output":mcp_path,"commands":commands}),
        );
        tool_error(&response);
        assert_eq!(fs::read(&mcp_path).unwrap(), primary);
        assert_eq!(fs::read(recovery_path(&mcp_path)).unwrap(), recovery);
        fs::write(&commands_file, serde_json::to_vec(&commands).unwrap()).unwrap();
        let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
            .arg("execute")
            .arg(&mcp_path)
            .arg(&mcp_path)
            .arg(&commands_file)
            .output()
            .unwrap();
        assert!(!cli.status.success());
        assert_eq!(fs::read(&mcp_path).unwrap(), primary);
        assert_eq!(fs::read(recovery_path(&mcp_path)).unwrap(), recovery);
    }
    let raw = json!({"jsonrpc":"2.0","id":"duplicate-internal","method":"tools/call","params":{"name":"project_execute","arguments":{"input":mcp_path,"output":mcp_path,"commands":[{"type":"remove_internal_curve","data":{"curve":13}}]}}}).to_string().replace("\"curve\":13","\"curve\":13,\"curve\":13");
    server.send(&raw);
    tool_error(&server.receive());
    assert_eq!(fs::read(&mcp_path).unwrap(), primary);
    assert_eq!(fs::read(recovery_path(&mcp_path)).unwrap(), recovery);
    fs::write(
        &commands_file,
        r#"[{"type":"remove_internal_curve","data":{"curve":13,"curve":13}}]"#,
    )
    .unwrap();
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&mcp_path)
        .arg(&mcp_path)
        .arg(&commands_file)
        .output()
        .unwrap();
    assert!(!cli.status.success());
    assert_eq!(fs::read(&mcp_path).unwrap(), primary);
    assert_eq!(fs::read(recovery_path(&mcp_path)).unwrap(), recovery);
    let removed = server.call("project_execute",json!({"input":mcp_path,"output":mcp_path,"commands":[{"type":"remove_internal_curve","data":{"curve":13}}]}));
    assert_eq!(success(&removed)["revision"], 3);
    fs::write(
        &commands_file,
        serde_json::to_vec(&vec![Command::RemoveInternalCurve { curve: 13 }]).unwrap(),
    )
    .unwrap();
    let cli = Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg("execute")
        .arg(&cli_path)
        .arg(&cli_path)
        .arg(&commands_file)
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(fs::read(&cli_path).unwrap(), fs::read(&mcp_path).unwrap());
    let removed = Project::load(&mcp_path).unwrap();
    assert!(removed.pieces[0].internal_curves.is_empty());
    assert_eq!(removed.points.len(), 4);
    server.finish();
}

#[test]
fn straight_holes_keep_closed_discovery_shared_ids_and_atomic_refusal() {
    let directory = Directory::new();
    let path = directory.path("annulus.weftforge");
    let refused = directory.path("retained.weftforge");
    let original =
        include_bytes!("../../../crates/weftforge-core/tests/fixtures/original-cut-r240.weftforge");
    let backup = Project::default().to_json().unwrap();
    for target in [&path, &refused] {
        fs::write(target, original).unwrap();
        fs::write(recovery_path(target), &backup).unwrap();
    }
    let mut server = Server::start();
    server.initialize("2025-11-25");
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9);
    let execute = tools
        .iter()
        .find(|tool| tool["name"] == "project_execute")
        .unwrap();
    let commands = &execute["inputSchema"]["properties"]["commands"];
    assert_eq!(commands["minItems"], 1);
    assert_eq!(commands["maxItems"], 1024);
    let variants = commands["items"]["oneOf"].as_array().unwrap();
    assert_eq!(variants.len(), 32);
    for (name, required, id) in [
        ("add_hole", json!(["piece", "vertices"]), "piece"),
        ("remove_hole", json!(["hole"]), "hole"),
    ] {
        let matching = variants
            .iter()
            .filter(|variant| variant["properties"]["type"]["const"] == name)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        let variant = matching[0];
        assert_eq!(variant["required"], json!(["type", "data"]));
        assert_eq!(variant["additionalProperties"], false);
        let data = &variant["properties"]["data"];
        assert_eq!(data["required"], required);
        assert_eq!(data["additionalProperties"], false);
        assert_eq!(
            data["properties"].as_object().unwrap().len(),
            required.as_array().unwrap().len()
        );
        assert_eq!(data["properties"][id]["type"], "integer");
        assert_eq!(data["properties"][id]["minimum"], 1);
        assert_eq!(data["properties"][id]["maximum"], u64::MAX);
        if name == "add_hole" {
            let vertices = &data["properties"]["vertices"];
            assert_eq!(vertices["minItems"], 3);
            assert_eq!(vertices["maxItems"], 4096);
            assert_eq!(vertices["items"]["minItems"], 2);
            assert_eq!(vertices["items"]["maxItems"], 2);
            assert_eq!(vertices["items"]["items"]["type"], "number");
            assert_eq!(vertices["items"]["items"]["minimum"], -1000);
            assert_eq!(vertices["items"]["items"]["maximum"], 1000);
        }
    }
    for name in [
        "project_export_svg",
        "project_export_obj",
        "project_export_dxf",
        "project_import_svg",
    ] {
        let description = tools.iter().find(|tool| tool["name"] == name).unwrap()["description"]
            .as_str()
            .unwrap();
        assert!(description.contains("holes"), "{description}");
    }
    let add = json!({"type":"add_hole","data":{"piece":2,"vertices":[[0.08,0.12],[0.08,0.24],[0.16,0.24],[0.16,0.12]]}});
    let result = server.call(
        "project_execute",
        json!({"input":path,"output":path,"commands":[add]}),
    );
    assert_eq!(success(&result)["revision"], 2);
    let mut expected = Editor::new(Project::from_json(original).unwrap()).unwrap();
    expected
        .execute(serde_json::from_value(add.clone()).unwrap())
        .unwrap();
    assert_eq!(Project::load(&path).unwrap(), expected.project);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), original);
    let inspection = server.call("project_inspect", json!({"path":path}));
    assert_eq!(
        success(&inspection)["project"],
        native_project_json(&expected.project)
    );
    assert_eq!(
        success(&inspection)["project"]["pieces"][0]["holes"],
        json!([{"id":11,"boundary":[16,17,18,19]}])
    );
    let obj = directory.path("annulus.obj");
    let svg = directory.path("annulus.svg");
    for (tool, output) in [("project_export_obj", &obj), ("project_export_svg", &svg)] {
        let result = server.call(tool, json!({"input":path,"output":output}));
        assert_eq!(success(&result)["source_revision"], 2);
    }
    assert!(fs::read_to_string(&obj).unwrap().contains("g piece_2\n"));
    let svg_text = fs::read_to_string(&svg).unwrap();
    assert_eq!(svg_text.matches("<path ").count(), 1);
    assert!(svg_text.contains("fill-rule=\"evenodd\""));
    let primary = fs::read(&path).unwrap();
    tool_error(&server.call(
        "project_import_svg",
        json!({"input":path,"output":path,"svg":svg}),
    ));
    assert_eq!(fs::read(&path).unwrap(), primary);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), original);
    let dxf = directory.path("retained.dxf");
    fs::write(&dxf, b"DXF sentinel").unwrap();
    fs::write(recovery_path(&dxf), b"DXF recovery sentinel").unwrap();
    let result = server.call("project_export_dxf", json!({"input":path,"output":dxf}));
    tool_error(&result);
    assert!(
        result["result"]["structuredContent"]["error"]
            .as_str()
            .unwrap()
            .contains("DXF v1 export does not support holes")
    );
    assert_eq!(fs::read(&dxf).unwrap(), b"DXF sentinel");
    assert_eq!(
        fs::read(recovery_path(&dxf)).unwrap(),
        b"DXF recovery sentinel"
    );
    let mut failures = Vec::new();
    for (field, value) in [
        ("piece", json!(0)),
        ("piece", json!(-1)),
        ("piece", json!(2.5)),
        ("piece", json!("2")),
        ("piece", Value::Null),
        ("vertices", json!([])),
        ("vertices", json!([[0.08, 0.12], [0.08, 0.24]])),
        ("vertices", json!([[0.08], [0.08, 0.24], [0.16, 0.24]])),
        (
            "vertices",
            json!([[0.08, 0.12, 0.], [0.08, 0.24], [0.16, 0.24]]),
        ),
        (
            "vertices",
            json!([[1000.1, 0.12], [0.08, 0.24], [0.16, 0.24]]),
        ),
        (
            "vertices",
            json!([["0.08", 0.12], [0.08, 0.24], [0.16, 0.24]]),
        ),
        ("vertices", json!(vec![[0.08, 0.12]; 4097])),
        ("unknown", json!(true)),
    ] {
        let mut bad = add.clone();
        bad["data"][field] = value;
        failures.push(json!([bad]).to_string());
    }
    for field in ["piece", "vertices"] {
        let mut bad = add.clone();
        bad["data"].as_object_mut().unwrap().remove(field);
        failures.push(json!([bad]).to_string());
    }
    for raw in [
        r#"[{"type":"remove_hole","data":{}}]"#,
        r#"[{"type":"remove_hole","data":{"hole":11,"hole":11}}]"#,
        r#"[{"type":"remove_hole","data":{"hole":0}}]"#,
        r#"[{"type":"remove_hole","data":{"hole":11,"unknown":true}}]"#,
        r#"[{"type":"remove_hole","data":{"hole":18446744073709551616}}]"#,
    ] {
        failures.push(raw.to_owned());
    }
    let valid = json!([add.clone()]).to_string();
    failures.push(valid.replace("\"piece\":2", "\"piece\":2,\"piece\":2"));
    failures.push(valid.replace("\"piece\":2", "\"piece\":18446744073709551616"));
    failures.push(valid.replace("\"vertices\":", "\"vertices\":[],\"vertices\":"));
    failures.push(valid.replace("0.08", "1e309"));
    failures.push(valid.replace(
        "\"type\":\"add_hole\"",
        "\"type\":\"add_hole\",\"type\":\"add_hole\"",
    ));
    failures.push(json!([add.clone(), {"type":"set_resolution","data":{"metres":0}}]).to_string());
    failures.push(json!([{"type":"transaction","data":[add.clone()]}]).to_string());
    failures.push(json!(vec![add.clone(); 1025]).to_string());
    failures.push("[]".into());
    failures.push(format!("[{}]", " ".repeat(1024 * 1024)));
    for raw in failures {
        let request = json!({"jsonrpc":"2.0","id":"raw-hole","method":"tools/call","params":{"name":"project_execute","arguments":{"input":refused,"output":refused,"commands":"HOLE_RAW_COMMANDS"}}}).to_string().replace("\"HOLE_RAW_COMMANDS\"", &raw);
        server.send(&request);
        let result = server.receive();
        if request.len() > 1024 * 1024 {
            assert_eq!(result["error"]["code"], -32600);
        } else {
            tool_error(&result);
        }
        assert_eq!(fs::read(&refused).unwrap(), original);
        assert_eq!(fs::read(recovery_path(&refused)).unwrap(), backup);
    }
    let removed = server.call(
        "project_execute",
        json!({"input":path,"output":path,"commands":[{"type":"remove_hole","data":{"hole":11}}]}),
    );
    assert_eq!(success(&removed)["revision"], 3);
    expected.execute(Command::RemoveHole { hole: 11 }).unwrap();
    assert_eq!(Project::load(&path).unwrap(), expected.project);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), primary);
    server.finish();
}
