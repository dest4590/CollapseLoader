from collapse.utils.Configs import Configs, Config
from collapse.utils.Cheats import cheat_manager
from collapse.utils.Cheat import Cheat

def test_init_configs():
    config = Configs()
    assert config is not None
    
def test_config():
    cheat_manager.cheats.append(Cheat('test', 'test.zip'))
    
    config = Config(0, 'file', 'config_path', 1)
    assert config is not None
    assert config.line == 'test - file [red][Not installed][/]'