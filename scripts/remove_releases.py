"""Utility script to purge prerelease GitHub releases for CollapseLoader."""

import argparse
import logging
import os
import sys
import urllib.parse
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
DEFAULT_PREFIXES = ["build-", "prerelease-"]
ALL_RELEASES_URL = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases"
ALL_TAGS_URL = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/tags"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Delete GitHub releases whose tag names start with the prerelease prefix."
        )
    )
    parser.add_argument(
        "-A",
        "--all",
        action="store_true",
        help="Delete all matching releases/tags (including the latest). Without this flag, the newest one is kept.",
    )
    parser.add_argument(
        "--prefix",
        action="append",
        help=f"Tag prefix to target (can be specified multiple times, default: {', '.join(DEFAULT_PREFIXES)})",
    )
    parser.add_argument(
        "-p",
        "--prerelease",
        action="store_true",
        help="Target all releases marked as pre-release, regardless of prefix.",
    )
    parser.add_argument(
        "-a",
        "--apply",
        action="store_true",
        help="Actually perform the deletion. Without this flag, the script runs in dry-run mode.",
    )
    parser.add_argument(
        "--tags-only",
        action="store_true",
        help="Only delete tags, do not attempt to delete releases (useful if releases are already gone).",
    )
    return parser.parse_args(argv)


def build_headers(token: str) -> dict[str, str]:
    return {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json",
    }


def fetch_all_releases(headers: dict[str, str]) -> List[dict[str, Any]]:
    all_releases = []
    url = ALL_RELEASES_URL
    try:
        while url:
            logger.info("Fetching releases from %s", url)
            response = requests.get(url, headers=headers, timeout=30)
            response.raise_for_status()
            page_releases = response.json()
            all_releases.extend(page_releases)

            url = None
            if "Link" in response.headers:
                links = response.headers["Link"].split(",")
                for link in links:
                    if 'rel="next"' in link:
                        url = link.split(";")[0].strip("<> ")
                        break

        logger.info("Total number of releases fetched: %s", len(all_releases))
        return all_releases
    except requests.RequestException as exc:
        logger.error("Error fetching releases: %s", exc)
        return all_releases


def fetch_all_tags(headers: dict[str, str]) -> List[dict[str, Any]]:
    all_tags = []
    url = ALL_TAGS_URL
    try:
        while url:
            logger.info("Fetching tags from %s", url)
            response = requests.get(url, headers=headers, timeout=30)
            response.raise_for_status()
            page_tags = response.json()
            all_tags.extend(page_tags)

            url = None
            if "Link" in response.headers:
                links = response.headers["Link"].split(",")
                for link in links:
                    if 'rel="next"' in link:
                        url = link.split(";")[0].strip("<> ")
                        break

        logger.info("Total number of tags fetched: %s", len(all_tags))
        return all_tags
    except requests.RequestException as exc:
        logger.error("Error fetching tags: %s", exc)
        return all_tags


def describe_release(release: dict[str, Any]) -> str:
    tag = release.get("tag_name", "<unknown>")
    name = release.get("name") or "(no title)"
    created = release.get("created_at") or "unknown timestamp"
    return f"tag={tag} name={name!r} created_at={created}"


def filter_tags(
    tags: Iterable[dict[str, Any]], prefixes: List[str]
) -> List[dict[str, Any]]:
    prefixes_lower = [p.lower() for p in prefixes]
    filtered: List[dict[str, Any]] = []
    for tag_obj in tags:
        tag_name = (tag_obj.get("name") or "").lower()
        if not tag_name:
            continue

        matches_prefix = any(tag_name.startswith(p) for p in prefixes_lower)
        if matches_prefix:
            filtered.append(tag_obj)
    return filtered


def delete_release(
    release: dict[str, Any], headers: dict[str, str], apply: bool = False
) -> bool:
    tag_name = release.get("tag_name")
    delete_url = release.get("url")
    if not delete_url or not tag_name:
        logger.error("Release payload missing url or tag_name: %s", release)
        return False

    if not apply:
        logger.info("[DRY RUN] Would delete release: %s", describe_release(release))
        return True

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


def delete_tag(tag_name: str, headers: dict[str, str], apply: bool = False) -> bool:
    encoded_tag = urllib.parse.quote(tag_name)
    delete_url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/git/refs/tags/{encoded_tag}"

    if not apply:
        logger.info("[DRY RUN] Would delete tag: %s", tag_name)
        return True

    try:
        logger.info("Deleting tag: %s", tag_name)
        response = requests.delete(delete_url, headers=headers, timeout=30)
        if response.status_code == 204:
            logger.info("Successfully deleted tag %s", tag_name)
            return True
        if response.status_code == 404:
            logger.info("Tag %s already deleted or not found.", tag_name)
            return True
        logger.error(
            "Failed to delete tag %s: Status Code %s. Response: %s",
            tag_name,
            response.status_code,
            response.text,
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
    tag_to_release = {r.get("tag_name"): r for r in releases if r.get("tag_name")}

    tags = fetch_all_tags(headers)
    if not tags:
        logger.warning("No tags retrieved. Exiting.")
        return 1

    prefixes = args.prefix if args.prefix else DEFAULT_PREFIXES

    matching_tags = filter_tags(tags, prefixes)

    if args.prerelease:
        for tag_name, release in tag_to_release.items():
            if release.get("prerelease") and not any(
                t.get("name") == tag_name for t in matching_tags
            ):
                matching_tags.append({"name": tag_name})

    if not matching_tags:
        logger.info("No tags matching prefixes %s found.", prefixes)
        return 0

    logger.info("Found %s matching tag(s).", len(matching_tags))

    to_delete = matching_tags
    if not args.all:
        latest = matching_tags[0]
        logger.info(
            "Keeping most recent tag: %s (use --all to delete it as well)",
            latest.get("name"),
        )
        to_delete = matching_tags[1:]

    if not to_delete:
        logger.info("No tags left to delete after applying filters.")
        return 0

    deleted_releases = 0
    deleted_tags = 0

    for tag_obj in to_delete:
        tag_name = tag_obj.get("name")
        if not tag_name:
            continue

        release = tag_to_release.get(tag_name)
        if release and not args.tags_only:
            if delete_release(release, headers, args.apply):
                deleted_releases += 1
            else:
                logger.warning(
                    "Failed to delete release for tag %s, will still try to delete tag.",
                    tag_name,
                )

        if delete_tag(tag_name, headers, args.apply):
            deleted_tags += 1

    if not args.apply:
        logger.info(
            "Dry run complete. Use --apply or -a to actually remove %s releases and %s tags.",
            deleted_releases,
            deleted_tags,
        )
    else:
        logger.info(
            "Deletion complete. Releases removed: %s, tags removed: %s",
            deleted_releases,
            deleted_tags,
        )

    return 0 if deleted_tags else 1


if __name__ == "__main__":
    sys.exit(main())
