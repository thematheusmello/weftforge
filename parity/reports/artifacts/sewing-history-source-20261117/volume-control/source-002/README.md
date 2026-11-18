Synthetic fixture correction002 only.

The first sealed848 draft and its8-pass/1-fail root invocation remain untouched. The failed terminal-count test used the lexical macOS tempfile path under /var, whose ancestor is a symlink; monitor.no_links rejects it before mocked Popen, so kill_group cannot run.

The ONLY Python source/test change is Path(temporary).resolve() in that terminal-count test. All seven product/control variants, the no_links guard, kill assertion and product CONTROL-DELTA are byte-exact to848. The other eight tests remain exact.

Parent authorized one same-nine synthetic invocation using pinnedPython3.13 -B/optimization0,30seconds/1MiB, in the new002 runtime only after this source freeze. No Harbor/model/init/analyzer/GUI/binary/Rust/old-case operation is authorized. A green synthetic suite only makes the control repair ready for review; the next numerical packet/GO remains separate.
