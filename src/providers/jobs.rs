use smol_str::SmolStr;

use crate::{
    chunk::{Adjoin, Chunk},
    options::Options,
    style::to_superscript,
};

fn format_jobs_count(count: u32) -> SmolStr {
    let mut buf = itoa::Buffer::new();
    to_superscript(buf.format(count))
}

pub async fn show(opts: &Options) -> Option<Chunk<Adjoin<&'static str, SmolStr>>> {
    if !opts.jobs {
        return None;
    }
    let count = opts.jobs_count?;
    if count == 0 {
        return None;
    }
    Some(Chunk::info(Adjoin((
        opts.select_str("✦", "󰒲"),
        format_jobs_count(count),
    ))))
}

#[cfg(test)]
mod tests {
    use super::{format_jobs_count, show};
    use crate::options::Options;

    #[test]
    fn test_format_jobs_count() {
        assert_eq!(format_jobs_count(1), "¹");
        assert_eq!(format_jobs_count(2), "²");
        assert_eq!(format_jobs_count(12), "¹²");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_jobs_show_visibility() {
        let mut opts = Options {
            jobs: true,
            jobs_count: None,
            ..Default::default()
        };
        assert!(show(&opts).await.is_none());

        opts.jobs_count = Some(0);
        assert!(show(&opts).await.is_none());

        opts.jobs_count = Some(3);
        assert_eq!(show(&opts).await.unwrap().to_string(), "✦³");

        opts.nerd_font = true;
        assert_eq!(show(&opts).await.unwrap().to_string(), "󰒲³");

        opts.jobs = false;
        assert!(show(&opts).await.is_none());
    }
}
