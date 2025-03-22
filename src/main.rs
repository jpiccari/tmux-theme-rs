use std::{
    cell::OnceCell,
    collections::HashMap,
    fs,
    io::BufWriter,
    io::Write,
    path::{Path, PathBuf},
};

use batt_status::batt_status;
use clap::Parser;
use git_status::git_status;
use serde::Deserialize;
use themes::{Style, Theme};
use tmux_details::{tmux_get_variables, tmux_status};
use user_details::user_details;

mod batt_status;
mod git_status;
mod themes;
mod tmux_details;
mod user_details;

const THEME_FILE_NAME: &str = "tmux_theme.toml";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    theme: Option<String>,
    section: Component,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Component {
    TmuxLeft,
    TmuxRight,
}

struct StatusContext {
    theme: Theme,
    tmux_variables: OnceCell<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
struct Config {
    styles: HashMap<Style, String>,
}

impl StatusContext {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            tmux_variables: OnceCell::new(),
        }
    }

    fn get_tmux_variable(&self, key: &str) -> Option<&String> {
        let var_map = self
            .tmux_variables
            .get_or_init(|| tmux_get_variables().unwrap_or_default());
        var_map.get(key)
    }
}

fn main() {
    let args = Args::parse();
    let config_path = &get_config_path(&args.theme);
    let config = load_config(config_path);
    let theme = themes::Theme::from(config.styles);
    let ctx = StatusContext::new(theme);
    let mut output_buf = BufWriter::new(std::io::stdout());

    match args.section {
        Component::TmuxLeft => tmux_left(&ctx, &mut output_buf),
        Component::TmuxRight => tmux_right(&ctx, &mut output_buf),
    };
}

fn load_config(path: &PathBuf) -> Config {
    let content = fs::read_to_string(path).expect("Unable to read theme");
    toml::from_str(&content).expect("Unable to parse theme")
}

fn get_config_path(user_value: &Option<String>) -> PathBuf {
    if let Some(user_path) = user_value {
        return PathBuf::from(&user_path);
    }

    if let Ok(home) = std::env::var("HOME") {
        let home_theme_path = Path::new(&home).join(".config").join(THEME_FILE_NAME);
        if home_theme_path.exists() {
            return home_theme_path;
        }
    }

    std::env::current_dir()
        .expect("Unable to get current directory")
        .join(THEME_FILE_NAME)
}

fn tmux_left(ctx: &StatusContext, buf: &mut impl Write) {
    tmux_status(ctx, buf);
    git_status(ctx, buf);
}

fn tmux_right(ctx: &StatusContext, buf: &mut impl Write) {
    user_details(ctx, buf);
    batt_status(ctx, buf);
}
