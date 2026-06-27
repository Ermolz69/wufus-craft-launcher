use tracing::info_span;

pub fn check_for_updates() {
    let _span = info_span!("update", task="check").entered();
    tracing::info!("Checking for updates...");
    // Stub
}
