import pytest

from collapse.modules.storage.Data import DataManager


@pytest.fixture
def data_manager():
    return DataManager()

def test_data_manager_initialization(data_manager: DataManager):
    assert isinstance(data_manager, DataManager)

def test_data_manager_fetch_data(data_manager: DataManager):
    data = data_manager.get_local('test.txt')
    assert data is not None