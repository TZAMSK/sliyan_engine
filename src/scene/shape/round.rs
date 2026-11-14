#[allow(dead_code)]
pub trait Round {
    fn set_radius(&mut self);
    fn set_segments(&mut self);
}

pub struct RoundProperties {
    pub radius: f32,
    pub segments: usize,
}
