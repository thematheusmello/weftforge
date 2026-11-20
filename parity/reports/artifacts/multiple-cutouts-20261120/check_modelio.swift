import Foundation
import ModelIO
import simd

let asset = MDLAsset(url: URL(fileURLWithPath: CommandLine.arguments[1]))
let meshes = asset.childObjects(of: MDLMesh.self) as! [MDLMesh]
precondition(meshes.count == 1)
let mesh = meshes[0]
let positions = mesh.vertexAttributeData(forAttributeNamed: MDLVertexAttributePosition, as: .float3)!
func point(_ i: Int) -> SIMD3<Double> {
    precondition((0..<mesh.vertexCount).contains(i))
    let x = positions.dataStart.advanced(by: positions.stride * i).assumingMemoryBound(to: Float.self)
    return SIMD3(Double(x[0]), Double(x[1]), Double(x[2]))
}
let surfaces = mesh.submeshes! as! [MDLSubmesh]
precondition(surfaces.count == 1)
let face = surfaces[0]
precondition(face.geometryType == .triangles && face.indexCount % 3 == 0)
let data = face.indexBuffer(asIndexType: .uInt32).map()
let index = data.bytes.assumingMemoryBound(to: UInt32.self)
var edges = [SIMD2<Int32>: Int]()
var area = 0.0
for i in stride(from: 0, to: face.indexCount, by: 3) {
    let ids = (0..<3).map { Int(index[i + $0]) }
    let vertex = ids.map(point)
    area += simd_length(simd_cross(vertex[1] - vertex[0], vertex[2] - vertex[0])) / 2
    for k in 0..<3 {
        let a = Int32(ids[k]), b = Int32(ids[(k+1)%3])
        edges[SIMD2(min(a,b),max(a,b)),default:0] += 1
    }
}
var neighbors = [Int32:[Int32]]()
for (edge,count) in edges {
    precondition(count == 1 || count == 2)
    if count == 1 {
        neighbors[edge.x,default:[]].append(edge.y)
        neighbors[edge.y,default:[]].append(edge.x)
    }
}
precondition(neighbors.values.allSatisfy { $0.count == 2 })
var unseen = Set(neighbors.keys)
var loops = 0
while let first = unseen.first {
    var stack = [first]
    while let i = stack.popLast() {
        if unseen.remove(i) != nil { stack += neighbors[i]! }
    }
    loops += 1
}
let euler = mesh.vertexCount - edges.count + face.indexCount / 3
precondition(loops == 3 && euler == -1 && abs(area - 0.0756) < 1e-8)
let report: [String:Any] = ["vertices":mesh.vertexCount,"faces":face.indexCount/3,
                            "boundary_loops":loops,"euler_characteristic":euler,"area_m2":area]
let json = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted,.sortedKeys])
print(String(decoding:json,as:UTF8.self))
