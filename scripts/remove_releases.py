"""Utility script to purge prerelease GitHub releases for CollapseLoader."""

import argparse
import logging
import os
import sys
from typing import Any, Iterable, List, Sequence

import requests
from dotenv import load_dotenv

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)

logger = logging.getLogger("remove_releases")

load_dotenv()

GITHUB_TOKEN = os.getenv("GITHUB_TOKEN")
REPO_OWNER = "dest4590"
REPO_NAME = "CollapseLoader"
PREFIX = "prerelease"
ALL_RELEASES_URL = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Delete GitHub releases whose tag names start with the prerelease prefix."
        )
    )
    parser.add_argument(
        "-a",
        "--all",
        action="store_true",
        help="Delete all prerelease releases (including the latest). Without this flag, the newest prerelease is kept.",
    )
    parser.add_argument(
        "--prefix",
        default=PREFIX,
        help="Tag prefix to target (default: %(default)s)",
    )
    return parser.parse_args(argv)


def build_headers(token: str) -> dict[str, str]:
    return {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json",
    }


def fetch_all_releases(headers: dict[str, str]) -> List[dict[str, Any]]:
    try:
        logger.info("Fetching all releases from %s", ALL_RELEASES_URL)
        response = requests.get(ALL_RELEASES_URL, headers=headers, timeout=30)
        response.raise_for_status()
        releases = response.json()
        logger.info("Number of releases fetched: %s", len(releases))
        return releases
    except requests.RequestException as exc:
        logger.error("Error fetching releases: %s", exc)
        return []


def describe_release(release: dict[str, Any]) -> str:
    tag = release.get("tag_name", "<unknown>")
    name = release.get("name") or "(no title)"
    created = release.get("created_at") or "unknown timestamp"
    return f"tag={tag} name={name!r} created_at={created}"


def filter_prereleases(
    releases: Iterable[dict[str, Any]], prefix: str
) -> List[dict[str, Any]]:
    pref = prefix.lower()
    filtered: List[dict[str, Any]] = []
    for release in releases:
        tag = (release.get("tag_name") or "").lower()
        if not tag:
            continue
        if release.get("draft"):
            logger.debug("Skipping draft release %s", describe_release(release))
            continue
        if tag.startswith(pref):
            filtered.append(release)
    return filtered


def delete_release(release: dict[str, Any], headers: dict[str, str]) -> bool:
    tag_name = release.get("tag_name")
    delete_url = release.get("url")
    if not delete_url or not tag_name:
        logger.error("Release payload missing url or tag_name: %s", release)
        return False

    try:
        logger.info("Deleting release: %s", describe_release(release))
        response = requests.delete(delete_url, headers=headers, timeout=30)
        if response.status_code == 204:
            logger.info("Successfully deleted release %s", tag_name)
            return True
        logger.error(
            "Failed to delete release %s: Status Code %s",
            tag_name,
            response.status_code,
        )
    except requests.RequestException as exc:
        logger.error("Error deleting release %s: %s", tag_name, exc)
    return False


def delete_tag(tag_name: str, headers: dict[str, str]) -> bool:
    delete_url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/git/refs/tags/{tag_name}"
    try:
        logger.info("Deleting tag: %s", tag_name)
        response = requests.delete(delete_url, headers=headers, timeout=30)
        if response.status_code == 204:
            logger.info("Successfully deleted tag %s", tag_name)
            return True
        logger.error(
            "Failed to delete tag %s: Status Code %s", tag_name, response.status_code
        )
    except requests.RequestException as exc:
        logger.error("Error deleting tag %s: %s", tag_name, exc)
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

    releases = fetch_all_releases(headers)
    if not releases:
        logger.warning("No releases retrieved. Exiting.")
        return 1

    prerelease_candidates = filter_prereleases(releases, args.prefix)

    if not prerelease_candidates:
        logger.info(
            "No prerelease releases with prefix '%s' found. Nothing to delete.",
            args.prefix,
        )
        return 0

    logger.info(
        "Found %s prerelease release(s) matching prefix '%s'.",
        len(prerelease_candidates),
        args.prefix,
    )

    to_delete = prerelease_candidates
    if not args.all:
        latest = prerelease_candidates[0]
        logger.info(
            "Keeping most recent prerelease: %s (use --all to delete it as well)",
            describe_release(latest),
        )
        to_delete = prerelease_candidates[1:]

    if not to_delete:
        logger.info("No releases left to delete after applying filters.")
        return 0

    deleted_releases = 0
    deleted_tags = 0

    for release in to_delete:
        tag_name = release.get("tag_name")
        if not tag_name:
            logger.warning("Skipping release without tag name: %s", release)
            continue

        if delete_release(release, headers):
            deleted_releases += 1
            if delete_tag(tag_name, headers):
                deleted_tags += 1
            else:
                logger.warning(
                    "Release %s deleted but failed to remove tag. Manual cleanup may be required.",
                    tag_name,
                )
        else:
            logger.warning(
                "Skipping tag deletion for %s because release removal did not succeed.",
                tag_name,
            )

    logger.info(
        "Deletion complete. Releases removed: %s, tags removed: %s",
        deleted_releases,
        deleted_tags,
    )

    return 0 if deleted_releases else 1


if __name__ == "__main__":
    sys.exit(main())
