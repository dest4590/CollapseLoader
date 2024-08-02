import pytest

from collapse.utils.Servers import Servers


@pytest.fixture
def servers():
    servers = Servers(['google.com', 'facebook.com', 'twitter.com'])
    yield servers

def test_check_servers(servers: Servers):
    assert servers.check_servers() == 'https://google.com/'