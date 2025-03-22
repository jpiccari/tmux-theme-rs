use std::io::Write;

use crate::{StatusContext, themes::Style};

pub fn user_details(ctx: &StatusContext, buf: &mut impl Write) {
    if let Ok(user) = std::env::var("USER") {
        let _ = write!(
            buf,
            "  {}{}  ",
            ctx.theme.get_style(Style::UserDetails),
            &user
        );
    }
}
