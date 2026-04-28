"""Utility script to purge canceled GitHub Action runs for CollapseLoader."""

import argparse
import logging
import os
import sys
from typing import Any, List, Sequence

import requests
from dotenv import load_dotenv

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)

logger = logging.getLogger("remove_canceled_actions")

load_dotenv()

GITHUB_TOKEN = os.getenv("GITHUB_TOKEN")
REPO_OWNER = "dest4590"
REPO_NAME = "CollapseLoader"
BASE_URL = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/actions/runs"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Delete GitHub Action runs that were canceled."
    )
    parser.add_argument(
        "-a",
        "--apply",
        action="store_true",
        help="Actually perform the deletion. Without this flag, the script runs in dry-run mode.",
    )
    return parser.parse_args(argv)


def build_headers(token: str) -> dict[str, str]:
    return {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json",
    }


def fetch_canceled_runs(headers: dict[str, str]) -> List[dict[str, Any]]:
    all_runs = []
    # GitHub API uses 'cancelled' with two 'l's
    url = f"{BASE_URL}?status=cancelled&per_page=100"

    try:
        while url:
            logger.info("Fetching canceled runs from %s", url)
            response = requests.get(url, headers=headers, timeout=30)
            response.raise_for_status()
            data = response.json()

            runs = data.get("workflow_runs", [])
            all_runs.extend(runs)

            url = None
            if "Link" in response.headers:
                links = response.headers["Link"].split(",")
                for link in links:
                    if 'rel="next"' in link:
                        url = link.split(";")[0].strip("<> ")
                        break

        logger.info("Total number of canceled runs found: %s", len(all_runs))
        return all_runs
    except requests.RequestException as exc:
        logger.error("Error fetching workflow runs: %s", exc)
        return all_runs


def delete_run(
    run: dict[str, Any], headers: dict[str, str], apply: bool = False
) -> bool:
    run_id = run.get("id")
    run_name = run.get("display_title") or run.get("name") or "Unknown Run"
    created_at = run.get("created_at", "unknown time")
    delete_url = f"{BASE_URL}/{run_id}"

    if not apply:
        logger.info(
            "[DRY RUN] Would delete run: ID=%s Name=%r Created=%s",
            run_id,
            run_name,
            created_at,
        )
        return True

    try:
        logger.info("Deleting run: ID=%s Name=%r", run_id, run_name)
        response = requests.delete(delete_url, headers=headers, timeout=30)
        if response.status_code == 204:
            logger.info("Successfully deleted run %s", run_id)
            return True
        logger.error(
            "Failed to delete run %s: Status Code %s",
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

    canceled_runs = fetch_canceled_runs(headers)
    if not canceled_runs:
        logger.info("No canceled workflow runs found.")
        return 0

    deleted_count = 0
    for run in canceled_runs:
        if delete_run(run, headers, args.apply):
            deleted_count += 1

    if not args.apply:
        logger.info(
            "Dry run complete. Use --apply or -a to actually remove %s canceled runs.",
            deleted_count,
        )
    else:
        logger.info("Deletion complete. Total runs removed: %s", deleted_count)

    return 0


if __name__ == "__main__":
    sys.exit(main())
