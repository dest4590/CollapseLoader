from collapse.modules.network.Network import network

def test_network():
    assert network is not None

def test_network_get():
    response = network.get('https://api.ipify.org?format=json')
    assert response is not None
    assert response.status_code == 200
    assert response.json() is not None