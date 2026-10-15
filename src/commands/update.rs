use anyhow::Result;
use self_update::cargo_crate_version;

pub fn update() -> Result<()> {
    let target = self_update::get_target();

    let ext = if target.contains("windows") {
        ".exe"
    } else {
        ""
    };

    let asset_name = format!("ttcore-clip-preparer-{}{}", target, ext);

    let status = self_update::backends::github::Update::configure()
        .repo_owner("thatgurkangurk")
        .repo_name("ttcore-clip-preparer")
        .bin_name("ttcore-clip-preparer")
        .target(target)
        .bin_name(&asset_name)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .show_output(true)
        .build()?
        .update()?;

    if status.updated() {
        println!("Updated to {}", status.version());
    } else {
        println!("Already up to date.");
    }

    Ok(())
}
