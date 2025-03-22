use std::io::Write;

use git2::{Oid, Reference, Repository};

use crate::{
    StatusContext,
    themes::{Style, Theme},
};

const UNSTAGED_CHANGES_MASK: u32 = libgit2_sys::GIT_STATUS_WT_NEW
    | libgit2_sys::GIT_STATUS_WT_MODIFIED
    | libgit2_sys::GIT_STATUS_WT_RENAMED
    | libgit2_sys::GIT_STATUS_WT_DELETED;
const STAGED_CHANGES_MASK: u32 = libgit2_sys::GIT_STATUS_INDEX_NEW
    | libgit2_sys::GIT_STATUS_INDEX_MODIFIED
    | libgit2_sys::GIT_STATUS_INDEX_RENAMED
    | libgit2_sys::GIT_STATUS_INDEX_DELETED;

pub fn git_status(ctx: &StatusContext, buf: &mut impl Write) {
    if let Some(pane_current_path) = &ctx.get_tmux_variable("pane_current_path") {
        if let Ok(repo) = Repository::discover(pane_current_path) {
            print_status(&ctx.theme, &repo, buf);
        }
    }
}

fn print_status(theme: &Theme, repo: &Repository, buf: &mut impl Write) {
    let mut output = Vec::with_capacity(10);
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => todo!(),
    };

    {
        let reference_type = ReferenceType::from_ref_name(head.name());
        let (icon, ref_name) = match reference_type {
            ReferenceType::Tag(t) => (nerdfonts::md::MD_TAG, t),
            ReferenceType::RemoteBranch(rb) => (nerdfonts::dev::DEV_GIT_COMPARE, rb),
            ReferenceType::LocalBranch(lb) => (nerdfonts::dev::DEV_GIT_BRANCH, lb),
            ReferenceType::Head => (nerdfonts::md::MD_HEAD_ALERT, String::from("HEAD")),
        };

        let ref_str = format!("{} {}", icon, ref_name);
        output.push(theme.get_style_str(Style::GitBranch, &ref_str));
    }

    if let Ok(statuses) = repo.statuses(None) {
        let mut combined_status = 0u32;
        let mut untracked = false;
        for entry in statuses.into_iter() {
            combined_status |= entry.status().bits();
            if !untracked && ((combined_status & libgit2_sys::GIT_STATUS_IGNORED) != 0) {
                if let Some(p) = entry.path() {
                    if let Ok(false) = repo.is_path_ignored(p) {
                        untracked = true;
                    }
                }
            }
        }

        if combined_status & STAGED_CHANGES_MASK != 0 {
            output.push(theme.get_style_char(Style::GitStaged, nerdfonts::fa::FA_ROCKET));
        }

        if combined_status & UNSTAGED_CHANGES_MASK != 0 {
            output.push(theme.get_style_char(Style::GitUnstaged, nerdfonts::fa::FA_THUMBS_UP));
        }

        if untracked {
            output.push(theme.get_style_char(Style::GitUntracked, nerdfonts::md::MD_CARDS_PLAYING));
        }
    }

    if let Some(head_ref_name) = head.name() {
        if let Ok(upstream_name) = repo.branch_upstream_name(head_ref_name) {
            if let Some(upstream_str) = upstream_name.as_str() {
                if let Ok(upstream) = repo.find_reference(upstream_str) {
                    let head_oid = ref_to_id(repo, &head);
                    let upstream_oid = ref_to_id(repo, &upstream);
                    if let Ok((ahead, behind)) =
                        repo.graph_ahead_behind(head_oid.unwrap(), upstream_oid.unwrap())
                    {
                        if ahead > 0 {
                            let text = &format!("{}{}", nerdfonts::fa::FA_ARROW_UP, ahead);
                            output.push(theme.get_style_str(Style::GitAhead, text));
                        }

                        if behind > 0 {
                            let text = &format!("{}{}", nerdfonts::fa::FA_ARROW_DOWN, behind);
                            output.push(theme.get_style_str(Style::GitBehind, text));
                        }
                    }
                }
            }
        }
    }

    let _ = write!(buf, " {} ", output.join(" "));
}

fn ref_to_id(repo: &Repository, reference: &Reference) -> Option<Oid> {
    let name = reference.name().unwrap();
    match repo.refname_to_id(name) {
        Ok(oid) => Some(oid),
        Err(_) => None,
    }
}

enum ReferenceType {
    Head,
    LocalBranch(String),
    RemoteBranch(String),
    Tag(String),
}

impl ReferenceType {
    fn from_ref_name(s: Option<&str>) -> Self {
        match s {
            None | Some("HEAD") => ReferenceType::Head,
            Some(ref_name) => match ref_name[5..].split_once("/") {
                Some(("tags", tag_name)) => ReferenceType::Tag(tag_name.to_string()),
                Some(("heads", branch_name)) => ReferenceType::LocalBranch(branch_name.to_string()),
                Some(("remotes", branch_name)) => {
                    ReferenceType::RemoteBranch(branch_name.to_string())
                }
                _ => ReferenceType::Head,
            },
        }
    }
}
