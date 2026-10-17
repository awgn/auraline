use crate::{chunk::Chunk, options::Options, style::to_superscript};
use smol_str::SmolStr;

fn signal_name(sig: u8) -> Option<&'static str> {
    match sig {
        x if x == libc::SIGHUP as u8 => Some("HUP"),
        x if x == libc::SIGINT as u8 => Some("INT"),
        x if x == libc::SIGQUIT as u8 => Some("QUIT"),
        x if x == libc::SIGILL as u8 => Some("ILL"),
        x if x == libc::SIGTRAP as u8 => Some("TRAP"),
        x if x == libc::SIGABRT as u8 => Some("ABRT"),
        x if x == libc::SIGBUS as u8 => Some("BUS"),
        x if x == libc::SIGFPE as u8 => Some("FPE"),
        x if x == libc::SIGKILL as u8 => Some("KILL"),
        x if x == libc::SIGUSR1 as u8 => Some("USR1"),
        x if x == libc::SIGSEGV as u8 => Some("SEGV"),
        x if x == libc::SIGUSR2 as u8 => Some("USR2"),
        x if x == libc::SIGPIPE as u8 => Some("PIPE"),
        x if x == libc::SIGALRM as u8 => Some("ALRM"),
        x if x == libc::SIGTERM as u8 => Some("TERM"),
        x if x == libc::SIGCHLD as u8 => Some("CHLD"),
        x if x == libc::SIGCONT as u8 => Some("CONT"),
        x if x == libc::SIGSTOP as u8 => Some("STOP"),
        x if x == libc::SIGTSTP as u8 => Some("TSTP"),
        x if x == libc::SIGTTIN as u8 => Some("TTIN"),
        x if x == libc::SIGTTOU as u8 => Some("TTOU"),
        x if x == libc::SIGURG as u8 => Some("URG"),
        x if x == libc::SIGXCPU as u8 => Some("XCPU"),
        x if x == libc::SIGXFSZ as u8 => Some("XFSZ"),
        x if x == libc::SIGVTALRM as u8 => Some("VTALRM"),
        x if x == libc::SIGPROF as u8 => Some("PROF"),
        x if x == libc::SIGWINCH as u8 => Some("WINCH"),
        x if x == libc::SIGIO as u8 => Some("IO"),
        x if x == libc::SIGSYS as u8 => Some("SYS"),
        _ => None,
    }
}

fn format_exit_code(exit_code: u8) -> SmolStr {
    if exit_code > 128 {
        if let Some(sig) = signal_name(exit_code - 128) {
            return SmolStr::new_static(sig);
        }
    }
    let mut buffer = itoa::Buffer::new();
    to_superscript(buffer.format(exit_code))
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    let exit_code = opts.exit_code?;
    if exit_code == 0 {
        None
    } else {
        Some(Chunk::new("✘", format_exit_code(exit_code)))
    }
}

#[cfg(test)]
mod tests {
    use super::format_exit_code;

    #[test]
    fn test_format_exit_code() {
        assert_eq!(format_exit_code(1), "¹");
        assert_eq!(format_exit_code(127), "¹²⁷");
        assert_eq!(format_exit_code(130), "INT");
        assert_eq!(format_exit_code(137), "KILL");
        assert_eq!(format_exit_code(139), "SEGV");
        assert_eq!(format_exit_code(143), "TERM");
        assert_eq!(format_exit_code(255), "²⁵⁵");
    }
}
