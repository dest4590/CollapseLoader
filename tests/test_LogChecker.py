import pytest

from collapse.modules.utils.LogChecker import LogChecker
from collapse.modules.utils.Client import Client


@pytest.fixture
def logchecker():
    logchecker = LogChecker()
    yield logchecker

@pytest.fixture
def client():
    client = Client('Test', link='https://google.com')
    yield client

def test_LogChecker(logchecker: LogChecker, client: Client):
    assert logchecker.check_logs('Game crashed!', client)
    assert logchecker.check_logs('Game crashed! java.lang.OutOfMemoryError: Java heap space', client)