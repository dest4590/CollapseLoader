Testing the updater with a mock GitHub release

1. Start the mock server

Open a terminal and run:

```powershell
python .\scripts\serve_mock_release.py
```

The server will run at `http://127.0.0.1:8000` and serve `docs/mock_release.json` at the path `/repos/<owner>/<repo>/releases/latest`.

2. Point the updater to the mock server

Option A (recommended for quick testing): temporarily modify the request URL in `src-tauri/src/commands/updater.rs`.

-   Find the `url` variable where the GitHub API URL is constructed and replace it with:

```rust
let url = "http://127.0.0.1:8000/repos/dest4590/CollapseLoader/releases/latest".to_string();
```

-   Rebuild and run the app (or the Tauri backend) so the updater fetches from the mock server.

Option B: Use a local HTTP proxy or hosts trick to map `api.github.com` to `127.0.0.1` (not recommended unless you know what you're doing).

3. Test behavior

-   With the mock release served, trigger update check in the app.
-   The updater should fetch the mock release JSON, parse the `changelog` fenced block, and if `translations` is present it will be merged into the app i18n at runtime.
-   If `download_url` points to an external URL (example in mock_release.json uses `https://example.com/...`), the updater will try to download; you can edit the mock JSON to point to a local file or to a small test server.

4. Revert changes

-   After testing, revert any temporary URL changes in `updater.rs` and rebuild.
