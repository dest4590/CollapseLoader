from collapse.modules.network.Configs import Config, Configs


def test_init_configs():
    configs = Configs()
    assert configs is not None

def test_init_config():
    config = Config(1, 'file.txt', 'configs/', 'test', 1)
    
    assert config.id == 1
    assert config.file == 'file.txt'
    assert config.config_path == 'configs/'
    assert config.server == 'test'