import pytest

from collapse.modules.network.Updater import Updater


@pytest.fixture
def updater():
    return Updater()

def test_updater_initialization(updater: Updater):
    assert isinstance(updater, Updater)

def test_updater_fetch_commit_session(updater: Updater):
    with pytest.raises(Exception):
        updater.get_latest_commit()

def test_updater_fetch_latest_release(updater: Updater):
    with pytest.raises(Exception):
        updater.get_latest_releases()

def test_updater_fetch_latest_commit(updater: Updater):
    with pytest.raises(Exception):
        updater.get_latest_commit()