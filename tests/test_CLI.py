import mock
import pytest

from collapse.modules.render.CLI import Selector


@pytest.fixture
def selector():
    return Selector()


def test_selector(selector: Selector):
    with mock.patch("builtins.input", return_value="test"):
        assert selector.select() == "test"


def test_selector_ask_yes(selector: Selector):
    with mock.patch("builtins.input", return_value="y"):
        assert selector.ask("test") == True


def test_selector_ask_no(selector: Selector):
    with mock.patch("builtins.input", return_value="n"):
        assert selector.ask("test") == False


def test_select_username(selector: Selector):
    with mock.patch("builtins.input", return_value="test"):
        assert selector.select_username() == "test"
