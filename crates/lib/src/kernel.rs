use anyhow::Result;
use fn_error_context::context;

/// This is used by dracut.
pub(crate) const INITRD_ARG_PREFIX: &str = "rd.";
/// The kernel argument for configuring the rootfs flags.
pub(crate) const ROOTFLAGS: &str = "rootflags=";

/// Parse the kernel command line.  This is strictly
/// speaking not a correct parser, as the Linux kernel
/// supports quotes.  However, we don't yet need that here.
///
/// See systemd's code for one userspace parser.
#[context("Reading /proc/cmdline")]
pub(crate) fn parse_cmdline() -> Result<Vec<String>> {
    let cmdline_bytes = std::fs::read("/proc/cmdline")?;
    let cmdline = String::from_utf8_lossy(&cmdline_bytes);
    let r = cmdline
        .split_ascii_whitespace()
        .map(ToOwned::to_owned)
        .collect();
    Ok(r)
}

/// Return the value for the string in the vector which has the form target_key=value
pub(crate) fn find_first_cmdline_arg<'a>(
    args: impl Iterator<Item = &'a str>,
    target_key: &str,
) -> Option<&'a str> {
    args.filter_map(|arg| {
        if let Some((k, v)) = arg.split_once('=') {
            if target_key == k {
                return Some(v);
            }
        }
        None
    })
    .next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_first() {
        let kargs = &["foo=bar", "root=/dev/vda", "blah", "root=/dev/other"];
        let kargs = || kargs.iter().copied();
        assert_eq!(find_first_cmdline_arg(kargs(), "root"), Some("/dev/vda"));
        assert_eq!(find_first_cmdline_arg(kargs(), "nonexistent"), None);
    }

    #[test]
    fn test_non_utf8_chars() {
        let raw_input = b"foo=\xFFbar root=/dev/vda blah".to_vec();
        let result_args = {
            let cmdline = String::from_utf8_lossy(&raw_input);
            cmdline
                .split_ascii_whitespace()
                .map(ToOwned::to_owned)
                .collect::<Vec<String>>()
        };
        assert_eq!(result_args[0], "foo=\u{FFFD}bar");
        assert_eq!(result_args[1], "root=/dev/vda");
        assert_eq!(
            find_first_cmdline_arg(result_args.iter().map(|s| s.as_str()), "foo"),
            Some("\u{FFFD}bar")
        );
    }
}
