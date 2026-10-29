fn play(
    audio: &mut crate::audio::Audio,
    event: &str,
    at: Option<[f64; 3]>,
) -> core::ops::ControlFlow<()> {
    crate::audio::presence_footsteps::intercept(audio, event, at)
}
