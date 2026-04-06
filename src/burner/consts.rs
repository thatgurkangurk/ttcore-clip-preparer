use std::time::Duration;

// text render config
pub const FONT_SIZE: u32 = 34;
pub const LINE_SPACING: u32 = 6;
pub const PADDING_RIGHT: u32 = 50;
pub const PADDING_BOTTOM: u32 = 40;

/// time taken for a single slide transition (applies separately to entering and exiting)
pub const SLIDE_DUR: Duration = Duration::from_secs(1);
/// time taken for a single fade transition (applies separately to fading in and fading out)
pub const FADE_DUR: Duration = Duration::from_millis(500);
/// the exact timestamp when text 1 finishes exiting and text 2 begins entering
pub const SWITCH_TIME: Duration = Duration::from_secs(4);
/// delay applied to the bottom line so it animates slightly after the top line
pub const LINE_STAGGER: Duration = Duration::from_millis(150);

pub const BASE_SCALE_FILTER: &str =
    "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2";

pub const INTRO_LINE_1: &str = "unless specified";
pub const INTRO_LINE_2: &str = "all usernames are for discord";

pub const OUTRO_LINE_1: &str = "thank you for watching";
pub const OUTRO_LINE_2: &str = "please like and subscribe :)";
