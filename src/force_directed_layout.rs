use kurbo::Vec2;

#[derive(Copy, Clone)]
pub struct ForceDirectedLayoutParams {
    pub dt: f64,
    pub l_0: f64,
    pub k_s: f64,
    pub k_r: f64,
    pub n_iterations: usize,
    pub scale: f64,
}

pub fn compute(
    n: usize,
    interaction_matrix: &[Vec<f64>],
    params: ForceDirectedLayoutParams,
) -> Vec<Vec2> {
    let mut positions: Vec<Vec2> = (0..n)
        .map(|x| x as f64 / n as f64 * 6.2831)
        .map(Vec2::from_angle)
        .map(|v| v + Vec2::new(1., 1.))
        .collect();

    // unite: L**3 T**(-1)
    let k_r = params.k_r * params.l_0.powf(3.0) / params.dt;
    // unite: T**(-1)
    let k_s = params.k_s / params.dt;

    for _ in 0..params.n_iterations {
        let mut velocities = vec![Vec2::ZERO; n];
        for n1 in 0..n {
            for n2 in 0..n1 {
                let d = positions[n2] - positions[n1];
                let dist = d.length();
                // unite: L/t
                let force_rep = -k_r / (dist * dist);
                let force_spring = k_s * interaction_matrix[n1][n2] * (dist - params.l_0);
                velocities[n1] += d * (force_rep + force_spring) / dist;
                velocities[n2] -= d * (force_rep + force_spring) / dist;
            }
        }
        for n1 in 0..n {
            positions[n1] += velocities[n1] * params.dt;
        }
    }
    positions.into_iter().map(|x| x * params.scale).collect()
}
