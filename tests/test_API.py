import pytest

from collapse.utils.API import API


@pytest.fixture
def api_client():
    return API()

def test_api_initialization(api_client: API):
    assert isinstance(api_client, API)

def test_api_get_request(api_client: API):
    response = api_client.get('clients')
    assert response.status_code == 200
    assert response.json() is not None