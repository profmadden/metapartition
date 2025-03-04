# metapartition
Generic partitioner wrapper, with a general-purpose hyper graph interface


Can run with or without hMetis.  For hMetis on a Mac, add some feature flags:

<pre>
cargo run --features=hmetis --target=x86_64-apple-darwin -- -h benches/fs-save000.hgr -H
</pre>

The features toggles on compile with hMetis.  have to target x86_64, as that's
what the old library is built for.  Command line switch of -H to select the
hMetis partitioner (KaHyPar by default).

Also need to set the rpath on the Mac.  WTF?

<pre>
install_name_tool -add_rpath /usr/local/lib target/release/metapartition

install_name_tool -add_rpath /usr/local/lib target/x86_64-apple-darwin/release/metapartition
</pre>
