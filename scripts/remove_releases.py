import logging
import os

import requests
from dotenv import load_dotenv

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)

load_dotenv()

GITHUB_TOKEN = os.getenv('GITHUB_TOKEN')
REPO_OWNER = 'dest4590'
REPO_NAME = 'CollapseLoader'
PREFIX = 'autorelease'

HEADERS = {
    'Authorization': f'token {GITHUB_TOKEN}',
    'Accept': 'application/vnd.github.v3+json'
}

LATEST_RELEASE_URL = f'https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest'
ALL_RELEASES_URL = f'https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases'


def fetch_latest_release():
    try:
        logging.info(f'Fetching latest release from {LATEST_RELEASE_URL}')
        response = requests.get(LATEST_RELEASE_URL, headers=HEADERS)
        response.raise_for_status()
        latest_release = response.json()
        latest_tag = latest_release.get('tag_name')
        logging.info(f'Latest release tag: {latest_tag}')
        return latest_tag
    except requests.RequestException as e:
        logging.error(f'Error fetching latest release: {e}')
        return None


def fetch_all_releases():
    try:
        logging.info(f'Fetching all releases from {ALL_RELEASES_URL}')
        response = requests.get(ALL_RELEASES_URL, headers=HEADERS)
        response.raise_for_status()
        releases = response.json()
        logging.info(f'Number of releases fetched: {len(releases)}')
        return releases
    except requests.RequestException as e:
        logging.error(f'Error fetching all releases: {e}')
        return []


def delete_release(release):
    tag_name = release.get('tag_name')
    delete_url = release.get('url')
    try:
        logging.info(f'Deleting release: {tag_name}')
        del_response = requests.delete(delete_url, headers=HEADERS)
        if del_response.status_code == 204:
            logging.info(f'Successfully deleted release: {tag_name}')
        else:
            logging.error(f'Failed to delete release {tag_name}: Status Code {del_response.status_code}')
    except requests.RequestException as e:
        logging.error(f'Error deleting release {tag_name}: {e}')


def main():
    latest_tag = fetch_latest_release()
    if not latest_tag:
        logging.error('Cannot proceed without latest release tag.')
        return

    releases = fetch_all_releases()
    for release in releases:
        tag_name = release.get('tag_name')
        if tag_name.startswith(PREFIX) and tag_name != latest_tag:
            delete_release(release)


if __name__ == '__main__':
    main()