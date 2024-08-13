import asyncio
import webbrowser

import aiohttp

from ..utils.Module import Module
from ..render.CLI import selector
from ..storage.Data import data


class Updater(Module):
    """Handles checking for updates and opening download pages"""

    def __init__(self) -> None:
        super().__init__()
        self.latest_releases = []
        self.latest_release = None
        self.remote_version = None
        self.latest_commit = None
        self.local_version = data.version

        asyncio.run(self.initialize())

    async def initialize(self):
        try:
            async with aiohttp.ClientSession() as session:
                self.latest_releases = await self.get_latest_releases(session)
                self.latest_release = self.latest_releases[0]
                self.remote_version = self.get_remote_version()
                self.latest_commit = await self.get_latest_commit(session)

            self.debug(f'Remote: {self.remote_version}, local: {self.local_version}')
            self.debug(f'Latest commit: {self.latest_commit}')
        except aiohttp.ClientError as e:
            self.error(f'Error initializing Updater: {e}')
            self.remote_version = None
            self.latest_commit = None

    async def api_request(self, session: aiohttp.ClientSession, path: str, params: dict = None) -> dict:
        """Makes a request to the GitHub API"""
        url = f'https://api.github.com/repos/dest4590/CollapseLoader/{path}'
        try:
            async with session.get(url, params=params, timeout=5) as response:
                response.raise_for_status()
                if response.status == 403:
                    self.error('Rate limit exceeded, try again later')
                return await response.json()
        except aiohttp.ClientError as e:
            self.error(f'Failed to fetch {path}: {e}')
            raise

    async def get_latest_releases(self, session: aiohttp.ClientSession) -> dict:
        """Fetch releases from the GitHub API"""
        return await self.api_request(session, 'releases')

    def get_remote_version(self) -> str:
        """Fetch the latest remote version from the GitHub API without considering pre-releases"""
        latest_release = next((release for release in self.latest_releases if not release.get('prerelease')), None)
        if latest_release:
            return latest_release.get('tag_name')
        return None

    async def get_latest_commit(self, session: aiohttp.ClientSession) -> str:
        """Fetch the latest commit SHA from the GitHub API"""
        commits = await self.api_request(session, 'commits', {'per_page': 1})
        return commits[0].get('sha', '')[:7]

    def check_version(self) -> None:
        """Check if the local version is up to date with the remote version"""
        if self.remote_version and self.remote_version > self.local_version:
            self.info('Update your loader!')

            if selector.ask('Download a new version (y,n)'):
                if self.latest_releases:
                    if selector.ask('Dev version (y,n)'):  # Prelease
                        self.debug('Downloading dev version')
                        latest_prerelease = next((release for release in self.latest_releases if release.get('prerelease')), None)
                        if latest_prerelease and latest_prerelease.get('assets'):
                            webbrowser.open(latest_prerelease['assets'][0].get('browser_download_url'))
                    else:  # Release
                        self.debug('Downloading stable release')
                        latest_release = next((release for release in self.latest_releases if not release.get('prerelease')), None)
                        if latest_release and latest_release.get('assets'):
                            webbrowser.open(latest_release['assets'][0].get('browser_download_url'))
                else:
                    self.warn('No releases found')

updater = Updater()