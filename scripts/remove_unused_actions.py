import argparse
import logging
import os
import sys
from typing import Any, List, Sequence

import requests
from dotenv import load_dotenv
from tqdm import tqdm

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)

logger = logging.getLogger("remove_unused_actions")

load_dotenv()

GITHUB_TOKEN = os.getenv("GITHUB_TOKEN")
REPO_OWNER = "dest4590"
REPO_NAME = "CollapseLoader"
BASE_URL = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/actions/runs"
CHECK_FLAGS_JOB = "check-flags"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Delete GitHub Action runs that were cancelled OR where 'check-flags' "
            "completed and all other jobs were skipped."
        )
    )
    parser.add_argument(
        "-a",
        "--apply",
        action="store_true",
        help="Actually perform the deletion. Without this flag the script runs in dry-run mode.",
    )
    return parser.parse_args(argv)


def build_headers(token: str) -> dict[str, str]:
    return {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json",
    }


def paginate(
    url: str, headers: dict[str, str], max_items: int | None = None
) -> List[dict[str, Any]]:
    """Fetch all pages for a GitHub list endpoint and return combined items."""
    items: List[dict[str, Any]] = []
    while url:
        logger.debug("GET %s", url)
        response = requests.get(url, headers=headers, timeout=30)
        response.raise_for_status()
        data = response.json()

        if isinstance(data, list):
            items.extend(data)
        else:
            for key in ("workflow_runs", "jobs", "items"):
                if key in data:
                    items.extend(data[key])
                    break

        if max_items is not None and len(items) >= max_items:
            break

        url = None  # type: ignore
        link_header = response.headers.get("Link", "")
        for part in link_header.split(","):
            if 'rel="next"' in part:
                url = part.split(";")[0].strip("<> ")
                break

    return items[:max_items] if max_items is not None else items


def fetch_runs_by_status(
    headers: dict[str, str], status: str, max_runs: int | None = None
) -> List[dict[str, Any]]:
    url = f"{BASE_URL}?status={status}&per_page=100"
    logger.info("Fetching runs with status=%r …", status)
    try:
        runs = paginate(url, headers, max_runs)
        logger.info("  Found %d run(s) with status=%r", len(runs), status)
        return runs
    except requests.RequestException as exc:
        logger.error("Error fetching runs (status=%r): %s", status, exc)
        return []


def fetch_jobs(run_id: int, headers: dict[str, str]) -> List[dict[str, Any]]:
    url = f"{BASE_URL}/{run_id}/jobs?per_page=100"
    try:
        return paginate(url, headers)
    except requests.RequestException as exc:
        logger.error("Error fetching jobs for run %s: %s", run_id, exc)
        return []


def is_check_flags_only(run_id: int, headers: dict[str, str]) -> bool:
    """Return True when check-flags completed and every other job was skipped."""
    jobs = fetch_jobs(run_id, headers)
    if not jobs:
        return False

    check_flags_ok = False
    all_others_skipped = True

    for job in jobs:
        name = job.get("name", "")
        conclusion = job.get("conclusion", "")

        if name == CHECK_FLAGS_JOB:
            check_flags_ok = conclusion not in ("", None)
        else:
            if conclusion != "skipped":
                all_others_skipped = False

    return check_flags_ok and all_others_skipped


def delete_run(
    run: dict[str, Any], headers: dict[str, str], apply: bool, reason: str
) -> bool:
    run_id = run.get("id")
    run_name = run.get("display_title") or run.get("name") or "Unknown Run"
    created_at = run.get("created_at", "unknown time")
    delete_url = f"{BASE_URL}/{run_id}"

    if not apply:
        logger.info(
            "[DRY RUN] Would delete run: ID=%s  Name=%r  Created=%s  Reason=%s",
            run_id,
            run_name,
            created_at,
            reason,
        )
        return True

    try:
        logger.info("Deleting run ID=%s  Name=%r  Reason=%s", run_id, run_name, reason)
        response = requests.delete(delete_url, headers=headers, timeout=30)
        if response.status_code == 204:
            logger.info("Successfully deleted run %s", run_id)
            return True
        logger.error(
            "Failed to delete run %s: HTTP %s",
            run_id,
            response.status_code,
        )
    except requests.RequestException as exc:
        logger.error("Error deleting run %s: %s", run_id, exc)
    return False


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)

    token = GITHUB_TOKEN
    if not token:
        logger.error(
            "GITHUB_TOKEN is not set. Please provide a valid token in the environment."
        )
        return 1

    headers = build_headers(token)

    runs_to_delete: List[tuple[dict[str, Any], str]] = []

    for run in fetch_runs_by_status(headers, "cancelled"):
        runs_to_delete.append((run, "cancelled"))

    completed_runs = fetch_runs_by_status(headers, "completed", max_runs=30)
    logger.info(
        "Inspecting %d completed run(s) for check-flags-only pattern …",
        len(completed_runs),
    )
    for run in tqdm(completed_runs, desc="Inspecting completed runs"):
        run_id = run.get("id")
        if is_check_flags_only(run_id, headers):  # type: ignore
            runs_to_delete.append((run, "check-flags completed / others skipped"))

    if not runs_to_delete:
        logger.info("No runs matched the deletion criteria.")
        return 0

    logger.info("Total runs to delete: %d", len(runs_to_delete))

    deleted_count = 0
    for run, reason in runs_to_delete:
        if delete_run(run, headers, args.apply, reason):
            deleted_count += 1

    if not args.apply:
        logger.info(
            "Dry run complete. Use --apply / -a to actually remove %d run(s).",
            deleted_count,
        )
    else:
        logger.info("Done. Total runs removed: %d", deleted_count)

    return 0


if __name__ == "__main__":
    sys.exit(main())
