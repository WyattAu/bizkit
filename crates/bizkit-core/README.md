# bizkit-core

EDGAR full-text search client, filing store, and the
investment-by-risk idea taxonomy for
[bizkit](https://github.com/WyattAu/bizkit).

- `edgar.rs`: FTS client on fetch-kit, fixture-tested parsing
- `store.rs`: SQLite filing store with first/last-seen tracking
- `taxonomy.rs`: the idea matrix; every entry cites measured trendkit subjects

See the [workspace README](https://github.com/WyattAu/bizkit).
