#[derive(Clone, Copy, Debug)]
pub struct WeightedPoint {
    pub vertices: [usize; 2],
    pub weights: [f64; 2],
}

impl WeightedPoint {
    pub fn vertex(vertex: usize) -> Self {
        Self {
            vertices: [vertex; 2],
            weights: [1., 0.],
        }
    }
    fn position(self, x: &[Vec3]) -> Vec3 {
        add(
            mul(x[self.vertices[0]], self.weights[0]),
            mul(x[self.vertices[1]], self.weights[1]),
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stitch {
    pub a: WeightedPoint,
    pub b: WeightedPoint,
    /// m/N; zero creates a hard coincidence constraint.
    pub compliance: f64,
}

