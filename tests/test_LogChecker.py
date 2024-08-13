import pytest

from collapse.modules.utils.LogChecker import LogChecker


@pytest.fixture
def logchecker():
    logchecker = LogChecker()
    yield logchecker

def test_LogChecker(logchecker: LogChecker):
    assert logchecker.check_logs('Game crashed!')
    assert logchecker.check_logs('Game crashed! java.lang.OutOfMemoryError: Java heap space')
    
def test_reason_LogChecker(logchecker: LogChecker):
    assert logchecker.reason('it has too little heap memory') == 'Game crashed because it has too little heap memory'