use frunk::hlist;
use frunk::poly_fn;
use frunk::HCons;
use frunk::HNil;
use owo_colors::Styled;
use std::future::Future;
use std::sync::Arc;
use tokio::task::JoinError;

use crate::style::build_bold_style;
use crate::style::build_color_style;
use crate::providers::git::git_ahead_behind_icon;
use crate::providers::git::git_branch_icon;
use crate::providers::git::git_branch_name;
use crate::providers::git::git_commit_name;
use crate::providers::git::git_describe;
use crate::providers::git::git_stash_counter;
use crate::providers::git::git_status_icon;
use crate::providers::git::git_worktree;
use crate::providers::netns::namespace as net_namespace;
use crate::providers::ssh::show as ssh_show;
use crate::providers::netif::show as netif_show;
use crate::providers::os::show as os_show;
use crate::providers::manifest::show as manifest_show;
use crate::providers::virt::show as virt_show;
use crate::providers::exit_code::show as exit_code_show;

use crate::Options;
use owo_colors::Style;

macro_rules! item {
    ($provider:expr, $opt:expr) => {{
        let cloned_opts = Arc::clone(&$opt);
        tokio::spawn(async move { $provider(&cloned_opts).await.map(|s| Style::new().style(s)) })
    }};
    ($provider:expr, $opt:expr, $styl:expr) => {{
        let cloned_opts = Arc::clone(&$opt);
        let style = $styl;
        tokio::spawn(async move { $provider(&cloned_opts).await.map(|s| style.style(s)) })
    }};
}

type ResultStatic = Result<Option<Styled<&'static str>>, JoinError>;
type ResultString = Result<Option<Styled<String>>, JoinError>;

pub async fn print_prompt(opts: Arc<Options>) -> Result<(), JoinError> {
    let path = opts.path.clone();
    with_path(&path, async move {
        let color_style = build_color_style(opts.theme.as_deref());
        let bold_style = build_bold_style();
        let styled_prompt = hlist! [
            item![ os_show, opts, color_style ],
            item![ virt_show, opts, bold_style ],
            item![ ssh_show,opts, bold_style ],
            item![ netif_show, opts, bold_style],
            item![ net_namespace, opts, color_style ],
            item![ manifest_show, opts, color_style ],
            item![ git_branch_icon, opts ],
            item![ git_status_icon, opts, color_style ],
            item![ git_stash_counter, opts ],
            item![ git_worktree, opts, bold_style ],
            item![ git_branch_name, opts, color_style.bold() ],
            item![ git_commit_name, opts, bold_style ],
            item![ git_describe, opts, bold_style ],
            item![ git_ahead_behind_icon, opts],
            item![ exit_code_show, opts, bold_style.red()],
        ];

        let styled_results = styled_prompt.hjoin().await;

        styled_results.map(
            poly_fn!(
                |s: ResultStatic| -> () {
                   if let Ok(Some(s)) = s {
                       print!("{s} ")
                   }
                },
                |s: ResultString| -> () {
                    if let Ok(Some(s)) = s {
                        print!("{s} ")
                    }
                },
            )
        );

        Ok(())
    })
    .await
}

async fn with_path<F>(path: &Option<String>, action: F) -> Result<(), JoinError>
where
    F: Future<Output = Result<(), JoinError>>,
{
    match path {
        Some(p) => {
            let cur = std::env::current_dir().unwrap();
            std::env::set_current_dir(p).expect("could not change directory");
            let result = action.await;
            std::env::set_current_dir(cur).expect("could not change directory");
            result
        }
        None => action.await,
    }
}

trait HListJoin {
    type Output;
    async fn hjoin(self) -> Self::Output;
}

impl HListJoin for HNil {
    type Output = HNil;
    async fn hjoin(self) -> Self::Output {
        HNil
    }
}

impl<H, T> HListJoin for HCons<H, T>
where
    H: Future + Send,
    T: HListJoin + Send,
    T::Output: Send,
{
    type Output = HCons<H::Output, T::Output>;
    async fn hjoin(self) -> Self::Output {
        let (head_res, tail_res) = tokio::join!(self.head, self.tail.hjoin());
        HCons {
            head: head_res,
            tail: tail_res,
        }
    }
}
