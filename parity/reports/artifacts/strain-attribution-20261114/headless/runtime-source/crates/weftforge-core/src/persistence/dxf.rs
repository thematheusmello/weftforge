use crate::{Error, Project, Result, ensure};
use std::fmt::Write as _;

// Fixed drawing infrastructure owns handles 1..=1B; layers and curves follow.
const FIRST_DYNAMIC_HANDLE: usize = 0x1c;

pub(super) fn render(project: &Project) -> Result<String> {
    let mut pieces: Vec<_> = project.pieces.iter().collect();
    pieces.sort_unstable_by_key(|piece| piece.id);
    let handseed = FIRST_DYNAMIC_HANDLE + pieces.len() + project.curves.len();
    let mut output = format!(
        "0\nSECTION\n2\nHEADER\n999\nWeftForge FILE-005-v1: pattern boundaries in millimetres\n999\nSource document revision: {}\n999\nUV->XY; positive V->positive Y; Z=0; boundary-only\n9\n$ACADVER\n1\nAC1015\n9\n$INSUNITS\n70\n4\n9\n$MEASUREMENT\n70\n1\n9\n$LUNITS\n70\n2\n9\n$HANDSEED\n5\n{handseed:X}\n0\nENDSEC\n0\nSECTION\n2\nCLASSES\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n",
        project.revision
    );
    output.push_str(
        "0\nTABLE\n2\nVPORT\n5\n1\n330\n0\n100\nAcDbSymbolTable\n70\n0\n0\nENDTAB\n\
         0\nTABLE\n2\nLTYPE\n5\n2\n330\n0\n100\nAcDbSymbolTable\n70\n3\n",
    );
    for (handle, name) in [(3, "BYBLOCK"), (4, "BYLAYER"), (5, "CONTINUOUS")] {
        write!(
            output,
            "0\nLTYPE\n5\n{handle:X}\n330\n2\n100\nAcDbSymbolTableRecord\n100\nAcDbLinetypeTableRecord\n2\n{name}\n70\n0\n3\n\n72\n65\n73\n0\n40\n0\n"
        )
        .expect("Writing a String cannot fail");
    }
    write!(
        output,
        "0\nENDTAB\n0\nTABLE\n2\nLAYER\n5\n6\n330\n0\n100\nAcDbSymbolTable\n70\n{}\n",
        pieces.len() + 1
    )
    .expect("Writing a String cannot fail");
    layer(&mut output, 7, "0");
    let mut handle = FIRST_DYNAMIC_HANDLE;
    for piece in &pieces {
        layer(&mut output, handle, &format!("WF_PIECE_{}", piece.id));
        handle += 1;
    }
    output.push_str(
        "0\nENDTAB\n\
         0\nTABLE\n2\nSTYLE\n5\n8\n330\n0\n100\nAcDbSymbolTable\n70\n1\n\
         0\nSTYLE\n5\n9\n330\n8\n100\nAcDbSymbolTableRecord\n100\nAcDbTextStyleTableRecord\n2\nSTANDARD\n70\n0\n40\n0\n41\n1\n50\n0\n71\n0\n42\n2.5\n3\ntxt\n4\n\n0\nENDTAB\n\
         0\nTABLE\n2\nVIEW\n5\nA\n330\n0\n100\nAcDbSymbolTable\n70\n0\n0\nENDTAB\n\
         0\nTABLE\n2\nUCS\n5\nB\n330\n0\n100\nAcDbSymbolTable\n70\n0\n0\nENDTAB\n\
         0\nTABLE\n2\nAPPID\n5\nC\n330\n0\n100\nAcDbSymbolTable\n70\n1\n\
         0\nAPPID\n5\nD\n330\nC\n100\nAcDbSymbolTableRecord\n100\nAcDbRegAppTableRecord\n2\nACAD\n70\n0\n0\nENDTAB\n\
         0\nTABLE\n2\nDIMSTYLE\n5\nE\n330\n0\n100\nAcDbSymbolTable\n70\n1\n100\nAcDbDimStyleTable\n71\n1\n\
         0\nDIMSTYLE\n105\nF\n330\nE\n100\nAcDbSymbolTableRecord\n100\nAcDbDimStyleTableRecord\n2\nSTANDARD\n70\n0\n340\n9\n0\nENDTAB\n\
         0\nTABLE\n2\nBLOCK_RECORD\n5\n10\n330\n0\n100\nAcDbSymbolTable\n70\n2\n\
         0\nBLOCK_RECORD\n5\n11\n330\n10\n100\nAcDbSymbolTableRecord\n100\nAcDbBlockTableRecord\n2\n*Model_Space\n340\n1A\n\
         0\nBLOCK_RECORD\n5\n12\n330\n10\n100\nAcDbSymbolTableRecord\n100\nAcDbBlockTableRecord\n2\n*Paper_Space\n340\n1B\n\
         0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n\
         0\nBLOCK\n5\n13\n330\n11\n100\nAcDbEntity\n8\n0\n100\nAcDbBlockBegin\n2\n*Model_Space\n70\n0\n10\n0\n20\n0\n30\n0\n3\n*Model_Space\n1\n\n\
         0\nENDBLK\n5\n14\n330\n11\n100\nAcDbEntity\n8\n0\n100\nAcDbBlockEnd\n\
         0\nBLOCK\n5\n15\n330\n12\n100\nAcDbEntity\n67\n1\n8\n0\n100\nAcDbBlockBegin\n2\n*Paper_Space\n70\n0\n10\n0\n20\n0\n30\n0\n3\n*Paper_Space\n1\n\n\
         0\nENDBLK\n5\n16\n330\n12\n100\nAcDbEntity\n67\n1\n8\n0\n100\nAcDbBlockEnd\n\
         0\nENDSEC\n0\nSECTION\n2\nENTITIES\n",
    );
    for piece in pieces {
        for id in &piece.boundary {
            let curve = project
                .curve(*id)
                .ok_or_else(|| Error("Missing boundary curve".into()))?;
            let endpoint = |id| {
                millimetres(
                    project
                        .point(id)
                        .ok_or_else(|| Error("Missing boundary point".into()))?
                        .position,
                )
            };
            let start = endpoint(curve.start)?;
            let end = endpoint(curve.end)?;
            let kind = if curve.controls.is_some() {
                "SPLINE"
            } else {
                "LINE"
            };
            write!(
                output,
                "0\n{kind}\n5\n{handle:X}\n330\n11\n100\nAcDbEntity\n8\nWF_PIECE_{}\n67\n0\n410\nModel\n370\n-1\n",
                piece.id
            )
            .expect("Writing a String cannot fail");
            handle += 1;
            if let Some([a, b]) = curve.controls {
                let a = millimetres(a)?;
                let b = millimetres(b)?;
                output.push_str(
                    "100\nAcDbSpline\n210\n0\n220\n0\n230\n1\n70\n8\n71\n3\n72\n8\n73\n4\n74\n0\n40\n0\n40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n40\n1\n",
                );
                for [x, y] in [start, a, b, end] {
                    write!(output, "10\n{x}\n20\n{y}\n30\n0\n")
                        .expect("Writing a String cannot fail");
                }
            } else {
                write!(
                    output,
                    "100\nAcDbLine\n39\n0\n10\n{}\n20\n{}\n30\n0\n11\n{}\n21\n{}\n31\n0\n",
                    start[0], start[1], end[0], end[1]
                )
                .expect("Writing a String cannot fail");
            }
        }
    }
    output.push_str(
        "0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n\
         0\nDICTIONARY\n5\n17\n330\n0\n100\nAcDbDictionary\n280\n0\n281\n1\n3\nACAD_GROUP\n350\n18\n3\nACAD_LAYOUT\n350\n19\n\
         0\nDICTIONARY\n5\n18\n330\n17\n100\nAcDbDictionary\n280\n0\n281\n1\n\
         0\nDICTIONARY\n5\n19\n330\n17\n100\nAcDbDictionary\n280\n0\n281\n1\n3\nModel\n350\n1A\n3\nLayout1\n350\n1B\n",
    );
    for (handle, name, order, space) in [(0x1a, "Model", 0, 0x11), (0x1b, "Layout1", 1, 0x12)] {
        write!(
            output,
            "0\nLAYOUT\n5\n{handle:X}\n330\n19\n100\nAcDbPlotSettings\n70\n{}\n100\nAcDbLayout\n1\n{name}\n70\n1\n71\n{order}\n10\n0\n20\n0\n11\n0\n21\n0\n12\n0\n22\n0\n32\n0\n14\n0\n24\n0\n34\n0\n15\n0\n25\n0\n35\n0\n146\n0\n13\n0\n23\n0\n33\n0\n16\n1\n26\n0\n36\n0\n17\n0\n27\n1\n37\n0\n76\n0\n330\n{space:X}\n",
            if order == 0 { 1024 } else { 0 }
        )
        .expect("Writing a String cannot fail");
    }
    output.push_str("0\nENDSEC\n0\nEOF\n");
    debug_assert_eq!(handle, handseed);
    Ok(output)
}

fn layer(output: &mut String, handle: usize, name: &str) {
    write!(
        output,
        "0\nLAYER\n5\n{handle:X}\n330\n6\n100\nAcDbSymbolTableRecord\n100\nAcDbLayerTableRecord\n2\n{name}\n70\n0\n62\n7\n6\nCONTINUOUS\n"
    )
    .expect("Writing a String cannot fail");
}

fn millimetres(point: [f64; 2]) -> Result<[f64; 2]> {
    let scaled = point.map(|value| value * 1000.0);
    ensure(
        scaled.iter().all(|value| value.is_finite()),
        "DXF millimetre coordinates must be finite",
    )?;
    Ok(scaled)
}
