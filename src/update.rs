use anyhow::Result;

pub fn update() -> Result<()> {
    let target = self_update::get_target();

    let asset_name = match target {
        "x86_64-pc-windows-gnu" => "ttcore-clip-preparer-x86_64-pc-windows-gnu.exe",
        "x86_64-unknown-linux-gnu" => "ttcore-clip-preparer-x86_64-unknown-linux-gnu",
        _ => panic!("Unsupported target {}", target),
    };

    let status = self_update::backends::github::Update::configure()
        .repo_owner("thatgurkangurk")
        .repo_name("ttcore-clip-preparer")
        .bin_name("ttcore-clip-preparer")
        .target(&target)
        .bin_name(asset_name)
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    if status.updated() {
        println!("Updated to {}", status.version());
    } else {
        println!("Already up to date.");
    }

    Ok(())
}
