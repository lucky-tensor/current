pub mod release;
pub mod builder;
//////// 0L ///////
/// Returns the release bundle for the current code.
pub fn head_release_bundle() -> zapatos_framework::ReleaseBundle {
    release::ReleaseTarget::Head.load_bundle().expect("release build failed")
}
