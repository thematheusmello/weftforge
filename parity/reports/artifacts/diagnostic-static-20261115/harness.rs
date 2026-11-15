use std::io::{BufWriter, Write};

type Vec3 = [f64; 3];
mod strain {
    include!("sources/strain.rs");

    pub fn sum_self_check() {
        let small = 2_f64.powi(-53);
        assert_eq!(1. + small + small, 1.);
        assert_eq!(compensated_sum([1., small, small]), 1. + 2_f64.powi(-52));
        assert!(compensated_sum([f64::MAX, f64::MAX, 0.]).is_infinite());
        assert!(compensated_sum([f64::NAN, 0., 0.]).is_nan());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Peak {
    triangle: usize,
    axis: usize,
    signed: f64,
}

fn scan(
    positions: &[Vec3],
    uv: &[[f64; 2]],
    triangles: &[[usize; 3]],
    grains: &[f64],
) -> (f64, Peak) {
    let mut maximum = 0.;
    let mut winner: Option<Peak> = None;
    for (triangle, &vertices) in triangles.iter().enumerate() {
        for (axis, signed) in strain::signed_strains(
            vertices.map(|v| positions[v]),
            vertices.map(|v| uv[v]),
            grains[triangle],
        )
        .into_iter()
        .enumerate()
        {
            assert!(signed.is_finite());
            if winner.is_none_or(|peak| signed.abs() > peak.signed.abs()) {
                maximum = signed.abs();
                winner = Some(Peak {
                    triangle,
                    axis,
                    signed,
                });
            }
        }
    }
    let peak = winner.unwrap();
    assert_eq!(maximum.to_bits(), peak.signed.abs().to_bits());
    (maximum, peak)
}

fn self_checks() {
    strain::sum_self_check();
    let uv = [[0., 0.], [1., 0.], [0., 1.], [1., 1.]];
    let triangles = [[0, 1, 2], [1, 3, 2]];
    let original = [[0., 0., 0.], [1.5, 0., 0.], [0., 1., 0.], [1.5, 1., 0.]];
    let mut checks = 4;
    for grain in [0., std::f64::consts::FRAC_PI_2] {
        let axis = usize::from(grain != 0.);
        for positions in [
            original,
            original.map(|[x, y, z]| [x + 8., y - 4., z + 2.]),
            original.map(|[x, y, z]| [z, x, y]),
            original.map(|[x, y, z]| [-x, y, z]),
        ] {
            assert_eq!(
                scan(&positions, &uv, &triangles, &[grain; 2]),
                (
                    0.5,
                    Peak {
                        triangle: 0,
                        axis,
                        signed: 0.5
                    }
                )
            );
            checks += 1;
        }
        let mut perturbed = original;
        perturbed[3][0] += 2_f64.powi(-51);
        let signed = 0.5 + 2_f64.powi(-51);
        assert_eq!(
            scan(&perturbed, &uv, &triangles, &[grain; 2]),
            (
                signed,
                Peak {
                    triangle: 1,
                    axis,
                    signed
                }
            )
        );
        perturbed[3][0] = 1.625;
        assert_eq!(
            scan(&perturbed, &uv, &triangles, &[grain; 2]),
            (
                0.625,
                Peak {
                    triangle: 1,
                    axis,
                    signed: 0.625
                }
            )
        );
        checks += 2;
    }
    // A shared material-u edge with distinct v derivatives remains a rounded near tie.
    let uv_edge = [[0., 0.], [1., 0.], [0., 1.], [1., -1.]];
    let edge_positions = [[0., 0., 0.], [1.5, 0., 0.], [0.25, 1., 0.], [1.75, -1., 0.]];
    assert_eq!(
        scan(
            &edge_positions,
            &uv_edge,
            &[[0, 1, 2], [1, 0, 3]],
            &[std::f64::consts::FRAC_PI_2; 2]
        ),
        (
            0.5,
            Peak {
                triangle: 0,
                axis: 1,
                signed: 0.5
            }
        )
    );
    checks += 1;
    let affine_uv = [[2., 3.], [2.5, 3.125], [1.75, 3.5]];
    for [u_scale, shear, v_scale] in [[1.5, 0.25, 0.75], [0.75, 0.125, 1.5]] {
        let positions =
            affine_uv.map(|[u, v]| [10. - v_scale * v, 2. + u_scale * u + shear * v, -0.5]);
        for grain in [0., 0.37, std::f64::consts::FRAC_PI_2] {
            let (sin, cos) = grain.sin_cos();
            let expected = [
                (u_scale * cos + shear * sin).hypot(v_scale * sin) - 1.,
                (-u_scale * sin + shear * cos).hypot(v_scale * cos) - 1.,
            ];
            let axis = usize::from(expected[1].abs() > expected[0].abs());
            for triangle in [[0, 1, 2], [0, 2, 1]] {
                let (maximum, peak) = scan(&positions, &affine_uv, &[triangle], &[grain]);
                let actual = strain::signed_strains(
                    triangle.map(|v| positions[v]),
                    triangle.map(|v| affine_uv[v]),
                    grain,
                );
                for i in 0..2 {
                    assert!((actual[i] - expected[i]).abs() < 1e-13);
                }
                assert_eq!(peak.triangle, 0);
                assert_eq!(peak.axis, axis);
                assert!((peak.signed - expected[axis]).abs() < 1e-13);
                assert_eq!(maximum.to_bits(), peak.signed.abs().to_bits());
                checks += 1;
            }
        }
    }
    // Stored pi/2 has nonzero cosine; a diagnostic must not snap it to zero.
    let grain = std::f64::consts::FRAC_PI_2;
    let axes = strain::signed_strains(
        [[0.; 3], [1., 0., 0.], [1e16, 1., 0.]],
        [[0., 0.], [1., 0.], [0., 1.]],
        grain,
    );
    let expected_weft = (-grain.sin() + grain.cos() * 1e16).hypot(grain.cos()) - 1.;
    assert_eq!(axes[1], expected_weft);
    assert_ne!(axes[1], 0.);
    checks += 1;
    eprintln!("{{\"static_self_checks\":{checks},\"status\":\"passed\"}}");
}

struct Reader<'a> {
    data: &'a [u8],
    cursor: usize,
}
impl Reader<'_> {
    fn u64(&mut self) -> u64 {
        let value = u64::from_le_bytes(self.data[self.cursor..self.cursor + 8].try_into().unwrap());
        self.cursor += 8;
        value
    }
    fn usize(&mut self) -> usize {
        self.u64().try_into().unwrap()
    }
    fn f64(&mut self) -> f64 {
        f64::from_bits(self.u64())
    }
    fn vec3(&mut self) -> Vec3 {
        [self.f64(), self.f64(), self.f64()]
    }
}

fn main() {
    self_checks();
    let data = std::fs::read("inputs.bin").unwrap();
    assert_eq!(&data[..8], b"WFSTRN01");
    let mut reader = Reader {
        data: &data,
        cursor: 8,
    };
    let mut axes_output = BufWriter::new(std::fs::File::create("actual-axes.bin").unwrap());
    let mut states = 0;
    let mut axis_count = 0;
    for _ in 0..reader.usize() {
        let case_id = reader.usize();
        let vertices = reader.usize();
        let triangle_count = reader.usize();
        let frame_count = reader.usize();
        let uv: Vec<_> = (0..vertices)
            .map(|_| [reader.f64(), reader.f64()])
            .collect();
        let triangles: Vec<_> = (0..triangle_count)
            .map(|_| [reader.usize(), reader.usize(), reader.usize()])
            .collect();
        let grains: Vec<_> = (0..triangle_count).map(|_| reader.f64()).collect();
        for _ in 0..frame_count {
            let frame = reader.usize();
            let positions: Vec<_> = (0..vertices).map(|_| reader.vec3()).collect();
            let (maximum, peak) = scan(&positions, &uv, &triangles, &grains);
            for (triangle, &indices) in triangles.iter().enumerate() {
                let axes = strain::signed_strains(
                    indices.map(|v| positions[v]),
                    indices.map(|v| uv[v]),
                    grains[triangle],
                );
                for value in axes {
                    axes_output.write_all(&value.to_le_bytes()).unwrap();
                    axis_count += 1;
                }
            }
            println!(
                "{{\"case_id\":{case_id},\"frame\":{frame},\"maximum_bits\":{},\"triangle\":{},\"axis\":{},\"signed_bits\":{}}}",
                maximum.to_bits(),
                peak.triangle,
                peak.axis,
                peak.signed.to_bits()
            );
            states += 1;
        }
    }
    axes_output.flush().unwrap();
    assert_eq!(reader.cursor, data.len());
    assert_eq!(states, 362);
    assert_eq!(axis_count, 838392);
    eprintln!("{{\"states\":{states},\"triangle_axes\":{axis_count},\"new_physics_steps\":0}}");
}
