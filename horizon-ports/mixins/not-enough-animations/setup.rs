fn pose(
    m: &Mob,
    s: &State,
    p: &mut Poses,
    w_anims: &HashMap<String, super::keyframes::Animation>,
) -> core::ops::ControlFlow<()> {
    crate::mobs::anim::not_enough_animations::pose(m, s, p, w_anims)
}
