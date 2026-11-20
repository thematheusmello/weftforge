import Foundation
import ModelIO
import simd

let url = URL(fileURLWithPath: CommandLine.arguments[1])
let scene = MDLAsset(url: url)
let meshes = scene.childObjects(of: MDLMesh.self) as! [MDLMesh]
var result: [[String:Any]] = []
for mesh in meshes {
    for submesh in mesh.submeshes as? [MDLSubmesh] ?? [] {
        guard let material = submesh.material, let base = material.property(with:.baseColor) else {
            fatalError("Missing diffuse fabric on \(mesh.name)")
        }
        let rgb: [Float]
        switch base.type {
        case .float3:
            let v = base.float3Value
            rgb = [v.x,v.y,v.z]
        case .float4:
            let v = base.float4Value
            rgb = [v.x,v.y,v.z]
        case .color:
            guard let values = base.color?.components else {fatalError("Missing RGB components")}
            rgb = values.prefix(3).map(Float.init)
        default: fatalError("Unexpected material value \(base.type.rawValue)")
        }
        result.append(["mesh":mesh.name,"submesh":submesh.name,"vertices":mesh.vertexCount,"indices":submesh.indexCount,"material":material.name,"diffuse_rgb":rgb])
    }
}
let data = try JSONSerialization.data(withJSONObject:result,options:[.sortedKeys,.prettyPrinted])
print(String(decoding:data,as:UTF8.self))
