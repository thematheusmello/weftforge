use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use weftforge_core::{Command, Editor, Project};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn request(root: &Path, output: &Path, name: &str) -> Result<Value> {
    let bytes = fs::read(root.join("requests").join(format!("{name}.json")))?;
    let value = serde_json::from_slice(&bytes)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join(format!("{name}.request.json")))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(value)
}

fn load(root: &Path, output: &Path, name: &str) -> Result<Editor> {
    let request = request(root, output, name)?;
    let input = request["input"].as_str().ok_or("Missing input path")?;
    let editor = Editor::new(Project::load(root.join(input))?)?;
    write_json(
        &output.join(format!("{name}.invocation.json")),
        &json!({
            "kind":"public_api_invocation", "process_id":std::process::id(),
            "callsite":concat!(file!(), ":", line!()),
            "calls":["Project::load", "Editor::new"],
            "request":format!("{name}.request.json"), "input":input,
            "actual_revision":editor.project.revision,
        }),
    )?;
    write_json(
        &output.join(format!("{name}.response.json")),
        &json!({"loaded":true,"revision":editor.project.revision,
            "can_undo":editor.can_undo(),"can_redo":editor.can_redo()}),
    )?;
    println!(
        "{}",
        json!({"event":"loaded","checkpoint":name,"revision":editor.project.revision})
    );
    Ok(editor)
}

fn checkpoint(output: &Path, name: &str, editor: &Editor) -> Result<String> {
    let mesh = editor.project.mesh()?;
    mesh.validate()?;
    // Native serialization preserves f32 appearance scalars before composing Value.
    let source: Value = serde_json::from_slice(&serde_json::to_vec(&editor.project)?)?;
    let receipt = json!({
        "kind":"sewing_inspection",
        "source":{"format":"weftforge","version":weftforge_core::FORMAT_VERSION,"project":source},
        "sewing_groups":mesh.sewing_groups.iter().map(|group|json!({
            "id":group.id,"lengths":group.lengths,
            "portions":group.portions.iter().map(|portion|json!({"a":portion.a,"b":portion.b})).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "mesh":{
            "doc_revision":mesh.doc_revision,"positions":mesh.positions,"uv":mesh.uv,
            "triangles":mesh.triangles,"piece_ids":mesh.piece_ids,
            "boundary":mesh.boundary.iter().map(|sample|json!({
                "piece":sample.piece,"curve":sample.curve,"t":sample.t,"vertex":sample.vertex
            })).collect::<Vec<_>>(),
            "stitches":mesh.stitches.iter().map(|stitch|json!({
                "seam":stitch.seam,
                "a":{"vertices":stitch.a.vertices,"weight":stitch.a.weight},
                "b":{"vertices":stitch.b.vertices,"weight":stitch.b.weight}
            })).collect::<Vec<_>>()
        }
    });
    let filename = format!("{name}.receipt.json");
    write_json(&output.join(&filename), &receipt)?;
    println!(
        "{}",
        json!({"event":"checkpoint","checkpoint":name,
        "revision":editor.project.revision,"vertices":mesh.positions.len(),
        "triangles":mesh.triangles.len(),"stitches":mesh.stitches.len()})
    );
    Ok(filename)
}

fn operation(root: &Path, output: &Path, name: &str, editor: &mut Editor) -> Result<Value> {
    let request = request(root, output, name)?;
    let method = request["method"].as_str().ok_or("Missing method")?;
    let before = json!({"revision":editor.project.revision,
        "can_undo":editor.can_undo(),"can_redo":editor.can_redo()});
    let (accepted, error, callsite) = match method {
        "Editor::execute" => {
            let command: Command = serde_json::from_value(request["command"].clone())?;
            let callsite = concat!(file!(), ":", line!());
            match editor.execute(command) {
                Ok(()) => (true, None, callsite),
                Err(error) => (false, Some(error.to_string()), callsite),
            }
        }
        "Editor::undo" => {
            let callsite = concat!(file!(), ":", line!());
            let accepted = editor.undo();
            (
                accepted,
                (!accepted).then(|| "Editor::undo returned false".to_owned()),
                callsite,
            )
        }
        "Editor::redo" => {
            let callsite = concat!(file!(), ":", line!());
            let accepted = editor.redo();
            (
                accepted,
                (!accepted).then(|| "Editor::redo returned false".to_owned()),
                callsite,
            )
        }
        _ => return Err(format!("Unsupported driver method: {method}").into()),
    };
    let after = json!({"revision":editor.project.revision,
        "can_undo":editor.can_undo(),"can_redo":editor.can_redo()});
    let invocation_path = format!("{name}.invocation.json");
    let response_path = format!("{name}.response.json");
    write_json(
        &output.join(&invocation_path),
        &json!({
            "kind":"public_api_invocation","process_id":std::process::id(),
            "method":method,"callsite":callsite,"request":format!("{name}.request.json"),
            "before":before,"after":after,"returned":{"accepted":accepted,"error":error}
        }),
    )?;
    write_json(
        &output.join(&response_path),
        &json!({
            "accepted":accepted,"rejected":!accepted,"error":error,
            "before":before,"after":after
        }),
    )?;
    println!(
        "{}",
        json!({"event":"operation","operation":name,
        "method":method,"accepted":accepted,"error":error,"before":before,"after":after})
    );
    Ok(json!({
        "name":name,"request":format!("{name}.request.json"),
        "response":response_path,"invocation":invocation_path,
        "shared_editor_command":true,"observer_review":false,
        "rejected":!accepted,"error":error
    }))
}

fn run() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() != 2 {
        return Err("Usage: chain-sewing-operations-producer PRODUCER_ROOT NEW_CAPTURE_DIR".into());
    }
    let root = &args[0];
    let output = &args[1];
    fs::create_dir(output)?;
    let mut checkpoints = BTreeMap::new();
    let mut commands = Vec::new();
    let mut editor = load(root, output, "split_loaded")?;
    checkpoints.insert("split_loaded", checkpoint(output, "split_loaded", &editor)?);
    for (name, checkpoint_name) in [
        ("split", "split"),
        ("join", "joined"),
        ("undo_join", "undo_join"),
        ("reject_overlap", "rejected_overlap"),
        ("redo_join", "redo_join"),
        ("remesh", "remeshed"),
    ] {
        commands.push(operation(root, output, name, &mut editor)?);
        checkpoints.insert(
            checkpoint_name,
            checkpoint(output, checkpoint_name, &editor)?,
        );
    }
    let mut editor = load(root, output, "removal_loaded")?;
    checkpoints.insert(
        "removal_loaded",
        checkpoint(output, "removal_loaded", &editor)?,
    );
    for (name, checkpoint_name) in [
        ("remove_piece", "removed"),
        ("undo_removal", "undo_removal"),
        ("redo_removal", "redo_removal"),
    ] {
        commands.push(operation(root, output, name, &mut editor)?);
        checkpoints.insert(
            checkpoint_name,
            checkpoint(output, checkpoint_name, &editor)?,
        );
    }
    write_json(
        &output.join("index.json"),
        &json!({"commands":commands,"checkpoints":checkpoints}),
    )?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("chain-sewing-operations-producer: {error}");
        std::process::exit(1);
    }
}
