use duct::cmd;
use std::{collections::HashMap, io::Error, io::Write};

use crate::StatusContext;

pub fn tmux_get_variables() -> Result<HashMap<String, String>, Error> {
    let mut result: HashMap<String, String> = HashMap::new();
    let tmux_varaibles = cmd!("tmux", "display-message", "-a").read()?;
    for line in tmux_varaibles.lines() {
        if let Some((key, value)) = line.split_once("=") {
            result.insert(key.to_string(), value.to_string());
        }
    }

    Ok(result)
}

pub fn tmux_status(ctx: &StatusContext, buf: &mut impl Write) {
    if let Some(index) = ctx.get_tmux_variable("window_index") {
        let _ = write!(buf, " {} {} ", nerdfonts::cod::COD_WINDOW, index);
    }
}
