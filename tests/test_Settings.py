import os

import pytest

from collapse.utils.Settings import Settings


@pytest.fixture
def settings():
    test_config_file = 'test_config.ini'
    settings = Settings(file=test_config_file)
    yield settings
    if os.path.exists(test_config_file):
        os.remove(test_config_file)

def test_settings_initialization(settings: Settings):
    assert settings.file == 'test_config.ini'
    assert os.path.exists(settings.config_path)

def test_settings_set_and_get(settings: Settings):
    settings.set('test_key', 'test_value')
    assert settings.get('test_key') == 'test_value'

def test_settings_save_and_load(settings: Settings):
    settings.set('test_key', 'test_value')
    settings.save()
    new_settings = Settings(file='test_config.ini')
    assert new_settings.get('test_key') == 'test_value'

def test_settings_use_option(settings: Settings):
    settings.set('test_option', 'True')
    assert settings.use_option('test_option') == False
    settings.set('test_option', 'False')
    assert settings.use_option('test_option') == True