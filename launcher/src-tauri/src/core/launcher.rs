use tracing::info_span;

pub fn launch_game() {
    let _span = info_span!("launch", task="prepare").entered();
    tracing::info!("Preparing to launch Minecraft...");
    // Stub
}
