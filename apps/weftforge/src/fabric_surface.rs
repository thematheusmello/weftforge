use super::{Color32, Material};

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit(v: [f64; 3]) -> [f64; 3] {
    let length = dot(v, v).sqrt().max(1e-12);
    v.map(|x| x / length)
}

/// Flat, two-sided garment shading under the studio's single world-space key light.
pub(super) fn shade(t: [[f64; 3]; 3], view: [f64; 3], material: Option<&Material>) -> Color32 {
    const LIGHT: [f64; 3] = [0.3, 0.5, 0.8];
    let a: [f64; 3] = std::array::from_fn(|i| t[1][i] - t[0][i]);
    let b: [f64; 3] = std::array::from_fn(|i| t[2][i] - t[0][i]);
    let normal = unit([
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]);
    let base = material.map_or([0.3, 0.7, 0.65], |m| m.color);
    let surface = material.and_then(|m| m.surface).unwrap_or_default();
    let diffuse = (0.45 + 0.55 * dot(normal, LIGHT).abs().min(1.)) as f32;
    let [r, g, b] = if surface == Default::default() {
        base.map(|channel| (channel * diffuse * 255.) as u8)
    } else {
        let view = unit(view);
        let normal = if dot(normal, view) < 0. {
            normal.map(|x| -x)
        } else {
            normal
        };
        let light = unit(LIGHT);
        let half = unit(std::array::from_fn(|i| light[i] + view[i]));
        let nl = dot(normal, light).max(0.);
        let nv = dot(normal, view).max(0.);
        let nh = dot(normal, half).clamp(0., 1.);
        let vh = dot(view, half).clamp(0., 1.);
        // GGX distribution, Smith visibility and Schlick Fresnel. Limit the
        // preview lobe width at mirror roughness to keep a finite pixel sample.
        let a2 = surface.roughness.max(0.05).powi(4);
        let distribution = a2 / (std::f64::consts::PI * (nh * nh * (a2 - 1.) + 1.).powi(2));
        let smith_denominator = |cosine: f64| cosine + (a2 + (1. - a2) * cosine * cosine).sqrt();
        let specular = nl * distribution / (smith_denominator(nl) * smith_denominator(nv));
        // Fade the dielectric lobe into the studio's historical matte endpoint.
        let strength = 1. - (1. - surface.metallic) * surface.roughness;
        base.map(|channel| {
            let channel = f64::from(channel);
            let f0 = 0.04 * (1. - surface.metallic) + channel * surface.metallic;
            let fresnel = f0 + (1. - f0) * (1. - vh).powi(5);
            let body =
                channel * f64::from(diffuse) * (1. - surface.metallic) * (1. - strength * fresnel);
            let reflected = fresnel * (strength * specular + 0.18 * surface.metallic);
            ((body + reflected).clamp(0., 1.) * 255.) as u8
        })
    };
    Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use weftforge_core::FabricSurface;

    #[test]
    fn matte_is_camera_invariant_and_the_gloss_highlight_follows_the_half_vector() {
        let triangle = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        let mirror_view = unit([-0.3, -0.5, 0.8]);
        let off_axis = unit([0.7, 0.6, 0.3]);
        let mut material = Material {
            color: [0.3, 0.5, 0.7],
            ..Default::default()
        };
        let baseline = shade(triangle, mirror_view, Some(&material));
        assert_eq!(baseline, Color32::from_rgb(68, 113, 158));
        assert_eq!(baseline, shade(triangle, off_axis, Some(&material)));
        for metallic in [0., 0.8, 1.] {
            material.surface = Some(FabricSurface {
                metallic,
                roughness: 0.32,
            });
            let highlight = shade(triangle, mirror_view, Some(&material));
            let away = shade(triangle, off_axis, Some(&material));
            assert!(highlight.r() > away.r() && highlight.g() > away.g());
            assert_eq!(
                highlight,
                shade(
                    [triangle[2], triangle[1], triangle[0]],
                    mirror_view,
                    Some(&material)
                )
            );
            assert_eq!(away, shade(triangle, off_axis, Some(&material)));
        }
        material.surface = Some(FabricSurface {
            metallic: 1.,
            roughness: 0.,
        });
        let mirror = shade(triangle, mirror_view, Some(&material));
        let away = shade(triangle, off_axis, Some(&material));
        assert!(mirror.r() > away.r() && away.b() > away.r());
        material.surface = Some(FabricSurface {
            metallic: 1.,
            roughness: 1.,
        });
        assert!(shade(triangle, mirror_view, Some(&material)).r() < mirror.r());
    }
}
