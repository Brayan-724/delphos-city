use delphos_ecs::Resource;

#[derive(Default)]
pub struct Time {
    /// Millisecond of the day
    pub started: u32,
    /// Time elapsed since program started
    pub elapsed: u32,
    /// Current frame index
    pub frame_count: u32,
    pub(crate) last_frame_count: u32,
    pub(crate) update_frame_count: u32,
    /// Time elapsed since last frame in milliseconds
    pub delta: u32,
    pub(crate) last_delta: u32,
    pub(crate) average_delta: u32,
}

impl Resource for Time {}
